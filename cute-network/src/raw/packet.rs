use std::fmt::Debug;
use cute_core::CuteError;

pub enum CutePacketValid {
    /// 아무 문제 없음. header + payload + tail 의 binary 길이를 반환.
    ValidOK(usize),
    /// Valid 체크를 하기에는 부족한 경우.
    DataShort,
    /// 헤더 잘못되었을 경우.
    ValidFailed(CuteError),
}

pub enum CutePacketType {
    Empty = 0,
    Unary = 1,
    Streaming = 2,
    StreamClose = 3,
    StreamAllClose = 4,
}

pub trait CutePacketTrait : Send + Sync + 'static {
    /// 해당 packet 의 header size.
    fn get_header_size() -> usize;
    /// 해당 packet 의 tail size.
    fn get_tail_size() -> usize;
    /// `CutePacketValid::ValidFailed(err)` 이 나오는 경우 어떻게 처리를 해줘야 하냐?
    ///
    /// 0 인 경우 clear 시킴.
    ///
    /// 아닌 경우 store_data 를 반환된 길이만큼 drain 함.
    fn get_drain_size() -> usize;
    /// tcp_stream 에서 packet 에더 들어오는 데이터를 검증및 확인해 줌. 문제가 없다면 usize 반환.
    ///
    /// 1. header 체크.
    /// 2. 데이터 가져옴.
    /// 3. tail 확인.
    fn is_valid(store_data : &Vec<u8>) -> CutePacketValid;
    /// read 의 binary 데이터를 통해 packet 을 생성함.
    fn recv_create_packet(store_data : &[u8]) -> Box<Self>;
    /// chuck 된 packet 들을 만들어냄
    fn chuck_create_packet(write_data : Vec<u8>, protocol : u32,protocol_type : CutePacketType) -> Vec<Box<Self>>;
    fn send_create_packet(write_data : Vec<u8>, protocol : u32,protocol_type : CutePacketType) -> Box<Self>;

    /// virtual 함수임.
    ///
    /// error 구문을 반환하거나 보내고 싶은 경우 사용. packet 을 생성함.
    ///
    /// 만약 error 를 반환하고 싶지 않은 경우 해당 함수는 무조건 None 으로 반환시킴.
    ///
    /// 기본은 반환하지 않음.
    fn error_create_packet(err : CuteError) -> Option<Box<Self>> {
        None
    }

    fn get_packet_protocol(&self) -> u32;

    fn get_packet_type(&self) -> CutePacketType;
    /// chuck 된 요소를 이용하는 경우 사용.
    ///
    /// 아닌 경우 알아서 하기 바람.
    fn get_chuck_idx(&self) -> usize;
    /// chuck 된 요소를 이용하는 경우 사용.
    ///
    /// 아닌 경우 알아서 하기 바람.
    fn get_chuck_size(&self) -> usize;

    fn get_payload(&self) -> Vec<u8>;

    /// 자기 자신을 직렬화 시켜줌. send 시 사용.
    fn serialize(&self) -> Vec<u8>;
}

pub const CUTE_DELIMITER : u32 = 0x12345678;
pub const HEADER_SIZE: usize = 24;
pub const TAIL_SIZE: usize = 4;
pub const MAX_PAYLOAD_SIZE: usize = 65536 - HEADER_SIZE- TAIL_SIZE;

#[derive(Default, Debug)]
pub struct CutePacketHeader {
    delimiter : u32,
    protocol : u32,
    length : u32,
    compress_length : u32,
    protocol_type : u32,
    idx: u16,
    count : u16,
}

#[derive(Debug)]
pub struct CutePacket {
    header : CutePacketHeader,
    payload : Vec<u8>,
    tail : u32,
}
impl Default for CutePacket {
    fn default() -> Self {
        Self {
            header: Default::default(),
            payload: vec![0; MAX_PAYLOAD_SIZE],
            tail: 0,
        }
    }
}

impl CutePacketTrait for CutePacket {
    fn get_header_size() -> usize {
        HEADER_SIZE
    }

    fn get_tail_size() -> usize {
        TAIL_SIZE
    }

    fn get_drain_size() -> usize {
        0
    }

    fn is_valid(store_data: &Vec<u8>) -> CutePacketValid {
        if store_data.len() < HEADER_SIZE {
            CutePacketValid::DataShort
        } else {
            let mut u32_bytes = [0u8; 4];
            let mut u16_bytes = [0u8; 2];

            u32_bytes.copy_from_slice(&store_data[0.. 4]);
            let data_delimiter = u32::from_le_bytes(u32_bytes);
            u32_bytes.copy_from_slice(&store_data[4..8]);
            let data_protocol = u32::from_le_bytes(u32_bytes);
            u32_bytes.copy_from_slice(&store_data[8..12]);
            let data_len = u32::from_le_bytes(u32_bytes);
            u32_bytes.copy_from_slice(&store_data[12..16]);
            let data_comp_len = u32::from_le_bytes(u32_bytes);
            u32_bytes.copy_from_slice(&store_data[16..20]);
            let proc_type = u32::from_le_bytes(u32_bytes);
            u16_bytes.copy_from_slice(&store_data[20..22]);
            let idx = u16::from_le_bytes(u16_bytes);
            u16_bytes.copy_from_slice(&store_data[22..24]);
            let count = u16::from_le_bytes(u16_bytes);

            u32_bytes.copy_from_slice(&store_data[HEADER_SIZE + data_len as usize..HEADER_SIZE + TAIL_SIZE + data_len as usize]);
            let tail = u32::from_le_bytes(u32_bytes);

            if data_delimiter != CUTE_DELIMITER {
                CutePacketValid::ValidFailed(CuteError::internal("Packet delimiter do not match."))
            } else {
                if tail == data_delimiter + data_protocol + data_len + data_comp_len + proc_type + idx as u32 + count as u32 {
                    CutePacketValid::ValidOK(HEADER_SIZE + data_len as usize + TAIL_SIZE)
                } else {
                    CutePacketValid::ValidFailed(CuteError::internal("Packet valid failed."))
                }
            }
        }
    }

    fn recv_create_packet(store_data: &[u8]) -> Box<Self> {
        let mut u32_bytes = [0u8; 4];
        let mut u16_bytes = [0u8; 2];

        u32_bytes.copy_from_slice(&store_data[0.. 4]);
        let data_delimiter = u32::from_le_bytes(u32_bytes);
        u32_bytes.copy_from_slice(&store_data[4..8]);
        let data_protocol = u32::from_le_bytes(u32_bytes);
        u32_bytes.copy_from_slice(&store_data[8..12]);
        let data_len = u32::from_le_bytes(u32_bytes);
        u32_bytes.copy_from_slice(&store_data[12..16]);
        let data_comp_len = u32::from_le_bytes(u32_bytes);
        u32_bytes.copy_from_slice(&store_data[16..20]);
        let proc_type = u32::from_le_bytes(u32_bytes);
        u16_bytes.copy_from_slice(&store_data[20..22]);
        let idx = u16::from_le_bytes(u16_bytes);
        u16_bytes.copy_from_slice(&store_data[22..24]);
        let count = u16::from_le_bytes(u16_bytes);

        u32_bytes.copy_from_slice(&store_data[HEADER_SIZE + data_len as usize..HEADER_SIZE + TAIL_SIZE + data_len as usize]);
        let tail = u32::from_le_bytes(u32_bytes);

        Box::new(Self {
            header: CutePacketHeader {
                delimiter: data_delimiter,
                protocol: data_protocol,
                length: data_len,
                compress_length: data_comp_len,
                protocol_type: proc_type,
                idx,
                count,
            },
            payload: store_data[HEADER_SIZE..HEADER_SIZE + data_len as usize].to_vec(),
            tail,
        })
    }

    fn chuck_create_packet(write_data: Vec<u8>, protocol: u32, protocol_type: CutePacketType) -> Vec<Box<Self>> {
        let chuck_size = (write_data.len() / MAX_PAYLOAD_SIZE) + (write_data.len() % MAX_PAYLOAD_SIZE != 0) as usize;
        let mut result = vec![];
        let proc_type = protocol_type as u32;

        if write_data.len() > MAX_PAYLOAD_SIZE {
            for (idx, item) in write_data.chunks(MAX_PAYLOAD_SIZE).enumerate() {
                result.push(Box::new(Self {
                    header: CutePacketHeader {
                        delimiter: CUTE_DELIMITER,
                        protocol,
                        length: item.len() as u32,
                        compress_length: 0,
                        protocol_type: proc_type,
                        idx: idx as u16,
                        count: chuck_size as u16,
                    },
                    payload: item.to_vec(),
                    tail: CUTE_DELIMITER + protocol + item.len() as u32 + 0 + proc_type + idx as u32 + chuck_size as u32,
                }));
            }
        } else {
            let write_len = write_data.len();
            result.push(Box::new(Self {
                header: CutePacketHeader {
                    delimiter: CUTE_DELIMITER,
                    protocol: protocol,
                    length: write_len as u32,
                    compress_length: 0,
                    protocol_type: proc_type,
                    idx: 0,
                    count: 1,
                },
                payload: write_data,
                tail:CUTE_DELIMITER + protocol + write_len as u32 + 0 + proc_type + 0 + 1,
            }));
        }
        result
    }

    fn send_create_packet(write_data: Vec<u8>, protocol: u32, protocol_type: CutePacketType) -> Box<Self> {
        let write_len = write_data.len();
        let proc_type = protocol_type as u32;
        Box::new(Self {
            header: CutePacketHeader {
                delimiter: CUTE_DELIMITER,
                protocol,
                length: write_len as u32,
                compress_length: 0,
                protocol_type: proc_type,
                idx: 0,
                count: 1,
            },
            payload: write_data,
            tail: CUTE_DELIMITER + protocol + write_len as u32 + 0 + proc_type + 0 + 1,
        })
    }

    fn get_packet_type(&self) -> CutePacketType {
        match self.header.protocol_type {
            1 => {
                CutePacketType::Unary
            }
            2 => {
                CutePacketType::Streaming
            }
            3 => {
                CutePacketType::StreamClose
            },
            4 => {
                CutePacketType::StreamAllClose
            },
            _ => {
                CutePacketType::Empty
            }
        }
    }

    fn get_packet_protocol(&self) -> u32 {
        self.header.protocol
    }
    fn get_chuck_idx(&self) -> usize {
        self.header.idx as usize
    }

    fn get_chuck_size(&self) -> usize {
        self.header.count as usize
    }

    fn get_payload(&self) -> Vec<u8> {
        self.payload.clone()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut create_output = [0u8; HEADER_SIZE + MAX_PAYLOAD_SIZE + TAIL_SIZE];

        let payload_len = self.payload.len();
        create_output[0..4].copy_from_slice(self.header.delimiter.to_le_bytes().as_ref());
        create_output[4..8].copy_from_slice(self.header.protocol.to_le_bytes().as_ref());
        create_output[8..12].copy_from_slice(self.header.length.to_le_bytes().as_ref());
        create_output[12..16].copy_from_slice(self.header.compress_length.to_le_bytes().as_ref());
        create_output[16..20].copy_from_slice(self.header.protocol_type.to_le_bytes().as_ref());
        create_output[20..22].copy_from_slice(self.header.idx.to_le_bytes().as_ref());
        create_output[22..24].copy_from_slice(self.header.count.to_le_bytes().as_ref());
        create_output[HEADER_SIZE..HEADER_SIZE + payload_len].copy_from_slice(self.payload.as_ref());
        create_output[HEADER_SIZE + payload_len..HEADER_SIZE + payload_len + TAIL_SIZE].copy_from_slice(self.tail.to_le_bytes().as_ref());

        create_output[0..HEADER_SIZE + payload_len + TAIL_SIZE].to_vec()
    }
}
