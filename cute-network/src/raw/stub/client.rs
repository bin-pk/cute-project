use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::error::TryRecvError;
use cute_core::{CuteError, DataStream};
use crate::raw::packet::{CutePacketTrait, CutePacketType, CutePacketValid};

#[derive(Debug)]
pub struct CuteRawServiceClient<P : CutePacketTrait> {
    stop_signal : tokio::sync::watch::Sender<bool>,
    send_tx : tokio::sync::mpsc::Sender<Result<Box<P>,CuteError>>,
    unary_map : Arc<tokio::sync::Mutex<std::collections::HashMap<u32, Box<P>>>>,
    stream_map : Arc<tokio::sync::Mutex<std::collections::HashMap<u32, tokio::sync::mpsc::Sender<Result<Box<P>,CuteError>>>>>,
    _phantom_p: PhantomData<fn() -> P>
}

impl<P : CutePacketTrait> Drop for crate::raw::stub::CuteRawServiceClient<P> {
    fn drop(&mut self) {
        self.stop_signal.send(true).expect("stop signal receiver dropped");
    }
}

impl<P : CutePacketTrait> crate::raw::stub::CuteRawServiceClient<P>  {
    pub async fn connect(host_addr : SocketAddr) -> Result<Self,CuteError> {
        let (send_tx, mut rx) = tokio::sync::mpsc::channel::<Result<Box<P>, CuteError>>(64);
        let (stop_signal,_) = tokio::sync::watch::channel(false);
        let stop_rx = stop_signal.subscribe();
        let unary_map : Arc<tokio::sync::Mutex<std::collections::HashMap<u32, Box<P>>>>  = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        let stream_map : Arc<tokio::sync::Mutex<std::collections::HashMap<u32, tokio::sync::mpsc::Sender<Result<Box<P>,CuteError>>>>> = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        let mut tcp_stream = tokio::net::TcpStream::connect(host_addr).await.map_err(|e| CuteError::internal(format!("{:?}", e)))?;

        tokio::spawn({
            let arc_unary_map = unary_map.clone();
            let arc_stream_map = stream_map.clone();
            async move {
                let drain_size = P::get_drain_size();

                let mut read_buf = [0u8; 4096];
                let mut store_buffer = vec![];
                let mut delay = tokio::time::interval(Duration::from_micros(10));
                loop {
                    delay.tick().await;
                    if *stop_rx.borrow() {
                        break;
                    } else {
                        match tcp_stream.try_read(&mut read_buf) {
                            Ok(0) => { continue; }
                            Ok(n) => {
                                store_buffer.extend_from_slice(&read_buf[..n]);
                                match P::is_valid(&store_buffer) {
                                    CutePacketValid::ValidOK(payload_len) => {
                                        let packet = P::recv_create_packet(&store_buffer[0..payload_len]);
                                        store_buffer.drain(0..payload_len);

                                        let protocol = packet.get_packet_protocol();
                                        let protocol_type = packet.get_packet_type();

                                        match protocol_type {
                                            CutePacketType::Unary => {
                                                let mut lock_unary_map = arc_unary_map.lock().await;
                                                if !lock_unary_map.contains_key(&protocol) {
                                                    let _ = lock_unary_map.insert(protocol, packet);
                                                }
                                                drop(lock_unary_map);
                                            }
                                            CutePacketType::Streaming => {
                                                let mut lock_stream_map = arc_stream_map.lock().await;
                                                if let Some(tx) = lock_stream_map.get_mut(&protocol) {
                                                    if let Err(e) = tx.send(Ok(packet)).await {
                                                        println!("error sending stream: {}", e);
                                                    }
                                                }
                                                drop(lock_stream_map);
                                            }
                                            CutePacketType::StreamClose => {
                                                let mut lock_stream_map = arc_stream_map.lock().await;
                                                let _ = lock_stream_map.remove(&protocol);
                                                drop(lock_stream_map);
                                            }
                                            CutePacketType::StreamAllClose => {
                                                let mut lock_stream_map = arc_stream_map.lock().await;
                                                let _ = lock_stream_map.clear();
                                                drop(lock_stream_map);
                                            }
                                            _ => {}
                                        }

                                    }
                                    CutePacketValid::DataShort => {
                                        continue;
                                    }
                                    CutePacketValid::ValidFailed(_) => {
                                        if drain_size > 0 {
                                            store_buffer.drain(0..drain_size);
                                        } else {
                                            store_buffer.clear();
                                        }
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                if e.kind() != std::io::ErrorKind::WouldBlock {
                                    println!("client Error reading from stream: {}", e);
                                    break;
                                }
                            }
                        }
                        match rx.try_recv() {
                            Ok(res) => {
                                if let Ok(packet) = res {
                                    let protocol = packet.get_packet_protocol();
                                    let protocol_type = packet.get_packet_type();

                                    for item in P::chuck_create_packet(packet.get_payload(), protocol, protocol_type) {
                                        if let Err(e) = tcp_stream.write_all(&item.serialize()).await {
                                            break;
                                        }
                                    }
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
                }
                drop(arc_stream_map);
                drop(arc_unary_map);
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
        drop(lock_stream_map);

        let (tx,rx) = tokio::sync::mpsc::channel(64);
        let mut lock_stream_map = self.stream_map.lock().await;
        lock_stream_map.entry(protocol).or_insert(tx);
        drop(lock_stream_map);

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

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    pub async fn close_stream(&self, protocol : u32) -> Result<(),CuteError> {
        let _ = self.send_tx.send(Ok(P::send_create_packet(vec![0,0,0,0],protocol,CutePacketType::StreamClose))).await.
            map_err(|e| CuteError::internal(format!("{:?}", e)))?;

        Ok(())
    }

    pub async fn close_stream_all(&self) -> Result<(),CuteError> {
        let _ = self.send_tx.send(Ok(P::send_create_packet(vec![0,0,0,0],0x0FFFFFFF,CutePacketType::StreamAllClose))).await.
            map_err(|e| CuteError::internal(format!("{:?}", e)))?;

        Ok(())
    }
}