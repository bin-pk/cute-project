use std::io::Error;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use std::marker::PhantomData;
use async_stream::stream;
use crate::{bin_serialize, DataStream, Task, Worker};

pub struct BasicWorker<C,T> {
    stop_signal : watch::Sender<bool>,
    _phantom_t: PhantomData<T>,
    _phantom_c: PhantomData<fn() -> C>,
}

impl<C,T> BasicWorker<C,T>
where T : Task<C> + Sync + Sync,
      C : Send + Sync + 'static,
{
    pub fn new() -> Self {
        let (stop_signal, _) = watch::channel(false);
        Self {
            _phantom_t: Default::default(),
            _phantom_c: Default::default(),
            stop_signal,
        }
    }
}

#[async_trait::async_trait]
impl<C,T> Worker<C> for BasicWorker<C,T>
    where T : Task<C> + Sync + Sync + 'static,
        C : Send + Sync + 'static,
{
    type Output = T;

    async fn one_of_execute(&self, ctx: Arc<RwLock<C>>, input: Option<Box<[u8]>>) -> Result<Vec<u8>, Error> {
        let mut task = T::new(input)?;
        let output = task.execute(ctx).await?;
        task.destroy().await;
        bin_serialize(&output)
    }

    async fn iter_execute(&self, ctx: Arc<RwLock<C>>, input: Option<Box<[u8]>>) -> Result<DataStream<Vec<u8>>, Error> {
        let mut task = T::new(input)?;
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
                            match e.kind() {
                                std::io::ErrorKind::WouldBlock => {
                                    continue;
                                }
                                _ => {
                                    is_closed = true;
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
