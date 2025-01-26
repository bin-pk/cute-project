use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use async_stream::stream;
use tokio_stream::Stream;
use cute_core::{CuteError, Procedure, Task};
use crate::NetworkConfig;
use crate::raw::CutePacketTrait;
use crate::raw::packet::{CutePacket};
use crate::raw::stub::{CuteRawService, CuteRawServiceServer};

pub struct CuteRawServer<R, P, C, T>
where R : AsRef<P>,
      P : Procedure<C>,
      C : Send + Sync + 'static,
      T : CutePacketTrait + Send
{
    config: NetworkConfig,
    procedure: R,
    context: Arc<tokio::sync::RwLock<C>>,
    close_map : Arc<tokio::sync::Mutex<std::collections::HashMap<u32, tokio::sync::watch::Sender<bool>>>>,
    _phantom_p: PhantomData<fn() -> P>,
    _phantom_t : PhantomData<fn() -> T>,
}

impl <R, P, C, T> CuteRawServer<R, P, C, T>
where R : AsRef<P> + Send + Sync + 'static,
      P : Procedure<C> + Send + Sync + 'static,
      C : Clone + Send + Sync + 'static,
      T : CutePacketTrait + Send
{
    pub async fn start(procedure : R,
                       config : NetworkConfig,
                       ctx : Arc<tokio::sync::RwLock<C>>)-> Result<() , std::io::Error> {
        let server = CuteRawServer::<R, P, C, T> {
            config,
            procedure,
            context : ctx,
            close_map: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            _phantom_p: Default::default(),
            _phantom_t : Default::default(),
        };

        match CuteRawServiceServer::new(server, config.host_address).start().await {
            Ok(_) => {
                Ok(())
            }
            Err(_) => {
                Err(std::io::Error::new(std::io::ErrorKind::Other, ""))
            }
        }
    }
}

#[async_trait::async_trait]
impl<R, P, C, T> CuteRawService<T> for CuteRawServer<R, P, C, T>
where R : AsRef<P> + Send + Sync + 'static,
      P : Procedure<C> + Send + Sync + 'static,
      C : Clone + Send + Sync + 'static,
        T : CutePacketTrait + Send
{
    async fn server_unary(&self, protocol: u32, input: Box<[u8]>) -> Result<Vec<u8>, CuteError> {
        let proc_map = self.procedure.as_ref();

        match proc_map.get_task(protocol,Some(input)).await {
            Ok(mut task) => {
                let opt_output = task.execute(self.context.clone()).await?;
                match opt_output {
                    None => {
                        Ok(vec![])
                    }
                    Some(output) => {
                        Ok(output)
                    }
                }
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    async fn server_stream(&self, protocol: u32, input: Box<[u8]>) -> Result<Pin<Box<dyn Stream<Item=Result<Vec<u8>, CuteError>> + Send>>, CuteError> {
        let proc_map = self.procedure.as_ref();
        let (stop_signal,_) = tokio::sync::watch::channel(false);
        let stop_rx = stop_signal.subscribe();
        let mut lock_peer_map = self.close_map.lock().await;
        if let Some(sender) = lock_peer_map.get(&protocol) {
            let _ = sender.send(true).is_err();
        }
        lock_peer_map.entry(protocol).or_insert(stop_signal);
        drop(lock_peer_map);

        match proc_map.get_task(protocol,Some(input)).await {
            Ok(mut task) => {
                let ctx = self.context.clone();
                Ok(Box::pin(stream!{
                    let mut is_closed = false;
                    loop {
                        if *stop_rx.borrow() {
                            is_closed = true;
                        }
                        if is_closed {
                            break;
                        } else {
                            match task.execute(ctx.clone()).await {
                                Ok(opt_output) => {
                                    if let Some(output) = opt_output {
                                        yield Ok(output)
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                    }
                    yield Err(CuteError::internal("stream close"))
                }))
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    async fn server_stream_close(&self, protocol: u32) -> Result<(), CuteError> {
        let mut lock_close_map = self.close_map.lock().await;
        if let Some(sender) = lock_close_map.remove(&protocol) {
            let _ = sender.send(true).is_err();
        }
        drop(lock_close_map);

        Ok(())
    }

    async fn server_stream_all_close(&self) -> Result<(), CuteError> {
        let mut lock_close_map = self.close_map.lock().await;
        for (_, sender) in lock_close_map.drain() {
            let _ = sender.send(true).is_err();
        }
        drop(lock_close_map);

        Ok(())
    }
}