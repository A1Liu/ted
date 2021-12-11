use core::num::NonZeroUsize;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Idx(NonZeroUsize);

impl std::fmt::Debug for Idx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.0);
    }
}

impl Idx {
    pub fn new(i: usize) -> Idx {
        // this will panic anyways.
        return Idx(unsafe { NonZeroUsize::new_unchecked(i + 1) });
    }

    pub fn get(self) -> usize {
        return self.0.get() - 1;
    }
}
