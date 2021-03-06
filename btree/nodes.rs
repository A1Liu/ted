use crate::traits::*;
use crate::util::*;

const B: usize = 6;

// We're using the trick from Basic algo with the combining lists thing! From
// the 2-3 tree PSet. Very cute.
//                      - Albert Liu, Dec 06, 2021 Mon 19:11 EST
#[derive(Clone, Copy)]
pub(crate) struct Node<Info>
where
    Info: BTreeInfo,
{
    // is_leaf is necessary here to allow for moving the nodes when swap-removing
    pub(crate) is_leaf: bool,
    pub(crate) info: Info,
    pub(crate) count: usize,
    pub(crate) parent: Option<Idx>,
    pub(crate) kids: Kids,
}

impl<Info> core::fmt::Debug for Node<Info>
where
    Info: BTreeInfo, // + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        return f
            .debug_struct("Node")
            .field("is_leaf", &self.is_leaf)
            .field("count", &self.count)
            .field("parent", &NoPrettyPrint(self.parent))
            .field("kids", &NoPrettyPrint(self.kids))
            .finish();
    }
}

impl<Info> Node<Info>
where
    Info: BTreeInfo,
{
    pub fn assert_not_leaf(&self) {
        #[cfg(debug_assertions)]
        if self.is_leaf {
            core::panic!("thought it wouldnt be a leaf but it was")
        }
    }

    pub fn assert_is_leaf(&self) {
        #[cfg(debug_assertions)]
        if !self.is_leaf {
            core::panic!("thought it would be a leaf but it wasnt")
        }
    }

    pub fn empty(is_leaf: bool) -> Self {
        return Node {
            is_leaf,
            count: 0,
            info: Default::default(),
            parent: None,
            kids: Kids::new(),
        };
    }
}

#[derive(Clone, Copy)]
pub struct Kids {
    // First none is end of array
    value: [Option<Idx>; B],
}

impl From<[Idx; 2]> for Kids {
    fn from(value: [Idx; 2]) -> Self {
        let mut sel = Self::new();
        sel.value[0] = Some(value[0]);
        sel.value[1] = Some(value[1]);

        return sel;
    }
}

impl Kids {
    const SPLIT_POINT: usize = B / 2 + 1;

    pub fn new() -> Self {
        return Self { value: [None; B] };
    }

    pub fn remove_value(&mut self, value: Idx) -> usize {
        let mut write_index = 0;
        for index in 0..B {
            let prev = match self.value[index].take() {
                Some(prev) => prev,
                None => return write_index,
            };

            if prev != value {
                self.value[write_index] = Some(prev);
                write_index += 1;
            }
        }

        return write_index;
    }

    pub fn insert(&mut self, mut index: usize, mut value: Idx) -> Option<Self> {
        if index > B || (index > 0 && self.value[index - 1].is_none()) {
            core::panic!("index out of bounds");
        }

        while index < B {
            value = self.value[index].replace(value)?;
            index += 1;
        }

        let mut other = Self::new();
        let mut index = 0;
        for item in self.value[Self::SPLIT_POINT..].iter().map(|c| c.unwrap()) {
            other.value[index] = Some(item);
            index += 1;
        }

        other.value[index] = Some(value);

        for slot in &mut self.value[Self::SPLIT_POINT..] {
            *slot = None;
        }

        return Some(other);
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Idx> + 'a {
        return self.into_iter();
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut Idx> + 'a {
        return self.into_iter();
    }

    pub fn rev(self) -> impl Iterator<Item = Idx> {
        return self.value.into_iter().rev().filter_map(|i| i);
    }
}

impl core::ops::Index<usize> for Kids {
    type Output = Idx;

    fn index(&self, index: usize) -> &Self::Output {
        return self.value[index].as_ref().unwrap();
    }
}

impl core::ops::IndexMut<usize> for Kids {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return self.value[index].as_mut().unwrap();
    }
}

type DerefIdx = fn(&Option<Idx>) -> Option<Idx>;
impl<'a> core::iter::IntoIterator for &'a Kids {
    type Item = Idx;
    type IntoIter = core::iter::FilterMap<core::slice::Iter<'a, Option<Idx>>, DerefIdx>;

    fn into_iter(self) -> Self::IntoIter {
        return self.value.iter().filter_map(|i| *i);
    }
}

type DerefMutIdx = fn(&mut Option<Idx>) -> Option<&mut Idx>;
impl<'a> core::iter::IntoIterator for &'a mut Kids {
    type Item = &'a mut Idx;
    type IntoIter = core::iter::FilterMap<core::slice::IterMut<'a, Option<Idx>>, DerefMutIdx>;

    fn into_iter(self) -> Self::IntoIter {
        return self.value.iter_mut().filter_map(|i| i.as_mut());
    }
}

impl core::fmt::Debug for Kids {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        return f.debug_list().entries(self).finish();
    }
}
