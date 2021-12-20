use crate::traits::*;
use crate::tree::*;
use crate::util::*;

impl<T> BTree<T>
where
    T: BTreeItem,
{
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
        return Some(ElemIdx(idx));
    }

    pub fn key_idx<F>(&self, index: usize, get: F) -> Option<(ElemIdx, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self.find(false, index, move |_, info| get(info))?;
        return Some((ElemIdx(idx), remainder));
    }

    pub fn key_leq_idx<F>(&self, index: usize, get: F) -> Option<(ElemIdx, usize)>
    where
        F: Fn(T::Info) -> usize,
    {
        let (idx, remainder) = self.find(true, index, move |_, info| get(info))?;
        return Some((ElemIdx(idx), remainder));
    }

    // We can't return a mutable reference here because we need to update the
    // bookkeeping data after the mutation finishes
    //
    // This could ALSO be generic over more things or whatever. I don't care.
    //                                  - Albert Liu, Dec 18, 2021 Sat 23:56 EST
    pub fn get_mut<E, F>(&mut self, index: impl BTreeIdx<T>, mut f: F) -> Option<E>
    where
        F: FnMut(&mut T) -> E,
    {
        let idx = index.get(self)?.get();

        let elem = &mut self.elements[idx];
        let result = f(elem);

        let mut node = self.element_parents[idx];
        self.update_node(node);
        for _ in 0..self.levels {
            node = self.nodes[node.get()].parent.unwrap();
            self.update_node(node);
        }

        debug_assert_eq!(node, self.root);

        return Some(result);
    }

    pub fn edit_or_remove<F>(&mut self, index: impl BTreeIdx<T>, mut f: F) -> Option<T>
    where
        F: FnMut(&mut T) -> bool,
    {
        let idx = index.get(self)?.get();

        let elem = &mut self.elements[idx];
        if f(elem) {
            return self.remove(ElemIdx(Idx::new(idx)));
        }

        let mut node = self.element_parents[idx];
        self.update_node(node);
        for _ in 0..self.levels {
            node = self.nodes[node.get()].parent.unwrap();
            self.update_node(node);
        }

        debug_assert_eq!(node, self.root);

        return None;
    }

    pub fn last_idx(&self) -> Option<ElemIdx> {
        let (idx, _) = self.find(true, self.len(), |count, _| count)?;

        return Some(ElemIdx(idx));
    }

    #[inline]
    pub fn add(&mut self, element: T) -> ElemIdx {
        return self.insert(self.nodes[self.root.get()].count, element);
    }

    pub fn count_until(&self, index: impl BTreeIdx<T>) -> Option<usize> {
        return self.sum_until(index, |c, _| c);
    }
}
