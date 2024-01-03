use std::num::NonZeroUsize;

pub const DEFAULT_WEIGHT: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(20usize) };
