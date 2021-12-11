use crate::util::*;

const B: usize = 6;

#[derive(Clone, Copy)]
pub struct ElemIdx(Idx);

pub trait BTreeInfo
where
    Self: Sized + Copy + core::ops::Add<Self, Output = Self> + Default,
{
}

pub trait BTreeItem
where
    Self: Sized,
{
    type BTreeInfo: BTreeInfo;

    fn get_info(&self) -> Self::BTreeInfo;
}

// Use SOA stuff eventually: https://github.com/lumol-org/soa-derive
// They even have a trait! Very cute.
pub struct BTree<T>
where
    T: BTreeItem,
    <T as BTreeItem>::BTreeInfo: BTreeInfo,
{
    elements: Vec<T>,
    element_info: Vec<ElementInfo>,
    nodes: Vec<Node<T::BTreeInfo>>,
    first_free: Option<Idx>,
    root: Idx,
    levels: usize,
}

impl<T> BTree<T>
where
    T: BTreeItem,
{
    pub fn new() -> Self {
        let root_node = Node::leaf();

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

    pub fn info(&self) -> T::BTreeInfo {
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

    pub fn key<F>(&self, index: usize, getter: F) -> Option<(&T, usize)>
    where
        F: Fn(<T as BTreeItem>::BTreeInfo) -> usize,
    {
        let (idx, remainder) = self._key_idx(index, getter)?;
        return Some((&self.elements[idx.get()], remainder));
    }

    pub fn key_mut<F>(&mut self, index: usize, getter: F) -> Option<(&T, usize)>
    where
        F: Fn(<T as BTreeItem>::BTreeInfo) -> usize,
    {
        let (idx, remainder) = self._key_idx(index, getter)?;
        return Some((&mut self.elements[idx.get()], remainder));
    }

    pub fn key_idx<F>(&self, index: usize, getter: F) -> Option<(ElemIdx, usize)>
    where
        F: Fn(<T as BTreeItem>::BTreeInfo) -> usize,
    {
        let (idx, remainder) = self._key_idx(index, getter)?;
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

    fn insert_into_leaf(&mut self, mut node: Idx, index: usize, elem: T) {
        let (elem_info, mut right) = self.add_to_leaf(node, index, elem);
        for _ in 0..self.levels {
            let parent = self.nodes[node.get()].parent;
            self.nodes[parent.get()].assert_not_leaf();

            let to_insert = match right.take() {
                Some(right) => right,
                None => {
                    // parent references are correct so everythings a-ok
                    let parent_ref = &mut self.nodes[parent.get()];
                    parent_ref.count += 1;
                    parent_ref.info = parent_ref.info + elem_info;

                    node = parent;
                    continue;
                }
            };

            let mut kids_iter = self.nodes[parent.get()].kids.into_iter();
            let node_index = kids_iter.position(|kid| kid.get() == node.get()).unwrap() + 1;
            let kids = match self.nodes[parent.get()].kids.insert(node_index, to_insert) {
                Some(right_kids) => right_kids,
                None => {
                    // inserted successfully
                    self.nodes[to_insert.get()].parent = parent;
                    let parent_ref = &mut self.nodes[parent.get()];
                    parent_ref.info = parent_ref.info + elem_info;
                    parent_ref.count += 1;

                    node = parent;
                    continue;
                }
            };

            node = parent;
            let parent = ();

            let right_idx = Idx::new(self.nodes.len());
            let (mut count, mut info) = (0, Default::default());
            for kid in &kids {
                self.nodes[kid.get()].parent = right_idx;
                info = info + self.elements[kid.get()].get_info();
                count += 1;
            }

            let mut right_node = Node::new();
            right_node.kids = kids;
            right_node.info = info;
            right_node.count = count;
            right_node.parent = self.nodes[node.get()].parent; // not really necessary
            self.nodes.push(right_node);

            let kids = self.nodes[node.get()].kids;
            let (mut count, mut info) = (0, <T::BTreeInfo as Default>::default());
            for kid in &kids {
                info = info + self.elements[kid.get()].get_info();
                count += 1;
            }

            let node_ = &mut self.nodes[node.get()];
            node_.info = info;
            node_.count = count;
        }

        let right = match right {
            Some(right) => right,
            None => return,
        };

        let root_idx = Idx::new(self.nodes.len());
        let mut root = Node::new();
        root.kids.insert(0, node);
        root.kids.insert(1, right);

        let (node_ref, right_ref) = (self.nodes[node.get()], self.nodes[right.get()]);
        root.info = node_ref.info + right_ref.info;
        root.count = node_ref.count + right_ref.count;
        root.parent = root_idx;
        self.nodes[node.get()].parent = root_idx;
        self.nodes[right.get()].parent = root_idx;

        self.nodes.push(root);

        self.root = root_idx;
        self.levels += 1;
    }

    fn add_to_leaf(&mut self, node: Idx, index: usize, elem: T) -> (T::BTreeInfo, Option<Idx>) {
        self.nodes[node.get()].assert_is_leaf();

        let elem_agg_info = elem.get_info();
        let elem_idx = match self.first_free.take() {
            Some(idx) => {
                self.elements[idx.get()] = elem;
                let elem_info = &mut self.element_info[idx.get()];
                self.first_free = elem_info.next_free.take();

                idx
            }
            None => {
                let idx = self.elements.len();
                self.elements.push(elem);
                self.element_info.push(ElementInfo {
                    parent: Idx::new(0),
                    next_free: None,
                });

                Idx::new(idx)
            }
        };

        let kids = match self.nodes[node.get()].kids.insert(index, elem_idx) {
            None => {
                self.element_info[elem_idx.get()].parent = node;
                let node_ref = &mut self.nodes[node.get()];
                node_ref.info = node_ref.info + elem_agg_info;
                node_ref.count += 1;
                return (elem_agg_info, None);
            }
            Some(right) => right,
        };

        let right_idx = Idx::new(self.nodes.len());
        {
            let (mut count, mut info) = (0, Default::default());
            for kid in &kids {
                self.element_info[kid.get()].parent = right_idx;
                info = info + self.elements[kid.get()].get_info();
                count += 1;
            }

            let mut right_node = Node::leaf();
            right_node.kids = kids;
            right_node.info = info;
            right_node.count = count;
            right_node.parent = self.nodes[node.get()].parent; // not really necessary
            self.nodes.push(right_node);
        }

        let kids = self.nodes[node.get()].kids;
        let (mut count, mut info) = (0, Default::default());
        for kid in &kids {
            info = info + self.elements[kid.get()].get_info();
            count += 1;
        }

        let node_ = &mut self.nodes[node.get()];
        node_.info = info;
        node_.count = count;

        return (elem_agg_info, Some(right_idx));
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

    fn _key_idx<F>(&self, index: usize, getter: F) -> Option<(Idx, usize)>
    where
        F: Fn(<T as BTreeItem>::BTreeInfo) -> usize,
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
    parent: Idx, // For the root, I dont think this matters. Maybe it'll point to itself.
    kids: ChildArray,
}

impl<Info> Node<Info>
where
    Info: BTreeInfo,
{
    #[inline(always)]
    fn leaf() -> Self {
        let mut node = Self::new();

        #[cfg(debug_assertions)]
        {
            node.is_leaf = true;
        }

        return node;
    }

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

    fn new() -> Self {
        #[cfg(debug_assertions)]
        return Node {
            is_leaf: false,
            count: 0,
            info: Default::default(),
            parent: Idx::new(0),
            kids: ChildArray::new(),
        };

        #[cfg(not(debug_assertions))]
        return Node {
            count: 0,
            info: Default::default(),
            parent: Idx::new(0),
            kids: ChildArray::new(),
        };
    }
}

#[derive(Clone, Copy)]
struct ChildArray {
    value: [Option<Idx>; B],
}

impl ChildArray {
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

impl core::ops::Index<usize> for ChildArray {
    type Output = Idx;

    fn index(&self, index: usize) -> &Self::Output {
        return self.value[index].as_ref().unwrap();
    }
}

impl core::ops::IndexMut<usize> for ChildArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return self.value[index].as_mut().unwrap();
    }
}

type DerefIdx = fn(&Option<Idx>) -> Option<Idx>;
impl<'a> core::iter::IntoIterator for &'a ChildArray {
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

    impl core::ops::Add for TestData {
        type Output = Self;

        fn add(self, other: Self) -> Self::Output {
            return Self(self.0 + other.0);
        }
    }

    impl BTreeInfo for TestData {}

    impl BTreeItem for TestData {
        type BTreeInfo = Self;

        fn get_info(&self) -> Self::BTreeInfo {
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
