# Cute-Core
각종 영역의 Core 가 되는 부분을 작업합니다.

## Task
```rust
pub struct TestTask {
    test : Vec<i32>
}

#[async_trait::async_trait]
impl Task<TestContext> for TestTask {

    fn new(_input: Option<Box<[u8]>>) -> Result<Box<dyn Task<TestContext> + Send>, CuteError>
    where
        Self: Sized
    {
        Ok(Box::new(Self {
            test: Vec::new()
        }))
    }

    async fn execute(&mut self, ctx: Arc<tokio::sync::RwLock<TestContext>>) -> Result<Option<Vec<u8>>, CuteError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let reader = ctx.read().await;
        let echo = TestData { data : reader.test * 2 };
        drop(reader);

        Ok(Some(bin_serialize(echo)?))
    }

    async fn destroy(&mut self) {
    }
}
```

해당 특성을 받은 Struct 는 아래 3가지에 대하여 작업을 수행해야 한다.
+ ### new
  + Parameter 로는 외부의 Input Data 가 있다.
  + 해당 외부데이터는 option 처리가 되어 있기때문에 존재하는지 반드시 확인해야 한다.
  + 작업자는 외부의 Input Data 를 `serde::Deserialize` 할지 기타 처리를 할지를 스스로 구현하면 된다.
  + 이때 작업된 결과물은 `Sized` 속성. 즉 Compile Time 에서 해당 객체의 크기를 판별가능하도록 해야한다.
+ ### execute
  + new 로 생성된 자기자신의 여러 값등을 사용하여 작업을 구현한다.
  + Parameter 로는 스마트 공유 포인터인 Arc 와 Rwlock 을 포함하는 Context 로 구성되어 있다.
    + context 의 경우 다른 Task 에서 동작한 것을 Read 해야 할 수도 있기에 RwLock 으로 구성하였다.
      + Mutex 로 구성시 읽을때도 Lock 을 걸어줘야 해서 비용이 든다.
    + 데이터를 처리 후 binary 로 변환하여 반환하여야 한다.
+ ### destroy
  + new 를 통해 생성시 메모리 해제 및 drop 등을 명시해줘야될 필요가 있는 경우 사용한다.


## Task_Constructor
작업물 생성자.

해당 생성자를 사용하는 이유는 아래와 같다.
```rust

fn main() {
  let ctx = Arc::new(tokio::sync::RwLock::new(TestContext::default()));
  // aa 의 경우 외부에서 생성 및 구현함.
  let mut aa = std::collections::HashMap::new();
  aa.entry(1).or_insert(TestTask::new(None));

  let res_task = aa.get_mut(&1).unwrap();
  match res_task {
    // 반환값으로 받는 Task 도 실제로는 aa 의 요소를 참조함.
    Ok(task) => {
      let ctx_clone = ctx.clone();
      // thread 내부로 들어오는 순간. aa 의 생존주기는 알 수 없음. 
      //
      // 그래서 aa 가 `static 특성을 만족해야 하나 그렇게 하려면 해당 aa 를 Arc<Mutex<>> 으로 감싸주어야 함.
      //
      // 매우 복잡해지며 개발자는 공유 포인터 해제 등을 관리해줘야 함으로 힘듬.
      tokio::spawn(async move {
        let mut count = 0;
        loop {
          if count > 10 {
            break;
          }
          let _ = task.execute(ctx_clone.clone()).await;
          count += 1;
        }
        drop(ctx_clone);
      });
    }
    Err(_) => {}
  }
}
```
위와 같이 Task 를 생성시 다음의 코드에서 문제가 발생한다.
1. aa 의 생존 주기 (수명) 문제로 인하여 task 자체를 thread 한곳에서 안전하게 사용 불가.
2. 작업물은 Parameter 마다 변경되어 수행되어야 하는데 collection 저장시에 이미 등록이 되기에 불가능한 방법이 되어버림.

회피 방법으로는 aa 자체를 공유 포인터 및 Mutex 에 깜싸서 처리하는 방법을 쓰거나

Task 를 Collection 에서 매번 Input 을 받을때마다 Update 를 시켜줘야 한다. 

심지어 update 시키는 곳은 해당 key 와 매칭되도록 따로 코드를 만들어 주어야 하기에 비효율적이다.

그래서 Collection 에서는 작업물 생성자만 저장하게 하여 collection 와의 Task 간의 수명 상호작용을 제거한다.

또한 아래의 Macro 를 사용하여 구현하기 쉽게 하였다.
```rust
#[macro_export]
macro_rules! create_task_constructor {
    ($task : ident,$constructor : ident,$context: ident) => {
        #[derive(Debug, Clone, Default)]
        pub struct $constructor;

        impl TaskConstructor<$context> for $constructor {
            fn create(&self, input : Option<Box<[u8]>>) -> Result<Box<dyn Task<$context> + Send>, CuteError> {
                $task::new(input)
            }
        }
    };
}
```

## Procedure
일련의 작업들을 기록하여 관리하기 위한 것.

쉽게 생각해서 RDBMS 에서 사용하는 Procedure 와 동일하다고 생각하면 편함.

예제 코드는 gRPC Server 에 Procedure 를 적용하도록 하였다.

```rust
#[tokio::main]
async fn main() {
    let mut proc_map = ProcManager::new();
    proc_map.insert("echo".into(), Box::new(EchoTaskConstructor::default()));
    proc_map.insert("test".into(), Box::new(TestTaskConstructor::default()));
    match cute_network::Server::create_grpc(cute_network::NetworkConfig::default()).start_server(Box::new(proc_map)).await { 
      Ok(_) => {}
      Err(_) => {}
    } 
}

/// server_stream 의 내용
async fn server_stream(&self, mut request: Request<Input>) -> Result<Response<Self::ServerStreamStream>, Status> {
  let (stop_signal,_) = tokio::sync::watch::channel(false);
  let name = request.get_ref().name.clone();
  let remote_addr = request
          .remote_addr()
          .map(|addr| addr.to_string())
          .unwrap_or_else(|| "unknown".to_string());
  let key_name = format!("{}_{}",remote_addr,name);

  info!("key : {}",key_name);

  let stop_rx = stop_signal.subscribe();
  let mut lock_peer_map = self.peer_map.lock().await;
  if let Some(sender) = lock_peer_map.get(&key_name.clone().into_boxed_str()) {
    let _ = sender.send(true).is_err();
  }
  lock_peer_map.entry(key_name.clone().into_boxed_str()).or_insert(stop_signal);
  drop(lock_peer_map);

  let proc_map = self.procedure.as_ref();
  match proc_map.get_task(name.clone().into_boxed_str(),
                          request.get_mut().data.take().map(Vec::into_boxed_slice)).await {
    Ok(mut task) => {
      let ctx = self.context.clone();
      let max_page_byte_size = self.config.max_page_byte_size;
      Ok(Response::new(Box::pin(stream! {
                    let mut is_closed = false;
                    loop {
                        if *stop_rx.borrow() {
                            is_closed = true;
                        }
                        if is_closed {
                            break;
                        } else {
                            match task.execute(ctx.clone()).await {
                                Ok(opt_output) => {
                                    if let Some(output) = opt_output {
                                        let output_len = output.len();
                                        let chuck_size = output_len / max_page_byte_size + (output_len % max_page_byte_size != 0) as usize;
                                        for (chuck_idx, chuck_item) in output.chunks(max_page_byte_size).enumerate() {
                                            yield Ok( Output {
                                                name : name.clone(),
                                                page_size: chuck_size as u32,
                                                page_idx: chuck_idx as u32,
                                                data: chuck_item.to_vec(),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    yield Err(convert_cute_error_to_status(e));
                                }
                            }
                        }
                    }
                    info!("Server Stream stopped");
                })))
    }
    Err(e) => {
      Err(convert_cute_error_to_status(e))
    }
  }
}
```
사용을 하는 부분은 아래와 같다.

1. main 에서 collection 생성 및 인자로 전송.
2. 위의 코드와 같이 해당 `server_stream` 함수에서는 key 값을 `get_task` 실행.
3. `get_task` 에서는 HashMap 에서 입력받은 key 값을 토대로 `TaskConstructor` 추출.
4. 추출한 `TaskConstructor` 에서 create 를 실행하여 `Task` 반환.
5. 해당 `Task` 는 해당 함수가 종료되기 전까지 생존함.

위와 같은 방식으로 동작하도록 구현되어 있다.

streaming 방식등을 사용 목적에 따라 개발자가 별도로 구현하여야 한다.


## Serde
```rust
pub fn bin_serialize<T : Serialize>(input : T) -> Result<Vec<u8>, CuteError>{
    bincode::serialize(&input).map_err(|e| CuteError::serialize_invalid(e.to_string()))
}

pub fn bin_deserialize<'a, T : Deserialize<'a>>(input: &'a [u8]) -> Result<T, CuteError>{
    bincode::deserialize(input).map_err(|e| CuteError::deserialize_invalid(e.to_string()))
}

```

Prost (Protobuf) 및 bincode 에 대한 역직렬화, 직렬화 API.

현재는 bincode 만 작업 수행을 진행하였다.

## CuteError
다른 곳에서는 `Status` 보통의 경우 `std::io::Error` 를 사용한다. 공부를 위해서 그냥 Custom Error 를 만들어 보았다.

필요 없으면 사용안해도 된다.

gRPC 의 `Status` 코드를 보고 생성자 등을 만들었다.

