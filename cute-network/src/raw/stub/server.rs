use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::error::TryRecvError;
use tokio_stream::{StreamExt, StreamMap};
use cute_core::CuteError;
use crate::raw::{CutePacketTrait, CutePacketType, CutePacketValid};
use crate::raw::stub::{CuteRawService};

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

    pub async fn start(&self) -> Result<(), CuteError> {
        let listener = tokio::net::TcpListener::bind(self.host_addr)
            .await.map_err(|e| CuteError::internal(e.to_string()))?;

        let (close_tx,mut close_rx) = tokio::sync::mpsc::channel(64);
        let (send_tx,mut send_rx) = tokio::sync::mpsc::channel(64);
        let peer_map: Arc<tokio::sync::Mutex<HashMap<SocketAddr, (tokio::net::TcpStream, Vec<u8>)>>> = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let stream_map : Arc<tokio::sync::Mutex<StreamMap<String, Pin<Box<dyn tokio_stream::Stream<Item=Result<Vec<u8>, CuteError>> + Send>>>>> = Arc::new(tokio::sync::Mutex::new(StreamMap::new()));

        //task stream execute
        tokio::spawn({
            let arc_send_tx = send_tx.clone();
            let arc_stream_map = stream_map.clone();
            async move {
                let mut delay = tokio::time::interval(Duration::from_micros(10));
                loop {
                    delay.tick().await;
                    if let Some((key_name, res)) = arc_stream_map.lock().await.next().await {
                        let parts: Vec<&str> = key_name.split('_').collect();
                        let socket_addr = SocketAddr::from_str(parts[0]).unwrap();
                        let protocol = parts[1].parse::<u32>().unwrap();

                        match res {
                            Ok(output) => {
                                let _ = arc_send_tx.send((socket_addr, P::send_create_packet(output, protocol, CutePacketType::Streaming))).await;
                            }
                            Err(err) => {
                                break;
                            }
                        }
                    }
                }
                drop(arc_send_tx);
                drop(arc_stream_map);
            }
        });

        // tcp_read & close
        tokio::spawn({
            let arc_peer_map = peer_map.clone();
            let arc_stream_map = stream_map.clone();
            let arc_service = self.inner.0.clone();
            let arc_send_tx = send_tx.clone();
            let arc_close_tx = close_tx.clone();
            async move {
                let mut delay = tokio::time::interval(Duration::from_micros(10));
                let drain_size = P::get_drain_size();
                let mut chuck_protocol_map : HashMap<SocketAddr,HashMap<u32, Vec<Box<P>>>> = HashMap::new();
                loop {
                    delay.tick().await;
                    for (remote_addr, (tcp_stream, store_buffer)) in arc_peer_map.lock().await.iter_mut() {
                        if !chuck_protocol_map.contains_key(remote_addr) {
                            let inner_hash_map = HashMap::new();
                            chuck_protocol_map.insert(*remote_addr, inner_hash_map);
                        }
                        let protocol_item = chuck_protocol_map.get_mut(remote_addr);
                        let mut read_buf = [0u8; 65536];

                        match tcp_stream.try_read(&mut read_buf) {
                            Ok(0) => {
                                let _ = arc_close_tx.send(*remote_addr).await;
                            }
                            Ok(n) => {
                                println!("Read {} bytes", n);
                                store_buffer.extend_from_slice(&read_buf[..n]);
                                match P::is_valid(&store_buffer) {
                                    CutePacketValid::ValidOK(payload_len) => {
                                        let packet = P::recv_create_packet(&store_buffer[0..payload_len]);
                                        store_buffer.drain(0..payload_len);

                                        let chuck_idx = packet.get_chuck_idx();
                                        let chuck_size = packet.get_chuck_size();
                                        let protocol = packet.get_packet_protocol();
                                        let protocol_type = packet.get_packet_type();
                                        let key_name = format!("{}_{}",remote_addr,protocol);

                                        if let Some(packet_hash_map) = protocol_item {
                                            if !packet_hash_map.contains_key(&protocol) {
                                                packet_hash_map.entry(protocol).or_insert_with(Vec::new).push(packet);
                                            } else {
                                                packet_hash_map.get_mut(&protocol).unwrap().push(packet);
                                            }

                                            if chuck_size == chuck_idx + 1 {
                                                if let Some(inner_packets) = packet_hash_map.remove(&protocol) {
                                                    let summation_payload: Vec<u8> = inner_packets.iter().
                                                        flat_map(|x| x.get_payload()).
                                                        collect();

                                                    match protocol_type {
                                                        CutePacketType::Empty => {}
                                                        CutePacketType::Unary => {
                                                            if let Ok(output) = arc_service.server_unary(protocol, summation_payload.into_boxed_slice()).await {
                                                                let _ = arc_send_tx.send((*remote_addr,P::send_create_packet(output,protocol,protocol_type))).await;
                                                            }
                                                        }
                                                        CutePacketType::Streaming => {
                                                            let mut lock_stream_map = arc_stream_map.lock().await;
                                                            if lock_stream_map.contains_key(&key_name) {
                                                                let _ = lock_stream_map.remove(&key_name.clone());
                                                            }
                                                            if let Ok(inner_stream) = arc_service.server_stream(protocol, summation_payload.into_boxed_slice()).await {
                                                                let _ = lock_stream_map.insert(key_name.clone(), inner_stream);
                                                            }
                                                            drop(lock_stream_map);
                                                        }
                                                        CutePacketType::StreamClose => {
                                                            let mut lock_stream_map = arc_stream_map.lock().await;
                                                            if lock_stream_map.contains_key(&key_name) {
                                                                let _ = lock_stream_map.remove(&key_name.clone());
                                                            }
                                                            drop(lock_stream_map);
                                                        }
                                                        CutePacketType::StreamAllClose => {
                                                            let remote_addr_str= remote_addr.to_string();

                                                            let mut lock_stream_map = arc_stream_map.lock().await;
                                                            let key_to_remove: Vec<String> = lock_stream_map.keys().filter(|key| key.contains(remote_addr_str.as_str())).cloned().collect();
                                                            for item in key_to_remove {
                                                                let _ = lock_stream_map.remove(&item);
                                                            }
                                                            drop(lock_stream_map);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                    }
                                    CutePacketValid::DataShort => {
                                    }
                                    CutePacketValid::ValidFailed(_) => {
                                        if drain_size > 0 {
                                            store_buffer.drain(0..drain_size);
                                        } else {
                                            store_buffer.clear();
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                if e.kind() != std::io::ErrorKind::WouldBlock {
                                    let _ = arc_close_tx.send(*remote_addr).await;
                                }
                            }
                        }
                    }

                    match close_rx.try_recv() {
                        Ok(remote_addr) => {
                            if let Some((mut tcp_stream,mut store_buffer)) = arc_peer_map.lock().await.remove(&remote_addr) {
                                store_buffer.clear();
                                let _ = tcp_stream.shutdown().await;
                            }
                        }
                        Err(e) => {
                            match e {
                                TryRecvError::Empty => {}
                                TryRecvError::Disconnected => {
                                    break;
                                }
                            }
                        }
                    }

                }
                drop(arc_close_tx);
                drop(arc_send_tx);
                drop(arc_service);
                drop(arc_peer_map);
                drop(arc_stream_map);
            }
        });

        // tcp_write
        tokio::spawn({
            let arc_close_tx = close_tx.clone();
            let arc_peer_map = peer_map.clone();
            async move {
                let mut delay = tokio::time::interval(Duration::from_micros(10));
                loop {
                    delay.tick().await;
                    match send_rx.recv().await {
                        None => {}
                        Some((remote_addr, res_packet)) => {
                            let mut lock_peer_map = arc_peer_map.lock().await;
                            if let Some((tcp_stream, _)) = lock_peer_map.get_mut(&remote_addr) {
                                for item in P::chuck_create_packet(res_packet.get_payload(), res_packet.get_packet_protocol(), res_packet.get_packet_type()) {
                                    match tcp_stream.write_all(&item.serialize()).await {
                                        Ok(_) => {}
                                        Err(_) => {
                                            let _ = arc_close_tx.send(remote_addr).await;
                                        }
                                    }
                                }
                            }
                            drop(lock_peer_map);
                        }
                    }
                }
                drop(arc_peer_map);
                drop(arc_close_tx);
            }
        });

        loop {
            let (tcp_stream, remote_addr) = listener.accept().await.map_err(|e| CuteError::internal(e.to_string()))?;
            tcp_stream.set_linger(None).expect("set_linger call failed");

            let mut lock_peer_map = peer_map.lock().await;
            lock_peer_map.entry(remote_addr).or_insert((tcp_stream, vec![]));
            drop(lock_peer_map);
        }

        Ok(())
    }
}