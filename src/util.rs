use core::num::NonZeroUsize;
pub use mint::*;
pub use winit::event_loop::EventLoopProxy;
pub use winit::window::Window;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[macro_use]
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;

    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen::JsCast;

    pub type JsFunc = dyn 'static + FnMut() -> Result<(), JsValue>;

    pub fn enclose(f: impl 'static + FnMut() -> Result<(), JsValue>) -> Closure<JsFunc> {
        return Closure::wrap(Box::new(f) as Box<JsFunc>);
    }

    pub fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
        let window = unwrap(web_sys::window());
        let document = unwrap(window.document());
        let canvas = unwrap(document.get_element_by_id("canvas"));
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        return Ok(canvas);
    }

    #[macro_export]
    macro_rules! out {
        ($str:expr, $( $e:expr ),+ ) => {{
            #[cfg(debug_assertions)]
            {
                out!(@CLEAN, core::concat!("[{}:{}]: ", $str, "\n"), file!(), line!(), $( $e ),+ );
            }
        }};
        (@CLEAN, $str:expr, $( $e:expr ),+ ) => {{
            let s = format!( $str, $( $e ),+ );
            $crate::util::wasm::console_log(&s);
        }};
    }

    #[macro_export]
    macro_rules! dbg {
        ($fmt:literal) => {{
             out!("{}", $fmt);
        }};
        ($fmt:literal, $( $e:expr ),+ ) => {{
             out!($fmt, $( $e ),+ );
        }};
        ($expr:expr) => {{
            out!("{} = {:?}", stringify!($expr), $expr);
        }};
        () => {{
            out!("{}", "Nothing to see here");
        }};
    }

    #[macro_export]
    macro_rules! println {
        ($fmt:literal) => {{
             out!("{}", $fmt);
        }};
        ($fmt:literal, $( $e:expr ),+ ) => {{
             out!($fmt, $( $e ),+ );
        }};
        ($expr:expr) => {{
             out!("{} = {:?}", stringify!($expr), $expr);
        }};
        () => {{
            out!("{}", "Nothing to see here");
        }};
    }

    #[macro_export]
    macro_rules! print {
        ( $( $arg:tt )* ) => {{
            println!( $( $arg )* );
        }};
    }

    #[macro_export]
    macro_rules! panic {
        ( $( $arg:tt )* ) => {{
            println!( $( $arg )* );
            #[cfg(debug_assertions)]
            core::panic!();

            #[cfg(not(debug_assertions))]
            core::arch::wasm32::unreachable();
        }};
    }

    #[macro_export]
    macro_rules! unreachable {
        ( $( $arg:tt )* ) => {{
            panic!()
        }};
    }

    pub fn console_log(a: &str) {
        log(a);
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(a: &str);
    }
}

#[cfg(debug_assertions)]
pub fn expect<V, E>(res: Result<V, E>) -> V
where
    E: core::fmt::Debug,
{
    return res.unwrap();
}

#[cfg(not(debug_assertions))]
pub fn expect<V, E>(res: Result<V, E>) -> V {
    let err = match res {
        Ok(v) => return v,
        Err(err) => err,
    };

    panic!("Expected value");
}

pub fn unwrap<V>(opt: Option<V>) -> V {
    if let Some(v) = opt {
        return v;
    }

    panic!("Expected value");
}

pub type Rect = Vector2<u32>;

pub const fn new_rect(x: u32, y: u32) -> Rect {
    return Vector2 { x, y };
}

// ----------------------------------------------------------------------------
//
//                                  POD ARRAY
//
// ----------------------------------------------------------------------------
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

    pub fn remove(&mut self, i: usize) -> T {
        let len = self.raw.length;
        if i >= len {
            panic!("invalid index");
        }

        unsafe {
            let ptr = self.raw.ptr(i) as *mut T;
            let value = *ptr;

            // Shift everything down to fill in that spot.
            core::ptr::copy(ptr.add(self.raw.info.size), ptr, len - i - 1);
            self.raw.length = len - 1;

            return value;
        }
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

    fn slice(&self, r: core::ops::Range<usize>) -> Option<(*mut T, usize)> {
        if r.end > self.raw.length || r.end < r.start {
            return None;
        }

        let data = self.raw.ptr(r.start);
        let len = r.end - r.start;

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
    pub fn get_slice(&self, r: core::ops::Range<usize>) -> Option<&[T]> {
        let (ptr, len) = self.slice(r)?;

        return Some(unsafe { core::slice::from_raw_parts(ptr, len) });
    }

    #[inline(always)]
    pub fn get_mut_slice(&mut self, r: core::ops::Range<usize>) -> Option<&mut [T]> {
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

impl<T, E, A, B> core::cmp::PartialEq<Pod<E, B>> for Pod<T, A>
where
    T: Copy + core::cmp::PartialEq<E>,
    A: Allocator,
    E: Copy,
    B: Allocator,
{
    fn eq(&self, other: &Pod<E, B>) -> bool {
        use core::ops::Deref;

        return self.deref() == other.deref();
    }
}

impl<T, A> core::ops::Deref for Pod<T, A>
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

impl<T, A> core::ops::DerefMut for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        return unwrap(self.get_mut_slice(0..self.raw.length));
    }
}

impl<T, A> core::ops::Index<usize> for Pod<T, A>
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

impl<T, A> core::ops::IndexMut<usize> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: usize) -> &mut T {
        return unwrap(self.get_mut(i));
    }
}

impl<T, A> core::ops::Index<core::ops::RangeTo<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: core::ops::RangeTo<usize>) -> &[T] {
        return unwrap(self.get_slice(0..i.end));
    }
}

impl<T, A> core::ops::IndexMut<core::ops::RangeTo<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: core::ops::RangeTo<usize>) -> &mut [T] {
        return unwrap(self.get_mut_slice(0..i.end));
    }
}

impl<T, A> core::ops::Index<core::ops::RangeFrom<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: core::ops::RangeFrom<usize>) -> &[T] {
        return unwrap(self.get_slice(i.start..self.raw.length));
    }
}

impl<T, A> core::ops::IndexMut<core::ops::RangeFrom<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: core::ops::RangeFrom<usize>) -> &mut [T] {
        return unwrap(self.get_mut_slice(i.start..self.raw.length));
    }
}

impl<T, A> core::ops::Index<core::ops::RangeFull> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: core::ops::RangeFull) -> &[T] {
        return unwrap(self.get_slice(0..self.raw.length));
    }
}

impl<T, A> core::ops::IndexMut<core::ops::RangeFull> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: core::ops::RangeFull) -> &mut [T] {
        return unwrap(self.get_mut_slice(0..self.raw.length));
    }
}

impl<T, A> core::ops::Index<core::ops::Range<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, i: core::ops::Range<usize>) -> &[T] {
        return unwrap(self.get_slice(i));
    }
}

impl<T, A> core::ops::IndexMut<core::ops::Range<usize>> for Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, i: core::ops::Range<usize>) -> &mut [T] {
        return unwrap(self.get_mut_slice(i));
    }
}

// ----------------------------------------------------------------------------
//
//                               POD ARRAY UTILS
//
// ----------------------------------------------------------------------------
use crate::alloc_api::*;
use alloc::alloc::Layout;
use core::ptr::NonNull;

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
    fn ptr(&self, i: usize) -> *mut u8 {
        return unsafe { self.data.as_ptr().add(self.info.size * i) };
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
