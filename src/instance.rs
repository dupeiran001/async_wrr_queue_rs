use crate::consts;
use std::num::NonZeroUsize;
use std::ops::Deref;

#[derive(PartialEq, Eq, Debug)]
pub struct Instance<T: PartialEq> {
    data: T,
    weight: NonZeroUsize,
}

impl<T: PartialEq> Instance<T> {
    pub fn new(data: T) -> Self {
        Instance {
            data,
            weight: consts::DEFAULT_WEIGHT,
        }
    }

    pub fn new_with_weight(data: T, weight: NonZeroUsize) -> Self {
        Instance { data, weight }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn weight(&self) -> &NonZeroUsize {
        &self.weight
    }
}

impl<T: PartialEq, U: Into<usize>> From<(T, U)> for Instance<T> {
    fn from(value: (T, U)) -> Self {
        Instance {
            data: value.0,
            weight: NonZeroUsize::new(value.1.into()).unwrap(),
        }
    }
}

impl<T: PartialEq> Deref for Instance<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
