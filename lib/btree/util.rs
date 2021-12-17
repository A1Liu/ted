use core::num::NonZeroUsize;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Idx(NonZeroUsize);

impl Into<usize> for Idx {
    fn into(self) -> usize {
        return self.get();
    }
}

impl core::fmt::Debug for Idx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        return write!(f, "{}", self.0);
    }
}

impl Idx {
    #[inline(always)]
    pub fn new(i: usize) -> Idx {
        // this will panic anyways later on in the pipeline
        return Idx(unsafe { NonZeroUsize::new_unchecked(i + 1) });
    }

    #[inline(always)]
    pub fn get(self) -> usize {
        return self.0.get() - 1;
    }
}
