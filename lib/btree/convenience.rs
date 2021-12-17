use crate::traits::*;
use crate::tree::*;

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

    pub fn last_idx(&self) -> Option<ElemIdx> {
        let (idx, _) = self.find(true, self.len(), |count, _| count)?;

        return Some(ElemIdx(idx));
    }

    #[inline]
    pub fn add(&mut self, element: T) -> ElemIdx {
        return self.insert(self.nodes[self.root.get()].count, element);
    }
}
