use std::collections::HashMap;
use std::io::Read;
use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream};
use std::pin::Pin;
use std::ptr::read;
use std::sync::Arc;
use std::thread::yield_now;
use std::time::Duration;
use async_stream::stream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::error::{SendError, TryRecvError, TrySendError};
use tokio_stream::{Stream, StreamExt, StreamMap};
use cute_core::{CuteError, DataStream};
use crate::raw::packet::{CutePacketTrait, CutePacketType, CutePacketValid};

pub use server::CuteRawServiceServer;
pub use client::CuteRawServiceClient;

#[async_trait::async_trait]
pub trait CuteRawService<P> : Send + Sync + 'static
where P : CutePacketTrait + Send
{
    async fn server_unary(&self,protocol : u32, input: Box<[u8]>) -> Result<Vec<u8>, CuteError>;
    async fn server_stream(&self,protocol : u32, input: Box<[u8]>) -> Result<Pin<Box<dyn tokio_stream::Stream<Item=Result<Vec<u8>, CuteError>> + Send>>, CuteError>;
    async fn server_stream_close(&self, protocol : u32) -> Result<(), CuteError>;
    async fn server_stream_all_close(&self) -> Result<(), CuteError>;
}

async fn cute_stream<P : CutePacketTrait>(mut tcp_stream : tokio::net::TcpStream,
                                          mut rx: tokio::sync::mpsc::Receiver<Result<Box<P>, CuteError>>) -> Pin<Box<dyn Stream<Item=Result<Box<P>, CuteError>> + Send>>
{
    Box::pin(stream! {
        let mut is_closed = false;
        let mut read_buf = [0u8; 4096];
        let mut store_buffer = vec![];
        let header_size = P::get_header_size();
        let drain_size = P::get_drain_size();
        while !is_closed {
            match tcp_stream.read(&mut read_buf[..]).await {
                Ok(0) => {
                    is_closed = true;
                }
                Ok(n) => {
                    store_buffer.extend_from_slice(&read_buf[..n]);
                    loop {
                        match P::is_valid(&store_buffer) {
                            CutePacketValid::ValidOK(payload_len) => {
                                let res = P::recv_create_packet(&store_buffer[0..payload_len]);
                                store_buffer.drain(0..payload_len);
                                yield Ok(res);
                            }
                            CutePacketValid::DataShort => {
                                break;
                            }
                            CutePacketValid::ValidFailed(_) => {
                                if drain_size > 0 {
                                    store_buffer.drain(0..drain_size);
                                } else {
                                    store_buffer.clear();
                                }
                                break;
                            }
                        }

                    }
                }
                Err(e) => {
                    is_closed = true;
                }
            }

            match rx.try_recv() {
                Ok(output) => {
                    match output {
                        Ok(packet) => {
                            for item in P::chuck_create_packet(packet.get_payload(),packet.get_packet_protocol(),packet.get_packet_type()) {
                                if let Err(_) = tcp_stream.write_all(&item.serialize()).await {
                                    is_closed = true;
                                }
                            }
                        }
                        Err(e) => {
                            if let Some(item) = P::error_create_packet(e) {
                                if let Err(_) = tcp_stream.write_all(&item.serialize()).await {
                                    is_closed = true;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    match e {
                        TryRecvError::Empty => {
                            continue;
                        }
                        TryRecvError::Disconnected => {
                            is_closed = true;
                        }
                    }
                }
            }
        }
        yield Err(CuteError::internal("Read Stream Closed"));
        rx.close();
    })
}

mod server;
mod client;