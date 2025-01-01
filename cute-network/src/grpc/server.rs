use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};
use cute_core::{Procedure};
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
    _phantom_p: PhantomData<fn() -> P>,
}

impl<R, P, C> GRPCServer<R, P, C>
where R : AsRef<P> + Send + Sync + 'static,
      P : Procedure<C> + Send + Sync + 'static,
      C : Default + Clone + Send + Sync + 'static,
{
    pub async fn start(procedure : R, config : NetworkConfig) -> Result<() , std::io::Error> {
        let ctx = C::default();
        let server = GRPCServer {
            config,
            procedure,
            context : Arc::new(tokio::sync::RwLock::new(ctx)),
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
        match proc_map.get_service_names().await {
            Ok(res) => {
                Ok(Response::new(Protocols {
                    name: res,
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
        let name = request.get_ref().name.clone();
        match proc_map.one_of_run(name.clone().into_boxed_str(),
                                  self.context.clone(), request.get_mut().data.take().map(Vec::into_boxed_slice)).await {
            Ok(output) => {
                let output_len = output.len();
                let chuck_size = output_len / self.config.max_page_byte_size + (output_len % self.config.max_page_byte_size != 0) as usize;
                let mut result = Vec::new();
                for (chuck_idx, chuck_item) in output.chunks(self.config.max_page_byte_size).enumerate() {
                    let paged_output = Output {
                        name : name.clone(),
                        page_size: chuck_size as u32,
                        page_idx: chuck_idx as u32,
                        data: chuck_item.to_vec(),
                    };
                    result.push(Ok(paged_output));
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
        let proc_map = self.procedure.as_ref();
        let name = request.get_ref().name.clone();
        match proc_map.iter_run(name.clone().into_boxed_str(),
                                self.context.clone(), request.get_mut().data.take().map(Vec::into_boxed_slice)).await {
            Ok(mut stream) => {
                let max_page_byte_size = self.config.max_page_byte_size;
                let time_out = tokio::time::Duration::from_secs(self.config.time_out);
                let (tx,rx) = tokio::sync::mpsc::channel(self.config.max_channel_size);

                tokio::spawn(async move {
                    while let Some(res_output) = stream.next().await {
                        match res_output {
                            Ok(output) => {
                                let output_len = output.len();
                                let chuck_size = output_len / max_page_byte_size + (output_len % max_page_byte_size != 0) as usize;
                                for (chuck_idx, chuck_item) in output.chunks(max_page_byte_size).enumerate() {
                                    let paged_output = Output {
                                        name: name.clone(),
                                        page_size: chuck_size as u32,
                                        page_idx: chuck_idx as u32,
                                        data: chuck_item.to_vec(),
                                    };
                                    if let Err(_) = tokio::time::timeout(time_out, tx.send(Ok(paged_output))).await {
                                        println!("Timed out while sending output");
                                    }
                                }
                            }
                            Err(e) => {
                                if let Err(_) = tokio::time::timeout(time_out, tx.send(Err(convert_cute_error_to_status(e)))).await {
                                    println!("Timed out while sending output");
                                }
                            }
                        }
                    }
                    drop(tx);
                });
                Ok(Response::new(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))))
            }
            Err(e) => {
                Err(convert_cute_error_to_status(e))
            }
        }
    }

    async fn server_stream_close(&self, request: Request<Input>) -> Result<Response<Empty>, Status> {
        let proc_map = self.procedure.as_ref();
        match proc_map.iter_close(request.get_ref().name.clone().into_boxed_str()).await {
            Ok(_) => {
                Ok(Response::new(Empty {}))
            }
            Err(e) => {
                Err(convert_cute_error_to_status(e))
            }
        }
    }

    async fn server_stream_all_close(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let proc_map = self.procedure.as_ref();
        let _ = proc_map.iter_all_close().await;
        Ok(Response::new(Empty {}))
    }
}