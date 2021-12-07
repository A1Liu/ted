use core::num::NonZeroUsize;

#[derive(Clone, Copy)]
pub struct Idx(NonZeroUsize);

impl Idx {
    pub fn new(i: usize) -> Idx {
        // this will panic anyways.
        return Idx(unsafe { NonZeroUsize::new_unchecked(i + 1) });
    }

    pub fn get(self) -> usize {
        return self.0.get() - 1;
    }
}
