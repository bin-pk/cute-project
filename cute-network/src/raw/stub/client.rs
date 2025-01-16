use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::{Stream, StreamExt, StreamMap};
use cute_core::CuteError;
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
                let mut protocol_map: HashMap<u32, Vec<Option<Box<P>>>> = HashMap::new();
                let mut is_closed = false;

                while !is_closed {
                    if *stop_rx.borrow() {
                        is_closed = true;
                    } else {
                        if let Some(res_packet) = reader.next().await {
                            if let Ok(packet) = res_packet {
                                let chuck_idx = packet.get_chuck_idx();
                                let chuck_size = packet.get_chuck_size();
                                let protocol = packet.get_packet_protocol();
                                let protocol_type = packet.get_packet_type();

                                if chuck_size == 1 {
                                    match protocol_type {
                                        CutePacketType::Unary => {
                                            let mut lock_unary_map = arc_unary_map.lock().await;
                                            lock_unary_map.entry(protocol).or_insert(packet);
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
                                    // 없는 경우. 완전 새롭게 들어온 protocol 인 경우
                                    if !protocol_map.contains_key(&protocol) {
                                        protocol_map.insert(protocol, Vec::with_capacity(chuck_size));
                                    }
                                    protocol_map.get_mut(&protocol).unwrap()[chuck_size] = Some(packet);

                                    if chuck_size == chuck_idx + 1 {
                                        if let Some(inner_packets) = protocol_map.remove(&protocol) {
                                            if inner_packets.iter().all(|x| x.is_some()) {

                                                let summation_payload: Vec<u8> = inner_packets.iter().
                                                    filter_map(|x| x.as_ref()).
                                                    flat_map(|x| x.get_payload()).
                                                    collect();

                                                match protocol_type {
                                                    CutePacketType::Unary => {
                                                        let mut lock_unary_map = arc_unary_map.lock().await;
                                                        lock_unary_map.entry(protocol).or_insert(P::send_create_packet(summation_payload, protocol, protocol_type));
                                                        drop(lock_unary_map);
                                                    }
                                                    CutePacketType::Streaming => {
                                                        let mut lock_stream_map = arc_stream_map.lock().await;
                                                        if let Some(tx) = lock_stream_map.get_mut(&protocol) {
                                                            let _ = tx.send(P::send_create_packet(summation_payload,protocol,protocol_type));
                                                        }
                                                        drop(lock_stream_map);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
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
}