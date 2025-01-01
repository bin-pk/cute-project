use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use std::marker::PhantomData;
use async_stream::stream;
use crate::{bin_serialize, DataStream, Task, TaskConstructor, Worker};
use crate::errors::{CuteError, CuteErrorCode};

pub struct BasicWorker<TC,T,C> {
    constructor : TC,
    stop_signal : watch::Sender<bool>,
    _phantom_t: PhantomData<T>,
    _phantom_c: PhantomData<fn() -> C>,
}

impl<TC,T,C> BasicWorker<TC,T,C>
where TC : TaskConstructor<T,C>,
    T : Task<C> + Send + Sync,
    C : Send + Sync + 'static,
{
    pub fn new(constructor : TC) -> Self {
        let (stop_signal, _) = watch::channel(false);
        Self {
            constructor,
            stop_signal,
            _phantom_t: Default::default(),
            _phantom_c: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl<TC,T,C> Worker<C> for BasicWorker<TC,T,C>
where TC : TaskConstructor<T,C> + Send + Sync,
      T : Task<C> + Send + Sync + 'static,
      C : Send + Sync + 'static,
{
    async fn one_of_execute(&self, ctx: Arc<RwLock<C>>, input: Option<Box<[u8]>>) -> Result<Vec<u8>, CuteError> {
        let mut task = self.constructor.create(input)?;
        let output = task.execute(ctx).await?;
        task.destroy().await;
        bin_serialize(&output)
    }

    async fn iter_execute(&self, ctx: Arc<RwLock<C>>, input: Option<Box<[u8]>>) -> Result<DataStream<Vec<u8>>, CuteError> {
        let mut task = self.constructor.create(input)?;
        let stop_rx = self.stop_signal.subscribe();
        Ok(Box::pin(stream! {
            let mut is_closed = false;
            loop {
                if *stop_rx.borrow() {
                    is_closed = true;
                }
                if is_closed {
                    break;
                } else {
                    match task.execute(ctx.clone()).await {
                        Ok(ser) => {
                            match bin_serialize(&ser) {
                                Ok(output) => {
                                    yield Ok(output);
                                }
                                Err(e) => {
                                    yield Err(e);
                                }
                            }
                        }
                        Err(e) => {
                            match e.code {
                                CuteErrorCode::Ok => {
                                    continue;
                                }
                                _ => {
                                    break
                                }
                            }
                        }
                    }
                }
            }
        }))
    }

    async fn iter_close(&self) {
        let _ = self.stop_signal.send(true);
    }
}
