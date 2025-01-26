use cute_core::CuteError;
pub use self::server::CuteRawServer;
pub use self::client::RawClient;
pub use self::packet::CutePacket;

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
    fn chuck_create_packet(write_data : Vec<u8>, protocol : u32, protocol_type : CutePacketType) -> Vec<Box<Self>>;
    fn send_create_packet(write_data : Vec<u8>, protocol : u32, protocol_type : CutePacketType) -> Box<Self>;

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

mod client;
mod server;
mod packet;
mod stub;