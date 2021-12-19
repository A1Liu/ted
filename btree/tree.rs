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
    pub(crate) element_parents: Vec<Idx>,
    pub(crate) nodes: Vec<Node<T::Info>>,
    pub(crate) root: Idx,
    pub(crate) levels: usize,
}

impl<T> BTree<T>
where
    T: BTreeItem,
{
    pub fn new() -> Self {
        let root_node = Node::empty(true);

        return Self {
            elements: Vec::new(),
            element_parents: Vec::new(),
            nodes: vec![root_node],
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

    pub fn sum_until<F>(&self, index: impl BTreeIdx<T>, get: F) -> Option<usize>
    where
        F: Fn(usize, T::Info) -> usize,
    {
        let elem_idx = index.get(self)?;
        let mut sum = get(0, <T::Info as Default>::default());

        let mut node = self.element_parents[elem_idx.get()];
        for kid in &self.nodes[node.get()].kids {
            if kid == elem_idx {
                break;
            }

            let kid_info = self.elements[kid.get()].get_info();
            sum += get(0, kid_info);
        }

        for _ in 0..self.levels {
            let parent = self.nodes[node.get()].parent.unwrap();

            for kid in &self.nodes[parent.get()].kids {
                if kid == node {
                    break;
                }
                let kid = self.nodes[kid.get()];
                sum += get(kid.count, kid.info);
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

    pub fn remove(&mut self, index: impl BTreeIdx<T>) -> Option<T>
    where
        T: core::fmt::Debug,
        T::Info: core::fmt::Debug,
    {
        let idx = index.get(self)?;

        let elem = self.elements.swap_remove(idx.get());
        let leaf = self.element_parents.swap_remove(idx.get());
        assert_eq!(self.elements.len(), self.element_parents.len());

        let mut new_size = self.nodes[leaf.get()].kids.remove_value(idx);

        // Update reference to swapped element
        let old_idx = Idx::new(self.elements.len());
        if old_idx != idx {
            let swapped_parent = self.element_parents[idx.get()];
            let mut kids = self.nodes[swapped_parent.get()].kids.iter_mut();
            let slot = kids.find(|i| **i == old_idx).unwrap();
            *slot = idx;
        }

        let mut node = leaf;
        for _ in 0..self.levels {
            self.update_node(node);
            let parent = match new_size {
                0 => self.remove_node(node).parent.unwrap(),
                _ => self.nodes[node.get()].parent.unwrap(),
            };

            node = parent;
        }
        self.update_node(node);

        debug_assert_eq!(self.nodes[node.get()].parent, None);

        let mut new_size = self.nodes[node.get()].kids.iter().count();
        while new_size == 1 && self.levels > 0 {
            let node_data = self.remove_node(node);
            assert!(node_data.parent.is_none());

            let new_root = node_data.kids[0];
            self.nodes[new_root.get()].parent = None;
            new_size = self.nodes[new_root.get()].kids.iter().count();
            node = new_root;
            self.levels -= 1;
        }

        self.root = node;

        // TODO do more cleanup of tree.

        return Some(elem);
    }

    pub(crate) fn remove_node(&mut self, node: Idx) -> Node<T::Info> {
        let parent = self.nodes[node.get()].parent;

        if let Some(parent) = parent {
            self.nodes[parent.get()].kids.remove_value(node);
        }

        // Fix up references to node that will be moved
        let move_idx = Idx::new(self.nodes.len() - 1);

        // the swapped node might be the root
        if let Some(swapped_parent) = self.nodes[move_idx.get()].parent {
            let mut kids = self.nodes[swapped_parent.get()].kids.iter_mut();
            let slot = kids.find(|i| **i == move_idx).unwrap();
            *slot = node;
        }

        let move_node = &self.nodes[move_idx.get()];
        let (kids, is_leaf) = (move_node.kids, move_node.is_leaf);

        if is_leaf {
            for kid in &kids {
                self.element_parents[kid.get()] = node;
            }
        } else {
            for kid in &kids {
                self.nodes[kid.get()].parent = Some(node);
            }
        }

        let mut node_data = self.nodes.swap_remove(node.get());
        if node_data.parent == Some(move_idx) {
            node_data.parent = Some(node);
        }

        return node_data;
    }

    // There's probably some kind of way to make this cute and work for any BTreeIdx,
    // but I couldn't figure it out in the 10 or so minutes I wasted trying.
    //                                  - Albert Liu, Dec 18, 2021 Sat 23:49 EST
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
        let leaf = self.element_parents[index.0.get()];
        let mut kids_iter = self.nodes[leaf.get()].kids.into_iter();
        let index = kids_iter.position(|kid| kid == index.0).unwrap();

        return self.insert_into_leaf(leaf, index, elem);
    }

    pub fn insert_after(&mut self, index: ElemIdx, elem: T) -> ElemIdx {
        let leaf = self.element_parents[index.0.get()];
        let mut kids_iter = self.nodes[leaf.get()].kids.into_iter();
        let index = kids_iter.position(|kid| kid == index.0).unwrap();

        return self.insert_into_leaf(leaf, index + 1, elem);
    }

    pub(crate) fn insert_into_leaf(&mut self, mut node: Idx, index: usize, elem: T) -> ElemIdx {
        let info = elem.get_info();
        let elem = self.allocate_elem(node, elem);

        let mut right = self.add_child(node, index, elem, info).map(|kids| {
            self.update_node(node);
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
                self.update_node(parent);
                self.new_node(false, kids)
            });

            node = parent;
        }

        right.map(|right| {
            self.root = self.new_node(false, [node, right]);
            self.levels += 1;
        });

        return ElemIdx(elem);
    }

    pub(crate) fn allocate_elem(&mut self, parent: Idx, elem: T) -> Idx {
        self.nodes[parent.get()].assert_is_leaf();

        let idx = self.elements.len();
        self.elements.push(elem);
        self.element_parents.push(parent);

        return Idx::new(idx);
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

    pub(crate) fn update_node(&mut self, node: Idx) {
        let node_ref = &self.nodes[node.get()];
        let (kids, is_leaf) = (node_ref.kids, node_ref.is_leaf);
        let (mut count, mut info) = (0, T::Info::default());
        for kid in &kids {
            let (kid_info, kid_count) = if is_leaf {
                debug_assert!(self.element_parents[kid.get()] == node);
                (self.elements[kid.get()].get_info(), 1)
            } else {
                let kid = &self.nodes[kid.get()];
                debug_assert!(kid.parent == Some(node));
                (kid.info, kid.count)
            };

            info = info.add(kid_info);
            count += kid_count;
        }

        let node_ = &mut self.nodes[node.get()];
        node_.info = info;
        node_.count = count;
    }

    pub(crate) fn new_node(&mut self, is_leaf: bool, kids: impl Into<Kids>) -> Idx {
        let kids: Kids = kids.into();
        let idx = Idx::new(self.nodes.len());
        let (mut count, mut info) = (0, T::Info::default());
        for kid in &kids {
            let (kid_info, kid_count) = if is_leaf {
                self.element_parents[kid.get()] = idx;
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
