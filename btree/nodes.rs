use crate::traits::*;
use crate::util::*;

const B: usize = 6;

// We're using the trick from Basic algo with the combining lists thing! From
// the 2-3 tree PSet. Very cute.
//                      - Albert Liu, Dec 06, 2021 Mon 19:11 EST
#[derive(Clone, Copy)]
pub struct Node<Info>
where
    Info: BTreeInfo,
{
    #[cfg(debug_assertions)]
    pub is_leaf: bool,
    pub info: Info,
    pub count: usize,
    pub parent: Option<Idx>, // For the root, I dont think this matters. Maybe it'll point to itself.
    pub kids: Kids,
}

impl<Info> Node<Info>
where
    Info: BTreeInfo,
{
    #[inline(always)]
    pub fn assert_not_leaf(&self) {
        #[cfg(debug_assertions)]
        if self.is_leaf {
            core::panic!("thought it wouldnt be a leaf but it was")
        }
    }

    #[inline(always)]
    pub fn assert_is_leaf(&self) {
        #[cfg(debug_assertions)]
        if !self.is_leaf {
            core::panic!("thought it would be a leaf but it wasnt")
        }
    }

    pub fn empty(is_leaf: bool) -> Self {
        #[cfg(debug_assertions)]
        return Node {
            is_leaf,
            count: 0,
            info: Default::default(),
            parent: None,
            kids: Kids::new(),
        };

        #[cfg(not(debug_assertions))]
        return Node {
            count: 0,
            info: Default::default(),
            parent: None,
            kids: Kids::new(),
        };
    }
}

#[derive(Clone, Copy)]
pub struct Kids {
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

impl Kids {
    pub fn rev(self) -> impl Iterator<Item = Idx> {
        return self.value.into_iter().rev().filter_map(|i| i);
    }
}
