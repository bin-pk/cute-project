# Raw
Code 작성자가 임의로 작성한 TCP 용 Stub 이다. 

# CutePacketTrait
Server 및 Client 가 받거나 전송할 데이터를 처리하기 위한 특성이다.

해당 특성을 상속받은 경우 아래의 경우를 필수적으로 구현하여야 한다.
+ `get_header_size`
  + recv 된 데이터의 binary 값을 확인시 header 크기 확인
+ `get_drain_size`
  + recv 된 데이터의 `is_valid` 가 `ValidFailed(CuteError)` 가 나온경우 버릴 길이.
  + 0 인 경우 기존에 저장하던 모든 버퍼를 초기화 한다.
+ `is_valid`
  + 유효성 검사를 진행.
  + 아래와 같이 구성되어 있음
  ```rust
    pub enum CutePacketValid {
        /// 아무 문제 없음. header + payload + tail 의 binary 길이를 반환.
        ValidOK(usize),
        /// Valid 체크를 하기에는 부족한 경우.
        DataShort,
        /// 헤더 잘못되었을 경우.
        ValidFailed(CuteError),
    }
  ```
  + header, data, tail 등이 전부 만족하는 경우 `ValidOK(usize)`
  + header 등을 만족할 만큼의 데이터 크기가 안되는 경우 `DataShort`
  + header 등 유효성 검사를 실패한 경우 `ValidFailed(CuteError)`
    + 명시된 `get_drain_size` 만큼 지움.
+ `recv_create_packet`
  + `ValidOK(usize)` 의 usize 반환값 만큼 읽어서 packet 을 만들어낸다. 
+ `chuck_create_packet`
  + payload 등의 데이터 덩어리를 protocol 및 type 을 붙여서 packet list 를 만들어냄. chuck 하게 하고 싶으면 사용.
+ `send_create_packet`
  + payload 등의 데이터 덩어리를 protocol 및 type 을 붙여서 단일 packet 으로 생성.
+ `get_packet_protocol`
  + 해당 packet 요소에서 protocol 정보 추출.
+ `get_packet_type`
  + 해당 packet 요소에서 protocol type 정보 추출.
  + enum 은 아래와 같음.
  ```rust
    pub enum CutePacketType {
        Empty = 0,
        Unary = 1,
        Streaming = 2,
        StreamClose = 3,
        StreamAllClose = 4,
    }
  ```
+ `get_chuck_idx`
  + 해당 packet 요소에서 chuck 데이터인 경우 chuck 된 위치를 반환.
  + chuck 하지 않는다면 값은 0 임.
+ `get_chuck_size`
  + 해당 packet 요소에서 chuck 데이터인 경우 chuck 크기를 반환.
  + chuck 하지 않는다면 값은 1 임.
+ `get_payload`
  + packet 에서 실제 데이터 요소만 추출.
+ `serialize`
  + packet 구조체를 binary 로 변환.

# Server
tokio tcp server 를 사용하여 구성하였다.

### Server Thread
총 3개의 Thread 및 한개의 loop 가 동작한다.
+ Loop
  + `Accept` 를 수행하며 Client 의 연결을 수행한다.
  + `Accept` 성공시 반환되는 `tcp_stream` 및 `remote_addr` 을 이용하여 공유 가능한 Mutex HashMap 을 구성한다.
+ Thread
  + tcp_read & close.
  + tcp_write.
  + task stream executor.

Read 는 NonBlocking 방식을 사용하여 데이터 읽기를 수행한다.
Write 의 경우 보수적으로 write_all 을 사용하여 모든 데이터를 tcp_stream 에 쓰기까지 대기한다.

Read 및 Write 시에 받은 binary 데이터는 `CutePacketTrait` 특성을 만족하며 변환된다.


