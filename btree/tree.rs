use crate::nodes::*;
use crate::traits::*;
use crate::util::*;
use alloc::vec;
use alloc::vec::Vec;

// Use SOA stuff eventually: https://github.com/lumol-org/soa-derive
// They even have a trait! Very cute.
pub struct BTree<T>
where
    T: BTreeItem,
{
    pub(crate) elements: Vec<T>,
    pub(crate) element_info: Vec<ElementInfo>,
    pub(crate) nodes: Vec<Node<T::Info>>,
    pub(crate) first_free: Option<Idx>,
    pub(crate) root: Idx,
    pub(crate) levels: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct ElementInfo {
    pub(crate) parent: Idx,
    pub(crate) next_free: Option<Idx>,
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

    pub fn find<F>(&self, inclusive: bool, mut key: usize, get: F) -> Option<(Idx, usize)>
    where
        F: Fn(usize, T::Info) -> usize,
    {
        let mut node = &self.nodes[self.root.get()];
        if key > get(node.count, node.info) {
            return None;
        }

        if !inclusive && key == get(node.count, node.info) {
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

    pub(crate) fn insert_into_leaf(&mut self, mut node: Idx, index: usize, elem: T) -> ElemIdx {
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

    pub(crate) fn allocate_elem(&mut self, parent: Idx, elem: T) -> Idx {
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

    pub(crate) fn add_child(
        &mut self,
        node: Idx,
        at: usize,
        child: Idx,
        info: T::Info,
    ) -> Option<Kids> {
        let kids = self.nodes[node.get()].kids.insert(at, child);
        if kids.is_none() {
            let node_ref = &mut self.nodes[node.get()];
            node_ref.info = node_ref.info.add(info);
            node_ref.count += 1;
        }

        return kids;
    }

    pub(crate) fn update_node(&mut self, is_leaf: bool, node: Idx) {
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
