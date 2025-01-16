use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::Stream;
use cute_core::{CuteError, Procedure};
use crate::NetworkConfig;
use crate::raw::packet::CutePacket;
use crate::raw::stub::{CuteRawService, CuteRawServiceServer};

pub struct CuteRawServer<R, P, C>
where R : AsRef<P>,
      P : Procedure<C>,
      C : Send + Sync + 'static,
{
    config: NetworkConfig,
    procedure: R,
    context: Arc<tokio::sync::RwLock<C>>,
    _phantom_p: PhantomData<fn() -> P>,
}

impl <R, P, C> CuteRawServer<R, P, C>
where R : AsRef<P> + Send + Sync + 'static,
      P : Procedure<C> + Send + Sync + 'static,
      C : Default + Clone + Send + Sync + 'static,
{
    pub async fn start(procedure : R, config : NetworkConfig, ctx : Arc<tokio::sync::RwLock<C>>)-> Result<() , std::io::Error> {
        let server = CuteRawServer {
            config,
            procedure,
            context : ctx,
            _phantom_p: Default::default(),
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
impl<R, P, C> CuteRawService<CutePacket> for CuteRawServer<R, P, C>
where R : AsRef<P> + Send + Sync + 'static,
      P : Procedure<C> + Send + Sync + 'static,
      C : Clone + Send + Sync + 'static,
{
    async fn server_unary(&self, protocol: u32, input: Box<[u8]>) -> Result<Vec<u8>, CuteError> {
        todo!()
    }

    async fn server_stream(&self, protocol: u32, input: Box<[u8]>) -> Result<Pin<Box<dyn Stream<Item=Result<Vec<u8>, CuteError>> + Send>>, CuteError> {
        todo!()
    }

    async fn server_stream_close(&self, protocol: u32) -> Result<(), CuteError> {
        todo!()
    }

    async fn server_stream_all_close(&self) -> Result<(), CuteError> {
        todo!()
    }
}