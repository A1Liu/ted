use crate::util::*;

const B: usize = 6;

pub trait BTreeInfo
where
    Self: Sized + Copy + core::ops::Add<Self, Output = Self> + Default,
{
}

pub trait BTreeItem
where
    Self: Sized,
{
    type BTreeInfo;

    fn get_info(&self) -> Self::BTreeInfo;
}

#[derive(Clone, Copy)]
struct ElementInfo {
    parent: Idx,
    next_free: Option<Idx>,
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
    root: usize,
    levels: usize,
}

impl<T> BTree<T>
where
    T: BTreeItem,
    <T as BTreeItem>::BTreeInfo: BTreeInfo,
{
    pub fn new() -> Self {
        let root_node = Node::new();

        return Self {
            elements: Vec::new(),
            element_info: Vec::new(),
            nodes: vec![root_node],
            first_free: None,
            root: 0,
            levels: 0,
        };
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let idx = self._get_idx(index)?;
        return Some(&self.elements[idx.get()]);
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let idx = self._get_idx(index)?;
        return Some(&mut self.elements[idx.get()]);
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

    pub fn insert(&mut self, index: usize, elem: T) {
        if index > self.nodes[self.root].count {
            panic!("insert index was too high");
        }

        let elem_agg_info = elem.get_info();
        let to_insert = match self.first_free.take() {
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

        let (mut node, mut index) = (Idx::new(self.root), index);
        'to_leaves: for _ in 0..self.levels {
            for child in &self.nodes[node.get()].kids {
                let count = self.nodes[child.get()].count;
                if index <= count {
                    node = child;
                    continue 'to_leaves;
                }

                index -= count;
            }
        }

        // node is a leaf node
        let mut at_leaf = true;
        let mut to_insert = to_insert;

        'to_root: for level in 0..self.levels {
            let update_parent = |sel: &mut Self, to_insert: Idx, parent: Idx| {
                if at_leaf {
                    sel.element_info[to_insert.get()].parent = parent;
                } else {
                    sel.nodes[to_insert.get()].parent = parent;
                }
            };

            let sum_children = |sel: &Self, kids: &ChildArray| {
                let init = Default::default();
                let iter = kids.into_iter();

                if at_leaf {
                    return iter.fold(init, |agg, node| agg + sel.elements[node.get()].get_info());
                }

                return iter.fold(init, |agg, node| agg + sel.nodes[node.get()].info);
            };

            let count_children = |sel: &Self, kids: &ChildArray| {
                if at_leaf {
                    return kids.into_iter().count();
                }

                let iter = kids.into_iter();
                return iter.fold(0, |agg, node| agg + sel.nodes[node.get()].count);
            };

            let mut right_node = Node::new();
            right_node.kids = match self.nodes[node.get()].kids.insert(index, to_insert) {
                None => {
                    update_parent(self, to_insert, node);
                    let node_ref = &mut self.nodes[node.get()];
                    node_ref.count += 1;
                    node_ref.info = node_ref.info + elem_agg_info;

                    node = node_ref.parent;
                    at_leaf = false;
                    continue 'to_root;
                }
                Some(right) => right,
            };

            let right_idx = Idx::new(self.nodes.len());
            for kid in &right_node.kids {
                update_parent(self, kid, node);
            }
            right_node.info = sum_children(self, &right_node.kids);
            right_node.count = count_children(self, &right_node.kids);

            let kids = self.nodes[node.get()].kids;
            let info = sum_children(self, &kids);
            let count = count_children(self, &kids);
            let node_ref = &mut self.nodes[node.get()];
            node_ref.info = info;
            node_ref.count = count;
            let parent = node_ref.parent;

            self.nodes.push(right_node);
            node = parent;
            at_leaf = false;
        }
    }

    fn _get_idx(&self, index: usize) -> Option<Idx> {
        let mut node = self.nodes[self.root];
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
        }

        let index = node.kids.into_iter().nth(running);
        return Some(index.unwrap());
    }

    fn _key_idx<F>(&self, index: usize, getter: F) -> Option<(Idx, usize)>
    where
        F: Fn(<T as BTreeItem>::BTreeInfo) -> usize,
    {
        let mut node = self.nodes[self.root];
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

// We're using the trick from Basic algo with the combining lists thing! From
// the 2-3 tree PSet. Very cute.
//                      - Albert Liu, Dec 06, 2021 Mon 19:11 EST
#[derive(Clone, Copy)]
struct Node<Info>
where
    Info: BTreeInfo,
{
    info: Info,
    count: usize,
    parent: Idx, // For the root, I dont think this matters. Maybe it'll point to itself.
    kids: ChildArray,
}

impl<Info> Node<Info>
where
    Info: BTreeInfo,
{
    fn new() -> Self {
        return Node {
            info: Default::default(),
            count: 0,
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
    fn new() -> Self {
        return Self { value: [None; B] };
    }

    fn insert(&mut self, mut index: usize, mut value: Idx) -> Option<Self> {
        if index >= B || self.value[index - 1].is_none() {
            panic!("index out of bounds");
        }

        while index < B {
            value = match self.value[index].replace(value) {
                None => return None,
                Some(old) => old,
            };

            index += 1;
        }

        let split_point = B / 2 + 1;
        let mut other = Self::new();
        let mut index = 0;
        for item in self.value[split_point..].iter().map(|c| c.unwrap()) {
            other.value[index] = Some(item);
            index += 1;
        }

        other.value[index] = Some(value);
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
