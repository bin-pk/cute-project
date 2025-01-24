use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use async_stream::stream;
use log::info;
use tonic::{Request, Response, Status};
use cute_core::Procedure;
use crate::grpc::convert_cute_error_to_status;
use crate::grpc::proto::cute::cute_service_server::{CuteService, CuteServiceServer};
use crate::grpc::proto::cute::{Empty, Input, Output, Protocols};
use crate::NetworkConfig;

/// Comment
/// `cute.proto` 를 통해 generate 된 CuteService 특성을 지정받아 제작하기 위한 Server Struct
#[derive(Debug, Clone)]
pub struct GRPCServer<R, P, C>
where R : AsRef<P>,
      P : Procedure<C>,
      C : Send + Sync + 'static,
{
    config : NetworkConfig,
    procedure: R,
    context : Arc<tokio::sync::RwLock<C>>,
    peer_map : Arc<tokio::sync::Mutex<std::collections::HashMap<Box<str>, tokio::sync::watch::Sender<bool>>>>,
    _phantom_p: PhantomData<fn() -> P>,
}

impl<R, P, C> GRPCServer<R, P, C>
where R : AsRef<P> + Send + Sync + 'static,
      P : Procedure<C> + Send + Sync + 'static,
      C : Clone + Send + Sync + 'static,
{
    pub async fn start(procedure : R, config : NetworkConfig, ctx : Arc<tokio::sync::RwLock<C>>) -> Result<() , std::io::Error> {
        let server = GRPCServer {
            config,
            procedure,
            context : ctx,
            peer_map : Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            _phantom_p: Default::default(),
        };
        tonic::transport::Server::builder()
            .http2_keepalive_timeout(Some(tokio::time::Duration::from_secs(config.keep_alive_time_out)))
            .timeout(std::time::Duration::from_secs(config.time_out))
            .add_service(CuteServiceServer::new(server))
            .serve_with_shutdown(config.host_address, async {
            tokio::signal::ctrl_c().await.unwrap();
        }).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }
}


#[async_trait::async_trait]
impl<R, P, C> CuteService for GRPCServer<R, P, C>
where R : AsRef<P> + Send + Sync + 'static,
      P : Procedure<C> + Send + Sync + 'static,
      C : Clone + Send + Sync + 'static,
{
    async fn get_services_name(&self, _request: Request<Empty>) -> Result<Response<Protocols>, Status> {
        let proc_map = self.procedure.as_ref();
        match proc_map.get_service_protocols().await {
            Ok(res) => {
                Ok(Response::new(Protocols {
                    protocol: res,
                }))
            }
            Err(e) => {
                Err(convert_cute_error_to_status(e))
            }
        }
    }
    type ServerUnaryStream = Pin<Box<dyn tokio_stream::Stream<Item = Result<Output, Status>> + Send>>;

    async fn server_unary(&self, mut request: Request<Input>) -> Result<Response<Self::ServerUnaryStream>, Status> {
        let proc_map = self.procedure.as_ref();
        let protocol = request.get_ref().protocol;

        match proc_map.get_task(protocol,
                                request.get_mut().data.take().map(Vec::into_boxed_slice)).await {
            Ok(mut task) => {
                let mut result = Vec::new();
                let opt_output = task.execute(self.context.clone()).await.map_err(|e| convert_cute_error_to_status(e))?;
                match opt_output {
                    None => {
                    }
                    Some(output) => {
                        let output_len = output.len();
                        let chuck_size = output_len / self.config.max_page_byte_size + (output_len % self.config.max_page_byte_size != 0) as usize;
                        for (chuck_idx, chuck_item) in output.chunks(self.config.max_page_byte_size).enumerate() {
                            let paged_output = Output {
                                protocol,
                                page_size: chuck_size as u32,
                                page_idx: chuck_idx as u32,
                                data: chuck_item.to_vec(),
                            };
                            result.push(Ok(paged_output));
                        }
                    }
                }
                Ok(Response::new(Box::pin(tokio_stream::iter(result))))

            }
            Err(e) => {
                Err(convert_cute_error_to_status(e))
            }
        }
    }

    type ServerStreamStream = Pin<Box<dyn tokio_stream::Stream<Item = Result<Output, Status>> + Send>>;

    async fn server_stream(&self, mut request: Request<Input>) -> Result<Response<Self::ServerStreamStream>, Status> {
        let (stop_signal,_) = tokio::sync::watch::channel(false);
        let protocol = request.get_ref().protocol;
        let remote_addr = request
            .remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let key_name = format!("{}_{}",remote_addr,protocol);

        info!("key : {}",key_name);

        let stop_rx = stop_signal.subscribe();
        let mut lock_peer_map = self.peer_map.lock().await;
        if let Some(sender) = lock_peer_map.get(&key_name.clone().into_boxed_str()) {
            let _ = sender.send(true).is_err();
        }
        lock_peer_map.entry(key_name.clone().into_boxed_str()).or_insert(stop_signal);
        drop(lock_peer_map);

        let proc_map = self.procedure.as_ref();
        match proc_map.get_task(protocol,
                                request.get_mut().data.take().map(Vec::into_boxed_slice)).await {
            Ok(mut task) => {
                let ctx = self.context.clone();
                let max_page_byte_size = self.config.max_page_byte_size;
                Ok(Response::new(Box::pin(stream! {
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
                                        let output_len = output.len();
                                        let chuck_size = output_len / max_page_byte_size + (output_len % max_page_byte_size != 0) as usize;
                                        for (chuck_idx, chuck_item) in output.chunks(max_page_byte_size).enumerate() {
                                            yield Ok( Output {
                                                protocol,
                                                page_size: chuck_size as u32,
                                                page_idx: chuck_idx as u32,
                                                data: chuck_item.to_vec(),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    yield Err(convert_cute_error_to_status(e));
                                }
                            }
                        }
                    }
                    info!("Server Stream stopped");
                })))
            }
            Err(e) => {
                Err(convert_cute_error_to_status(e))
            }
        }
    }

    async fn server_stream_close(&self, request: Request<Input>) -> Result<Response<Empty>, Status> {
        let protocol = request.get_ref().protocol;
        let remote_addr = request
            .remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let key_name = format!("{}_{}",remote_addr,protocol);

        let mut lock_peer_map = self.peer_map.lock().await;
        if let Some(sender) = lock_peer_map.remove(&key_name.clone().into_boxed_str()) {
            let _ = sender.send(true).is_err();
        }
        drop(lock_peer_map);

        Ok(Response::new(Empty {}))
    }

    async fn server_stream_all_close(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {

        let remote_addr = request
            .remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut lock_peer_map = self.peer_map.lock().await;

        let key_to_remove: Vec<Box<str>> = lock_peer_map.keys().filter(|key| key.contains(remote_addr.as_str())).cloned().collect();
        for item in key_to_remove {
            if let Some(sender) = lock_peer_map.remove(&item) {
                let _ = sender.send(true).is_err();
            }
        }
        drop(lock_peer_map);
        Ok(Response::new(Empty {}))
    }
}
