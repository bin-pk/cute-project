use std::marker::PhantomData;
use crate::{Task, TaskConstructor};

pub struct BasicTaskConstructor<T,C> {
    phantom_t : PhantomData<T>,
    phantom_c: PhantomData<fn() -> C>,
}

impl<T,C> BasicTaskConstructor<T,C> {
    pub fn new() -> Self {
        Self {
            phantom_t : PhantomData::default(),
            phantom_c: PhantomData::default(),
        }
    }
}

impl<T,C> TaskConstructor<T,C> for BasicTaskConstructor<T,C>
where T : Task<C> + Send + Sync,
      C : Send + Sync + 'static, {
}