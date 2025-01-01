use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{DataStream, Procedure, Worker};
use crate::errors::CuteError;

pub struct ProcManager<C> {
    worker_map : std::collections::HashMap<Box<str>, Box<dyn Worker<C> + Send + Sync + 'static>>,
    phantom : std::marker::PhantomData<fn() -> C>,
}

impl<C> ProcManager<C> {
    pub fn new() -> Self {
        Self {
            worker_map: std::collections::HashMap::new(),
            phantom: Default::default(),
        }
    }

    pub fn insert(&mut self,key : Box<str>, worker : Box<dyn Worker<C> + Send + Sync + 'static>) {
        self.worker_map.entry(key).or_insert(worker);
    }
}

#[async_trait::async_trait]
impl<C> Procedure<C> for ProcManager<C>
where C : Send + Sync + 'static
{
    async fn get_service_names(&self) -> Result<Vec<String>, CuteError> {
        let mut names = Vec::new();
        for (key, _) in self.worker_map.iter() {
            names.push(key.to_string());
        }
        Ok(names)
    }

    async fn one_of_run(&self, key: Box<str>, ctx: Arc<RwLock<C>>, input: Option<Box<[u8]>>) -> Result<Vec<u8>, CuteError> {
        if let Some(worker) = self.worker_map.get(&key) {
            worker.one_of_execute(ctx, input).await
        } else {
            Err(CuteError::not_found(format!("Worker {} not found", key)))
        }
    }

    async fn iter_run(&self, key: Box<str>, ctx: Arc<RwLock<C>>, input: Option<Box<[u8]>>) -> Result<DataStream<Vec<u8>>, CuteError> {
        if let Some(worker) = self.worker_map.get(&key) {
            worker.iter_execute(ctx, input).await
        } else {
            Err(CuteError::not_found(format!("Worker {} not found", key)))
        }
    }

    async fn iter_close(&self, key: Box<str>) -> Result<(), CuteError> {
        if let Some(worker) = self.worker_map.get(&key) {
            let _ = worker.iter_close().await;
            Ok(())
        } else {
            Err(CuteError::not_found(format!("Worker {} not found", key)))
        }
    }

    async fn iter_all_close(&self) {
        for (_, workers) in self.worker_map.iter() {
            let _ = workers.iter_close().await;
        }
    }
}