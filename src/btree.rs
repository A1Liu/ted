use crate::util::*;

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

    pub fn get(&self, index: impl BTreeIdx<T>) -> Option<&T> {
        let idx = index.get(self)?;
        return Some(&self.elements[idx.get()]);
    }

    pub fn key<F>(&self, index: usize, get: F) -> Option<(&T, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self.find(false, index, move |_, info| get(info))?;
        return Some((&self.elements[idx.get()], remainder));
    }

    pub fn key_leq<F>(&self, index: usize, get: F) -> Option<(&T, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self.find(true, index, move |_, info| get(info))?;
        return Some((&self.elements[idx.get()], remainder));
    }

    pub fn get_idx(&self, index: usize) -> Option<ElemIdx> {
        let (idx, _) = self.find(false, index, |count, _| count)?;
        return Some(e_idx(idx));
    }

    pub fn key_idx<F>(&self, index: usize, get: F) -> Option<(ElemIdx, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self.find(false, index, move |_, info| get(info))?;
        return Some((e_idx(idx), remainder));
    }

    pub fn key_leq_idx<F>(&self, index: usize, get: F) -> Option<(ElemIdx, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self.find(true, index, move |_, info| get(info))?;
        return Some((e_idx(idx), remainder));
    }

    // We can't return a mutable reference here because we need to update the
    // bookkeeping data after the mutation finishes
    pub fn get_mut<E, F>(&mut self, index: impl BTreeIdx<T>, f: F) -> Option<E>
    where
        F: Fn(&mut T) -> E,
    {
        let idx = index.get(self)?.get();

        let elem = &mut self.elements[idx];
        let result = f(elem);

        let mut node = self.element_info[idx].parent;
        self.update_node(true, node);
        for _ in 0..self.levels {
            node = self.nodes[node.get()].parent.unwrap();
            self.update_node(false, node);
        }

        debug_assert_eq!(node, self.root);

        return Some(result);
    }

    pub fn count_until(&self, index: impl BTreeIdx<T>) -> Option<usize> {
        let mut count = 0;

        let elem_idx = index.get(self)?;

        let mut node = self.element_info[elem_idx.get()].parent;
        for kid in &self.nodes[node.get()].kids {
            if kid == elem_idx {
                break;
            }

            count += 1;
        }

        for _ in 0..self.levels {
            let parent = self.nodes[node.get()].parent.unwrap();

            for kid in &self.nodes[parent.get()].kids {
                if kid == node {
                    break;
                }

                count += self.nodes[kid.get()].count;
            }

            node = parent;
        }

        return Some(count);
    }

    pub fn sum_until(&self, index: impl BTreeIdx<T>) -> Option<T::Info> {
        let mut sum = <T::Info as Default>::default();

        let elem_idx = index.get(self)?;

        let mut node = self.element_info[elem_idx.get()].parent;
        for kid in &self.nodes[node.get()].kids {
            if kid == elem_idx {
                break;
            }

            sum = sum.add(self.elements[kid.get()].get_info());
        }

        for _ in 0..self.levels {
            let parent = self.nodes[node.get()].parent.unwrap();

            for kid in &self.nodes[parent.get()].kids {
                if kid == node {
                    break;
                }

                sum = sum.add(self.nodes[kid.get()].info);
            }

            node = parent;
        }

        return Some(sum);
    }

    pub fn last_idx(&self) -> Option<ElemIdx> {
        let (idx, _) = self.find(true, self.len(), |count, _| count)?;

        return Some(ElemIdx(idx));
    }

    #[inline]
    pub fn add(&mut self, element: T) -> ElemIdx {
        return self.insert(self.nodes[self.root.get()].count, element);
    }

    pub fn insert(&mut self, index: usize, elem: T) -> ElemIdx {
        if index > self.nodes[self.root.get()].count {
            core::panic!("insert index was too high");
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

            core::unreachable!();
        }

        return self.insert_into_leaf(node, index, elem);
    }

    pub fn insert_before(&mut self, index: ElemIdx, elem: T) -> ElemIdx {
        let leaf = self.element_info[index.0.get()].parent;
        let mut kids_iter = self.nodes[leaf.get()].kids.into_iter();
        let index = kids_iter.position(|kid| kid == index.0).unwrap();

        return self.insert_into_leaf(leaf, index, elem);
    }

    pub fn insert_after(&mut self, index: ElemIdx, elem: T) -> ElemIdx {
        let leaf = self.element_info[index.0.get()].parent;
        let mut kids_iter = self.nodes[leaf.get()].kids.into_iter();
        let index = kids_iter.position(|kid| kid == index.0).unwrap();

        return self.insert_into_leaf(leaf, index + 1, elem);
    }

    fn insert_into_leaf(&mut self, mut node: Idx, index: usize, elem: T) -> ElemIdx {
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

        return e_idx(elem);
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

    pub fn find<F>(&self, inclusive: bool, mut key: usize, get: F) -> Option<(Idx, usize)>
    where
        F: Fn(usize, T::Info) -> usize,
    {
        let mut node = &self.nodes[self.root.get()];
        if key >= get(node.count, node.info) {
            return None;
        }

        'outer: for _ in 0..self.levels {
            for child_idx in &node.kids {
                let child = &self.nodes[child_idx.get()];
                let val = get(child.count, child.info);
                if key < val {
                    node = child;
                    continue 'outer;
                }

                if inclusive && key == val {
                    node = child;
                    continue 'outer;
                }

                key -= val;
            }

            // This is probably the implementation of T::Info being wrong
            return None;
        }

        for idx in &node.kids {
            let info = self.elements[idx.get()].get_info();
            let val = get(1, info);
            if key < val {
                return Some((idx, key));
            }

            if inclusive && key == val {
                return Some((idx, key));
            }

            key -= val;
        }

        // This is probably the implementation of T::Info being wrong
        return None;
    }
}

impl<T, I> core::ops::Index<I> for BTree<T>
where
    T: BTreeItem,
    I: BTreeIdx<T>,
{
    type Output = T;
    fn index(&self, idx: I) -> &T {
        self.get(idx).unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct ElemIdx(Idx);

fn e_idx(i: Idx) -> ElemIdx {
    return ElemIdx(i);
}

pub trait BTreeIdx<T>
where
    T: BTreeItem,
{
    fn get(self, tree: &BTree<T>) -> Option<Idx>;
}

impl<T> BTreeIdx<T> for usize
where
    T: BTreeItem,
{
    fn get(self, tree: &BTree<T>) -> Option<Idx> {
        let (idx, _) = tree.find(false, self, |count, _| count)?;
        return Some(idx);
    }
}

impl<T> BTreeIdx<T> for ElemIdx
where
    T: BTreeItem,
{
    fn get(self, tree: &BTree<T>) -> Option<Idx> {
        return Some(self.0);
    }
}

#[derive(Clone, Copy)]
struct ElementInfo {
    parent: Idx,
    next_free: Option<Idx>,
}

const B: usize = 6;

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
            core::panic!("thought it wouldnt be a leaf but it was")
        }
    }

    #[inline(always)]
    fn assert_is_leaf(&self) {
        #[cfg(debug_assertions)]
        if !self.is_leaf {
            core::panic!("thought it would be a leaf but it wasnt")
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

    const TREE_SIZE: usize = 100000;

    fn validate(tree: BTree<TestData>) {
        for i in 0..TREE_SIZE {
            assert_eq!(i, tree.get(i).unwrap().0);
        }

        let mut total = 0;
        for i in 0..TREE_SIZE {
            let next = total + i;
            if next == total {
                continue;
            }

            let test_cases = [
                total,
                (3 * total + next) / 4,
                (total + next) / 2,
                (total + 3 * next) / 4,
                next - 1,
            ];
            for key in test_cases.into_iter() {
                let (value, remainder) = tree.key(key, |n| n.0).unwrap();
                assert_eq!(i, value.0);
                assert_eq!(key - total, remainder);

                let sum = tree.sum_until(i).unwrap().0;
                assert_eq!(total, sum);
            }

            total = next;
        }
    }

    #[test]
    fn forward_insert() {
        let mut tree = BTree::new();

        for i in 0..TREE_SIZE {
            tree.insert(i, TestData(i));
        }

        validate(tree);
    }

    #[test]
    fn reverse_insert() {
        let mut tree = BTree::new();

        for i in 0..TREE_SIZE {
            tree.insert(0, TestData(TREE_SIZE - 1 - i));
        }

        validate(tree);
    }
}
