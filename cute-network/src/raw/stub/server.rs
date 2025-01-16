use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use cute_core::CuteError;
use crate::raw::packet::{CutePacketTrait, CutePacketType};
use crate::raw::stub::{cute_stream, CuteRawService};

struct _Inner<T>(Arc<T>);

pub struct CuteRawServiceServer<P,T : CuteRawService<P>>
where P : CutePacketTrait
{
    inner : _Inner<T>,
    host_addr : SocketAddr,
    timeout : Option<Duration>,
    _phantom_p: PhantomData<fn() -> P>
}

impl<P,T : CuteRawService<P>> CuteRawServiceServer<P,T>
where P : CutePacketTrait
{
    pub fn new(inner: T, host_addr: SocketAddr) -> Self {
        Self::from_arc(Arc::new(inner), host_addr)
    }

    pub fn from_arc(inner: Arc<T>,host_addr: SocketAddr) -> Self {
        let inner = _Inner(inner);
        Self {
            inner,
            host_addr,
            timeout: None,
            _phantom_p: Default::default(),
        }
    }

    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout = Some(duration);
    }

    pub async fn start(&self) -> Result<(), CuteError> {
        let mut listener = tokio::net::TcpListener::bind(self.host_addr)
            .await.map_err(|e| CuteError::internal(e.to_string()))?;

        loop {
            let (mut tcp_stream, remote_addr) = listener.accept().await.map_err(|e| CuteError::internal(e.to_string()))?;
            tcp_stream.set_linger(None).expect("set_linger call failed");

            let (send_tx, rx) = tokio::sync::mpsc::channel(64);
            let (stop_signal,_) = tokio::sync::watch::channel(false);
            let stop_rx = stop_signal.subscribe();
            let mut stream_map = Arc::new(tokio::sync::Mutex::new(tokio_stream::StreamMap::new()));

            tokio::spawn({
                let arc_tx = send_tx.clone();
                let arc_stream_map = stream_map.clone();
                let arc_service = self.inner.0.clone();

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
                                                match arc_service.server_unary(protocol, packet.get_payload().into_boxed_slice()).await {
                                                    Ok(output) => {
                                                        if let Err(_) = arc_tx.send(Ok(P::send_create_packet(output,protocol,protocol_type))).await {
                                                            is_closed = true;
                                                        }
                                                    }
                                                    Err(err) => {
                                                        if let Err(_) = arc_tx.send(Err(err)).await {
                                                            is_closed = true;
                                                        }
                                                    }
                                                }
                                            }
                                            CutePacketType::Streaming => {
                                                let mut lock_stream_map = arc_stream_map.lock().await;
                                                if lock_stream_map.contains_key(&protocol) {
                                                    let _ = lock_stream_map.remove(&protocol);
                                                }
                                                if let Ok(inner_stream) = arc_service.server_stream(protocol, packet.get_payload().into_boxed_slice()).await {
                                                    let _ = lock_stream_map.insert(protocol, inner_stream);
                                                }
                                                drop(lock_stream_map);
                                            }
                                            CutePacketType::StreamClose => {
                                                let _ = arc_service.server_stream_close(protocol).await;
                                                let mut lock_stream_map = arc_stream_map.lock().await;
                                                let _ = lock_stream_map.remove(&protocol);
                                                drop(lock_stream_map);
                                            }
                                            CutePacketType::StreamAllClose => {
                                                let _ = arc_service.server_stream_all_close().await;
                                                let mut lock_stream_map = arc_stream_map.lock().await;
                                                lock_stream_map.clear();
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
                                                            match arc_service.server_unary(protocol, summation_payload.into_boxed_slice()).await {
                                                                Ok(output) => {
                                                                    if let Err(_) = arc_tx.send(Ok(P::send_create_packet(output,protocol,protocol_type))).await {
                                                                        is_closed = true;
                                                                    }
                                                                }
                                                                Err(err) => {
                                                                    if let Err(_) = arc_tx.send(Err(err)).await {
                                                                        is_closed = true;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        CutePacketType::Streaming => {
                                                            let mut lock_stream_map = arc_stream_map.lock().await;
                                                            if lock_stream_map.contains_key(&protocol) {
                                                                let _ = lock_stream_map.remove(&protocol);
                                                            }
                                                            if let Ok(inner_stream) = arc_service.server_stream(protocol, summation_payload.into_boxed_slice()).await {
                                                                let _ = lock_stream_map.insert(protocol, inner_stream);
                                                            }
                                                            drop(lock_stream_map);
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {}
                            } else {
                                is_closed = true;
                            }
                        }
                    }
                    drop(arc_stream_map);
                    drop(arc_service);
                    drop(arc_tx);
                    stop_signal.send(true).expect("stop signal");
                }
            });

            tokio::spawn({
                let arc_tx = send_tx.clone();
                let arc_stream_map = stream_map.clone();
                async move {
                    let mut is_closed = false;
                    while !is_closed {
                        if *stop_rx.borrow() {
                            is_closed = true;
                        } else {
                            if let Some((protocol, res)) = arc_stream_map.lock().await.next().await {
                                match res {
                                    Ok(output) => {
                                        if let Err(e) = arc_tx.send(Ok(P::send_create_packet(output,protocol,CutePacketType::Streaming))).await {
                                            is_closed = true;
                                        }
                                    }
                                    Err(err) => {
                                        if let Err(e) = arc_tx.send(Err(err)).await {
                                            is_closed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    drop(arc_stream_map);
                    drop(arc_tx);
                    stop_signal.send(true).expect("stop signal");
                }
            });
        }

        Ok(())
    }
}