use std::ops::Deref;

mod generated;
mod input;
mod output;
mod tasks;

pub use tasks::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct EmbeddedContext {
    pub(crate) test : i32,
}