use crate::util::*;

const B: usize = 6;

#[derive(Clone, Copy)]
pub struct ElemIdx(Idx);

pub trait BTreeInfo
where
    Self: Sized + Copy + Default,
{
    fn add(self, other: Self) -> Self;
}

pub trait BTreeItem
where
    Self: Sized,
{
    type Info: BTreeInfo;

    fn get_info(&self) -> Self::Info;
}

// Use SOA stuff eventually: https://github.com/lumol-org/soa-derive
// They even have a trait! Very cute.
pub struct BTree<T>
where
    T: BTreeItem,
{
    elements: Vec<T>,
    element_info: Vec<ElementInfo>,
    nodes: Vec<Node<T::Info>>,
    first_free: Option<Idx>,
    root: Idx,
    levels: usize,
}

impl<T> BTree<T>
where
    T: BTreeItem,
{
    pub fn new() -> Self {
        let root_node = Node::empty(true);

        return Self {
            elements: Vec::new(),
            element_info: Vec::new(),
            nodes: vec![root_node],
            first_free: None,
            root: Idx::new(0),
            levels: 0,
        };
    }

    pub fn len(&self) -> usize {
        return self.nodes[self.root.get()].count;
    }

    pub fn info(&self) -> T::Info {
        return self.nodes[self.root.get()].info;
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let idx = self._get_idx(index)?;
        return Some(&self.elements[idx.get()]);
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let idx = self._get_idx(index)?;
        return Some(&mut self.elements[idx.get()]);
    }

    pub fn get_idx(&self, index: usize) -> Option<ElemIdx> {
        let idx = self._get_idx(index)?;
        return Some(ElemIdx(idx));
    }

    pub fn key_inclusive<F>(&self, index: usize, getter: F) -> Option<(&T, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self._key_idx(true, index, getter)?;
        return Some((&self.elements[idx.get()], remainder));
    }

    pub fn key_inclusive_mut<F>(&mut self, index: usize, getter: F) -> Option<(&T, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self._key_idx(true, index, getter)?;
        return Some((&mut self.elements[idx.get()], remainder));
    }

    pub fn key_inclusive_idx<F>(&self, index: usize, getter: F) -> Option<(ElemIdx, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self._key_idx(true, index, getter)?;
        return Some((ElemIdx(idx), remainder));
    }

    pub fn key<F>(&self, index: usize, getter: F) -> Option<(&T, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self._key_idx(false, index, getter)?;
        return Some((&self.elements[idx.get()], remainder));
    }

    pub fn key_mut<F>(&mut self, index: usize, getter: F) -> Option<(&T, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self._key_idx(false, index, getter)?;
        return Some((&mut self.elements[idx.get()], remainder));
    }

    pub fn key_idx<F>(&self, index: usize, getter: F) -> Option<(ElemIdx, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self._key_idx(false, index, getter)?;
        return Some((ElemIdx(idx), remainder));
    }

    #[inline]
    pub fn add(&mut self, element: T) {
        self.insert(self.nodes[self.root.get()].count, element);
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        if index > self.nodes[self.root.get()].count {
            panic!("insert index was too high");
        }

        let (mut node, mut index) = (self.root, index);
        'to_leaves: for _ in 0..self.levels {
            for child in &self.nodes[node.get()].kids {
                let count = self.nodes[child.get()].count;
                if index <= count {
                    node = child;
                    continue 'to_leaves;
                }

                index -= count;
            }

            unreachable!();
        }

        self.insert_into_leaf(node, index, elem);
    }

    pub fn insert_before(&mut self, index: ElemIdx, elem: T) {
        let leaf = self.element_info[index.0.get()].parent;
        let mut kids_iter = self.nodes[leaf.get()].kids.into_iter();
        let index = kids_iter.position(|kid| kid == index.0).unwrap();

        self.insert_into_leaf(leaf, index, elem);
    }

    pub fn insert_after(&mut self, index: ElemIdx, elem: T) {
        let leaf = self.element_info[index.0.get()].parent;
        let mut kids_iter = self.nodes[leaf.get()].kids.into_iter();
        let index = kids_iter.position(|kid| kid == index.0).unwrap();

        self.insert_into_leaf(leaf, index + 1, elem);
    }

    fn insert_into_leaf(&mut self, mut node: Idx, index: usize, elem: T) {
        let info = elem.get_info();
        let elem = self.allocate_elem(node, elem);

        let mut right = self.add_child(node, index, elem, info).map(|kids| {
            self.update_node(true, node);
            return self.new_node(true, kids);
        });

        for _ in 0..self.levels {
            let parent = self.nodes[node.get()].parent.unwrap();
            self.nodes[parent.get()].assert_not_leaf();

            let to_insert = match right.take() {
                Some(right) => right,
                None => {
                    // parent references are correct so everythings a-ok
                    let parent_ref = &mut self.nodes[parent.get()];
                    parent_ref.count += 1;
                    parent_ref.info = parent_ref.info.add(info);

                    node = parent;
                    continue;
                }
            };

            self.nodes[to_insert.get()].parent = Some(parent);
            let mut kids_iter = self.nodes[parent.get()].kids.into_iter();
            let node_index = kids_iter.position(|kid| kid == node).unwrap() + 1;
            let kids = self.add_child(parent, node_index, to_insert, info);
            right = kids.map(|kids| {
                self.update_node(false, parent);
                self.new_node(false, kids)
            });

            node = parent;
        }

        right.map(|right| {
            self.root = self.new_node(false, [node, right]);
            self.levels += 1;
        });
    }

    fn allocate_elem(&mut self, parent: Idx, elem: T) -> Idx {
        self.nodes[parent.get()].assert_is_leaf();

        match self.first_free.take() {
            Some(idx) => {
                self.elements[idx.get()] = elem;
                let elem_info = &mut self.element_info[idx.get()];
                self.first_free = elem_info.next_free.take();

                return idx;
            }
            None => {
                let idx = self.elements.len();
                self.elements.push(elem);
                self.element_info.push(ElementInfo {
                    parent,
                    next_free: None,
                });

                return Idx::new(idx);
            }
        };
    }

    fn add_child(&mut self, node: Idx, at: usize, child: Idx, info: T::Info) -> Option<Kids> {
        let kids = self.nodes[node.get()].kids.insert(at, child);
        if kids.is_none() {
            let node_ref = &mut self.nodes[node.get()];
            node_ref.info = node_ref.info.add(info);
            node_ref.count += 1;
        }

        return kids;
    }

    fn update_node(&mut self, is_leaf: bool, node: Idx) {
        let kids = self.nodes[node.get()].kids;
        let (mut count, mut info) = (0, T::Info::default());
        for kid in &kids {
            let (kid_info, kid_count) = if is_leaf {
                assert!(self.element_info[kid.get()].parent == node);
                (self.elements[kid.get()].get_info(), 1)
            } else {
                let kid = &self.nodes[kid.get()];
                assert!(kid.parent == Some(node));
                (kid.info, kid.count)
            };

            info = info.add(kid_info);
            count += kid_count;
        }

        let node_ = &mut self.nodes[node.get()];
        node_.info = info;
        node_.count = count;
    }

    fn new_node(&mut self, is_leaf: bool, kids: impl Into<Kids>) -> Idx {
        let kids: Kids = kids.into();
        let idx = Idx::new(self.nodes.len());
        let (mut count, mut info) = (0, T::Info::default());
        for kid in &kids {
            let (kid_info, kid_count) = if is_leaf {
                self.element_info[kid.get()].parent = idx;
                (self.elements[kid.get()].get_info(), 1)
            } else {
                let kid = &mut self.nodes[kid.get()];
                kid.parent = Some(idx);
                (kid.info, kid.count)
            };

            info = info.add(kid_info);
            count += kid_count;
        }

        let mut right_node = Node::empty(is_leaf);
        right_node.kids = kids;
        right_node.info = info;
        right_node.count = count;

        self.nodes.push(right_node);

        return idx;
    }

    fn _get_idx(&self, index: usize) -> Option<Idx> {
        let mut node = self.nodes[self.root.get()];
        if index >= node.count {
            return None;
        }

        let mut running = index;
        'outer: for _ in 0..self.levels {
            for child_idx in &node.kids {
                let child = self.nodes[child_idx.get()];
                if running < child.count {
                    node = child;
                    continue 'outer;
                }

                running -= child.count;
            }

            unreachable!();
        }

        let index = node.kids.into_iter().nth(running);
        return Some(index.unwrap());
    }

    fn _key_idx<F>(&self, inclusive: bool, index: usize, getter: F) -> Option<(Idx, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let mut node = self.nodes[self.root.get()];
        if index >= getter(node.info) {
            return None;
        }

        let mut running = index;
        'outer: for _ in 0..self.levels {
            for child_idx in &node.kids {
                let child = self.nodes[child_idx.get()];
                let sum = getter(child.info);
                if running < sum {
                    node = child;
                    continue 'outer;
                }

                if inclusive && running == sum {
                    node = child;
                    continue 'outer;
                }

                running -= sum;
            }

            unreachable!();
        }

        for idx in &node.kids {
            let info = self.elements[idx.get()].get_info();
            let size = getter(info);
            if running < size {
                return Some((idx, running));
            }

            running -= size;
        }

        unreachable!();
    }
}

#[derive(Clone, Copy)]
struct ElementInfo {
    parent: Idx,
    next_free: Option<Idx>,
}

// We're using the trick from Basic algo with the combining lists thing! From
// the 2-3 tree PSet. Very cute.
//                      - Albert Liu, Dec 06, 2021 Mon 19:11 EST
#[derive(Clone, Copy)]
struct Node<Info>
where
    Info: BTreeInfo,
{
    #[cfg(debug_assertions)]
    is_leaf: bool,
    info: Info,
    count: usize,
    parent: Option<Idx>, // For the root, I dont think this matters. Maybe it'll point to itself.
    kids: Kids,
}

impl<Info> Node<Info>
where
    Info: BTreeInfo,
{
    #[inline(always)]
    fn assert_not_leaf(&self) {
        #[cfg(debug_assertions)]
        if self.is_leaf {
            panic!("thought it wouldnt be a leaf but it was")
        }
    }

    #[inline(always)]
    fn assert_is_leaf(&self) {
        #[cfg(debug_assertions)]
        if !self.is_leaf {
            panic!("thought it would be a leaf but it wasnt")
        }
    }

    fn empty(is_leaf: bool) -> Self {
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
struct Kids {
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

    fn new() -> Self {
        return Self { value: [None; B] };
    }

    fn insert(&mut self, mut index: usize, mut value: Idx) -> Option<Self> {
        if index > B || (index > 0 && self.value[index - 1].is_none()) {
            panic!("index out of bounds");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Default, Debug)]
    struct TestData(usize);

    impl BTreeInfo for TestData {
        fn add(self, other: Self) -> Self {
            return Self(self.0 + other.0);
        }
    }

    impl BTreeItem for TestData {
        type Info = Self;

        fn get_info(&self) -> Self::Info {
            return TestData(self.0);
        }
    }

    #[test]
    fn forward_insert() {
        let mut tree = BTree::new();

        for i in 0..8 {
            println!("iter: {}", i);
            tree.insert(i, TestData(i));
            for j in 0..(i + 1) {
                dbg!(tree.get(j).unwrap().0);
            }
        }

        for i in 0..8 {
            assert_eq!(i, tree.get(i).unwrap().0);
        }

        let mut total = 0;
        for i in 0..8 {
            let next = total + i;
            for (idx, key) in (total..next).enumerate() {
                let (value, remainder) = tree.key(key, |n| n.0).unwrap();
                assert_eq!(i, value.0);
                assert_eq!(idx, remainder);
            }

            total = next;
        }

        println!("finished");
    }

    #[test]
    fn reverse_insert() {
        let mut tree = BTree::new();

        for i in 0..8 {
            println!("iter: {}", i);
            tree.insert(0, TestData(7 - i));
            for j in 0..(i + 1) {
                dbg!(tree.get(j).unwrap().0);
            }
        }

        for i in 0..8 {
            assert_eq!(i, tree.get(i).unwrap().0);
        }

        let mut total = 0;
        for i in 0..8 {
            let next = total + i;
            for (idx, key) in (total..next).enumerate() {
                let (value, remainder) = tree.key(key, |n| n.0).unwrap();
                assert_eq!(i, value.0);
                assert_eq!(idx, remainder);
            }

            total = next;
        }

        println!("finished");
    }
}
