use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::{Stream, StreamExt, StreamMap};
use cute_core::{CuteError, DataStream};
use crate::raw::packet::{CutePacketTrait, CutePacketType};
use crate::raw::stub::cute_stream;

#[derive(Debug)]
pub struct CuteRawServiceClient<P : CutePacketTrait> {
    stop_signal : tokio::sync::watch::Sender<bool>,
    send_tx : tokio::sync::mpsc::Sender<Result<Box<P>,CuteError>>,
    unary_map : Arc<tokio::sync::Mutex<std::collections::HashMap<u32, Box<P>>>>,
    stream_map : Arc<tokio::sync::Mutex<std::collections::HashMap<u32, tokio::sync::mpsc::Sender<Box<P>>>>>,
    _phantom_p: PhantomData<fn() -> P>
}

impl<P : CutePacketTrait> Drop for crate::raw::stub::CuteRawServiceClient<P> {
    fn drop(&mut self) {
        self.stop_signal.send(true).expect("stop signal receiver dropped");
    }
}

impl<P : CutePacketTrait> crate::raw::stub::CuteRawServiceClient<P>  {
    pub async fn connect(host_addr : SocketAddr) -> Result<Self,CuteError> {
        let mut tcp_stream = tokio::net::TcpStream::connect(host_addr).await.map_err(|e| CuteError::internal(format!("{:?}", e)))?;
        let (send_tx, rx) = tokio::sync::mpsc::channel(64);
        let (stop_signal,_) = tokio::sync::watch::channel(false);
        let stop_rx = stop_signal.subscribe();
        let mut unary_map = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        let mut stream_map = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

        tokio::spawn({
            let arc_unary_map = unary_map.clone();
            let arc_stream_map = stream_map.clone();
            async move {
                let mut reader = cute_stream::<P>(tcp_stream,rx).await;
                let mut is_closed = false;

                while !is_closed {
                    if *stop_rx.borrow() {
                        is_closed = true;
                    } else {
                        if let Some(res_packet) = reader.next().await {
                            if let Ok(packet) = res_packet {
                                let protocol = packet.get_packet_protocol();
                                let protocol_type = packet.get_packet_type();

                                match protocol_type {
                                    CutePacketType::Unary => {
                                        let mut lock_unary_map = arc_unary_map.lock().await;
                                        if !lock_unary_map.contains_key(&protocol_type) {
                                            let _ = lock_unary_map.insert(protocol_type, packet);
                                        }
                                        drop(lock_unary_map);
                                    }
                                    CutePacketType::Streaming => {
                                        let mut lock_stream_map = arc_stream_map.lock().await;
                                        if let Some(tx) = lock_stream_map.get_mut(&protocol) {
                                            let _ = tx.send(packet).await;
                                        }
                                        drop(lock_stream_map);
                                    }
                                    _ => {}
                                }
                            } else {
                                is_closed = true;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            stop_signal,
            send_tx : send_tx.clone() ,
            unary_map,
            stream_map,
            _phantom_p: Default::default(),
        })
    }

    pub async fn client_unary(&self, protocol : u32, parameter : Option<Vec<u8>>) -> Result<Box<P>,CuteError> {
        match parameter {
            Some(input) => {
                let _ = self.send_tx.send(Ok(P::send_create_packet(input,protocol,CutePacketType::Unary))).await.
                    map_err(|e| CuteError::internal(format!("{:?}", e)))?;
            }
            None => {
                let _ = self.send_tx.send(Ok(P::send_create_packet(vec![0,0,0,0],protocol,CutePacketType::Unary))).await.
                    map_err(|e| CuteError::internal(format!("{:?}", e)))?;
            }
        }

        loop {
            let mut lock_unary_map = self.unary_map.lock().await;
            if lock_unary_map.contains_key(&protocol) {
                return Ok(lock_unary_map.remove(&protocol).unwrap());
            }
            drop(lock_unary_map);
        }
    }

    pub async fn client_stream(&self, protocol : u32, parameter : Option<Vec<u8>>) -> Result<DataStream<Box<P>>, CuteError> {
        let lock_stream_map = self.stream_map.lock().await;
        if lock_stream_map.contains_key(&protocol) {
            return Err(CuteError::internal("terminate and run that stream first!!!"));
        }

        let (tx,mut rx) = tokio::sync::mpsc::channel(64);
        match parameter {
            Some(input) => {
                let _ = self.send_tx.send(Ok(P::send_create_packet(input,protocol,CutePacketType::Streaming))).await.
                    map_err(|e| CuteError::internal(format!("{:?}", e)))?;
            }
            None => {
                let _ = self.send_tx.send(Ok(P::send_create_packet(vec![0,0,0,0],protocol,CutePacketType::Streaming))).await.
                    map_err(|e| CuteError::internal(format!("{:?}", e)))?;
            }
        }

        let mut lock_stream_map = self.stream_map.lock().await;
        lock_stream_map.entry(protocol).or_insert(tx);
        drop(lock_stream_map);

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}