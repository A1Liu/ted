use crate::tree::*;
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

pub trait BTreeIdx<T>
where
    T: BTreeItem,
{
    fn get(self, tree: &BTree<T>) -> Option<Idx>;
}

#[derive(Clone, Copy)]
pub struct ElemIdx(pub(crate) Idx);

pub(crate) fn e_idx(i: Idx) -> ElemIdx {
    return ElemIdx(i);
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
    fn get(self, _tree: &BTree<T>) -> Option<Idx> {
        return Some(self.0);
    }
}
