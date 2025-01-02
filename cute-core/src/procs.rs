use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{DataStream, Procedure, Task, TaskConstructor};
use crate::errors::CuteError;

pub struct ProcManager<C> {
    constructor_map : std::collections::HashMap<Box<str>, Box<dyn TaskConstructor<C> + Send + Sync>>,
    _phantom_c : std::marker::PhantomData<fn() -> C>,
}

impl<C> ProcManager<C> {
    pub fn new() -> Self {
        Self {
            constructor_map: std::collections::HashMap::new(),
            _phantom_c: Default::default(),
        }
    }

    pub fn insert(&mut self,key : Box<str>, task_constructor : Box<dyn TaskConstructor<C> + Send + Sync + 'static>) {
        self.constructor_map.entry(key).or_insert(task_constructor);
    }
}

#[async_trait::async_trait]
impl<C> Procedure<C> for ProcManager<C>
where C : Send + Sync + 'static
{
    async fn get_service_names(&self) -> Result<Vec<String>, CuteError> {
        todo!()
    }

    async fn get_task(&self, key: Box<str>, input: Option<Box<[u8]>>) -> Result<Box<dyn Task<C> + Send>, CuteError> {
        match self.constructor_map.get(&key) {
            None => {
                Err(CuteError::not_found(format!("Task \"{}\" not found", key)))
            }
            Some(output) => {
                output.create(input)
            }
        }
    }
}