use super::alloc_api::*;
use super::CopyRange;
use super::{expect, unwrap};
use alloc::alloc::Layout;
use core::num::NonZeroUsize;
use core::ops::*;
use core::ptr::NonNull;

#[macro_export]
macro_rules! pod {
    ($elem:expr; $n:expr) => {{
        let n : usize = $n;
        let elem = $elem;

        let mut pod = $crate::util::Pod::with_capacity(n);
        pod.push_repeat(elem, n);

        pod
    }};
    ($($e:expr),* $(,)?) => {{
        let data = [ $( $e ),+ ];
        let mut pod = $crate::util::Pod::with_capacity(data.len());

        for value in data.into_iter() {
            pod.push(value);
        }

        pod
    }};
}

struct DataInfo {
    size: usize,
    align: usize,
}

// 2 purposes: Prevent monomorphization as much as possible, and allow for using
// the allocator API on stable.
pub struct Pod<T, A = Global>
where
    T: Copy,
    A: Allocator,
{
    raw: RawPod,
    allocator: A,
    phantom: core::marker::PhantomData<T>,
}

impl<T> Pod<T, Global>
where
    T: Copy,
{
    #[inline(always)]
    pub fn new() -> Self {
        return Self::with_allocator(Global);
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut s = Self::new();
        s.raw.realloc(&Global, capacity);

        return s;
    }
}

impl<T, A> Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    pub fn with_allocator(allocator: A) -> Self {
        let info = DataInfo {
            size: core::mem::size_of::<T>(),
            align: core::mem::align_of::<T>(),
        };

        return Self {
            raw: RawPod::new(info),
            allocator,
            phantom: core::marker::PhantomData,
        };
    }

    pub fn push(&mut self, t: T) {
        self.raw.reserve(&self.allocator, 1);

        let ptr = self.raw.ptr(self.raw.length) as *mut T;
        self.raw.length += 1;

        unsafe { *ptr = t };
    }

    pub fn clear(&mut self) {
        self.raw.length = 0;
    }

    pub fn insert(&mut self, i: usize, value: T) {
        self.raw.reserve(&self.allocator, 1);
        self.raw.length += 1;

        if self.raw.copy_range(i..self.raw.length, i + 1) {
            panic!("invalid position");
        }

        let ptr = self.raw.ptr(i) as *mut T;
        unsafe { *ptr = value };
    }

    pub fn splice<R, I>(&mut self, range: R, mut values: I)
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        let range = self.raw.translate_range(range);
        let mut iter = values.into_iter();
        self._splice(range, &mut iter);
    }

    fn _splice(&mut self, range: Range<usize>, mut values: &mut dyn Iterator<Item = T>) {
        let (start, end) = (range.start, range.end);

        if !self.raw.range_is_valid(start, end) {
            panic!("invalid range");
        }

        let mut current = start;
        while current < end {
            let value = match values.next() {
                Some(value) => value,
                None => {
                    self.raw.copy_range(end..self.raw.length, current);
                    self.raw.length = self.raw.length - end + current;

                    return;
                }
            };

            let ptr = self.raw.ptr(current) as *mut T;
            unsafe { *ptr = value };
            current += 1;
        }

        let (lower_bound, upper_bound) = values.size_hint();
        let bound = upper_bound.unwrap_or(lower_bound);

        let mut remainder = Pod::with_capacity(bound);
        for value in values {
            remainder.push(value);
        }

        let remainder_len = remainder.len();
        self.raw.reserve(&self.allocator, remainder_len);

        let len = self.raw.length;
        self.raw.length += remainder_len;

        if self.raw.copy_range(end..len, end + remainder_len) {
            panic!("idk wth?");
        }

        for value in remainder {
            let ptr = self.raw.ptr(current) as *mut T;
            unsafe { *ptr = value };
            current += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.raw.length == 0 {
            return None;
        }

        let ptr = self.raw.ptr(self.raw.length - 1) as *const T;
        let value = unsafe { *ptr };

        return Some(value);
    }

    pub fn remove(&mut self, i: usize) -> T {
        let value = self[i];

        self.raw.copy_range((i + 1)..self.raw.length, i);
        self.raw.length -= 1;

        return value;
    }

    pub fn push_repeat(&mut self, t: T, repeat: usize) {
        self.raw.reserve(&self.allocator, repeat);

        let ptr = self.raw.ptr(self.raw.length) as *mut T;
        let data = unsafe { core::slice::from_raw_parts_mut(ptr, repeat) };
        data.fill(t);

        self.raw.length += repeat;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        return self.raw.length;
    }

    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(&self.allocator, additional);
    }

    #[inline(always)]
    pub fn shrink_to_fit(&mut self) {
        let len = self.raw.length;
        self.raw.realloc(&self.allocator, len);
    }

    #[inline(always)]
    pub fn raw_ptr(&self, i: usize) -> Option<*mut T> {
        let data = self.raw.ptr(i);

        return Some(data as *mut T);
    }

    fn ptr(&self, i: usize) -> Option<*mut T> {
        if i >= self.raw.length {
            return None;
        }

        let data = self.raw.ptr(i);

        return Some(data as *mut T);
    }

    fn slice(&self, r: Range<usize>) -> Option<(*mut T, usize)> {
        let (start, end) = (r.start, r.end);
        if !self.raw.range_is_valid(start, end) {
            return None;
        }

        let data = self.raw.ptr(start);
        let len = end - start;

        return Some((data as *mut T, len));
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&T> {
        let ptr = self.ptr(i)?;

        return Some(unsafe { &*ptr });
    }

    #[inline(always)]
    pub fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        let ptr = self.ptr(i)?;

        return Some(unsafe { &mut *ptr });
    }

    #[inline(always)]
    pub fn get_slice(&self, r: Range<usize>) -> Option<&[T]> {
        let (ptr, len) = self.slice(r)?;

        return Some(unsafe { core::slice::from_raw_parts(ptr, len) });
    }

    #[inline(always)]
    pub fn get_mut_slice(&mut self, r: Range<usize>) -> Option<&mut [T]> {
        let (ptr, len) = self.slice(r)?;

        return Some(unsafe { core::slice::from_raw_parts_mut(ptr, len) });
    }
}

pub struct PodIter<T, A>
where
    T: Copy,
    A: Allocator,
{
    pod: Pod<T, A>,
    index: usize,
}

impl<T, A> Iterator for PodIter<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let index = self.index;
        self.index += 1;

        let value = self.pod.get(index)?;

        return Some(*value);
    }
}

impl<T, A> IntoIterator for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type IntoIter = PodIter<T, A>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        return PodIter {
            pod: self,
            index: 0,
        };
    }
}

impl<T, A> core::fmt::Debug for Pod<T, A>
where
    T: Copy + core::fmt::Debug,
    A: Allocator,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        return f.debug_list().entries(self.iter()).finish();
    }
}

impl<T, E, A, B> core::cmp::PartialEq<Pod<E, B>> for Pod<T, A>
where
    T: Copy + core::cmp::PartialEq<E>,
    A: Allocator,
    E: Copy,
    B: Allocator,
{
    fn eq(&self, other: &Pod<E, B>) -> bool {
        return self.deref() == other.deref();
    }
}

impl<T, A> Deref for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &[T] {
        return unwrap(self.get_slice(0..self.raw.length));
    }
}

impl<T, A> DerefMut for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        return unwrap(self.get_mut_slice(0..self.raw.length));
    }
}

impl<T, A> Index<usize> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = T;

    #[inline(always)]
    fn index(&self, i: usize) -> &T {
        return unwrap(self.get(i));
    }
}

impl<T, A> IndexMut<usize> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: usize) -> &mut T {
        return unwrap(self.get_mut(i));
    }
}

impl<T, A> Index<CopyRange> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: CopyRange) -> &[T] {
        return unwrap(self.get_slice(i.start..i.end));
    }
}

impl<T, A> IndexMut<CopyRange> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: CopyRange) -> &mut [T] {
        return unwrap(self.get_mut_slice(i.start..i.end));
    }
}

impl<T, A> Index<RangeTo<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: RangeTo<usize>) -> &[T] {
        return unwrap(self.get_slice(0..i.end));
    }
}

impl<T, A> IndexMut<RangeTo<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: RangeTo<usize>) -> &mut [T] {
        return unwrap(self.get_mut_slice(0..i.end));
    }
}

impl<T, A> Index<RangeFrom<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: RangeFrom<usize>) -> &[T] {
        return unwrap(self.get_slice(i.start..self.raw.length));
    }
}

impl<T, A> IndexMut<RangeFrom<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: RangeFrom<usize>) -> &mut [T] {
        return unwrap(self.get_mut_slice(i.start..self.raw.length));
    }
}

impl<T, A> Index<RangeFull> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: RangeFull) -> &[T] {
        return unwrap(self.get_slice(0..self.raw.length));
    }
}

impl<T, A> IndexMut<RangeFull> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: RangeFull) -> &mut [T] {
        return unwrap(self.get_mut_slice(0..self.raw.length));
    }
}

impl<T, A> Index<Range<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: Range<usize>) -> &[T] {
        return unwrap(self.get_slice(i));
    }
}

impl<T, A> IndexMut<Range<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: Range<usize>) -> &mut [T] {
        return unwrap(self.get_mut_slice(i));
    }
}

// ----------------------------------------------------------------------------
//
//                               POD ARRAY UTILS
//
// ----------------------------------------------------------------------------

struct RawPod {
    data: NonNull<u8>,
    info: DataInfo,
    length: usize,
    capacity: usize,
}

impl RawPod {
    fn new(info: DataInfo) -> Self {
        // We use the same trick that std::vec::Vec uses
        return Self {
            data: NonNull::dangling(),
            info,
            length: 0,
            capacity: 0,
        };
    }

    #[inline(always)]
    fn range_is_valid(&self, start: usize, end: usize) -> bool {
        return start <= end && end <= self.length;
    }

    fn translate_range(&self, range: impl RangeBounds<usize>) -> Range<usize> {
        let start = match range.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(e) => *e + 1,
            Bound::Excluded(e) => *e,
            Bound::Unbounded => self.length,
        };

        return start..end;
    }

    #[inline(always)]
    fn ptr(&self, i: usize) -> *mut u8 {
        return unsafe { self.data.as_ptr().add(self.info.size * i) };
    }

    fn copy_range(&mut self, range: Range<usize>, to: usize) -> bool {
        let (start, end) = (range.start, range.end);

        if !self.range_is_valid(start, end) {
            return true;
        }

        let src = self.ptr(start);
        let dest = self.ptr(to);
        let copy_len = end - start;

        // Shift everything down to fill in that spot.
        unsafe { core::ptr::copy(src, dest, self.info.size * copy_len) };

        return false;
    }

    fn reserve(&mut self, alloc: &dyn Allocator, additional: usize) {
        let needed = self.length + additional;
        if needed <= self.capacity {
            return;
        }

        let new_capacity = core::cmp::max(needed, self.capacity * 3 / 2);
        self.realloc(alloc, new_capacity);
    }

    fn realloc(&mut self, alloc: &dyn Allocator, capacity: usize) {
        let (size, align) = (self.info.size, self.info.align);
        let get_info = move |mut data: NonNull<[u8]>| -> (NonNull<u8>, usize) {
            let data = unsafe { data.as_mut() };
            let capacity = unwrap(data.len().checked_div(size));
            let data = unsafe { NonNull::new_unchecked(data.as_mut_ptr()) };

            return (data, capacity);
        };

        // We use the same trick that std::vec::Vec uses
        let (data, capacity) = match (size * self.capacity, size * capacity) {
            (x, y) if x == y => return,
            (0, 0) => {
                self.capacity = capacity;
                return;
            }

            (prev_size, 0) => {
                let layout = expect(Layout::from_size_align(prev_size, align));
                unsafe { alloc.deallocate(self.data, layout) };

                (NonNull::dangling(), capacity)
            }

            (0, new_size) => {
                let layout = expect(Layout::from_size_align(new_size, align));
                let data = expect(alloc.allocate(layout));

                get_info(data)
            }

            (prev_size, new_size) => {
                let prev_layout = expect(Layout::from_size_align(prev_size, align));
                let new_layout = expect(Layout::from_size_align(new_size, align));

                let result = unsafe {
                    if new_size > prev_size {
                        alloc.grow(self.data, prev_layout, new_layout)
                    } else {
                        alloc.shrink(self.data, prev_layout, new_layout)
                    }
                };

                let data = expect(result);

                get_info(data)
            }
        };

        self.data = data;
        self.length = core::cmp::min(self.length, capacity);
        self.capacity = capacity;
    }

    fn with_capacity(info: DataInfo, alloc: &dyn Allocator, capacity: usize) -> Self {
        // We use the same trick that std::vec::Vec uses
        let mut s = Self::new(info);
        s.realloc(alloc, capacity);

        return s;
    }
}
