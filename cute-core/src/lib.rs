use std::pin::Pin;
use std::sync::Arc;
pub use self::serdes::*;
pub use self::works::*;
pub use self::tasks::*;
pub use self::procs::*;
pub use self::errors::{CuteError, CuteErrorCode};
pub type DataStream<T> = Pin<Box<dyn tokio_stream::Stream<Item = Result<T, CuteError>> + Send>>;

/// # Comment
/// 특정 작업에 대한 것을 잓성하는 Trait 이다.
///
/// 생성자에 의해 생성될 것을 생각하여 trait 에 new 를 추가하였다.
///
/// 내부 `execute` 작업시에 해당 객체에 메모리 등을 생성하여 해제 할 경우를 대비하여 destroy 도 추가함.
/// # 보장 및 설명
/// + Sync : 여러 Thread 간의 참조(ref)시 공유 가능하도록 합니다.
/// + Send : 여러 Thread 간의 이동을 보장합니다.
/// + 생존주기 : 생성자를 통해 생성된 후에 Scope 를 벗어날 경우 없어지기에 `'static` 을 보장하지 않음.
#[async_trait::async_trait]
pub trait Task<C> : Send + Sync {
    type Output: serde::Serialize + Send;

    /// 생성자.
    fn new(input : Option<Box<[u8]>>) -> Result<Box<Self>, CuteError>
    where Self: Sized;

    /// 생성자에서 생성한 후에 동작을 수행후 결과를 반환한다.
    ///
    /// mut 가능하도록 한 것은 자기 자신 내부에서 Task 를 생성해 그 Task 결과가 동적 프로그래밍과 같이 작동할 수 있기에 mutable 하도록 함.
    async fn execute(&mut self, ctc: Arc<tokio::sync::RwLock<C>>) -> Result<Self::Output, CuteError>;

    /// 경우에 따라 생성자에서 생성시에 나온 Member 변수를 할당해제등을 수행하거나 생존주기를 종료시킬떄 사용.
    async fn destroy(&mut self);
}

/// Task 생성자.
///
/// 이것을 만드는 이유는 Procedure 특성을 지정받을 것을 만들때 Worker 의 Output 및 Task 의 Output 을 명시적으로 정의해줘야 하는데
///
/// 그 작업이 쉽지 않기 때문. 그레서 생성 대리자와 같은 느낌으로 사용.
pub trait TaskConstructor<T,C> : Send + Sync
where T : Task<C> + Send + Sync,
      C : Send + Sync + 'static,
{
    /// 사용시 Task::new() 가 내부적으로 동작한다. 이것이 전부이다.
    fn create(&self, input : Option<Box<[u8]>>) -> Result<Box<T>, CuteError> {
        T::new(input)
    }
}

/// # Comment
/// 특정 작업에 대한 동작을 정의합니다.
///
/// 모든 동작에 대하여 Task 의 생성자를 이용하여 함수 내부에서 Task 를 생성하여 사용합니다.
#[async_trait::async_trait]
pub trait Worker<C> {
    /// 한번의 Input 에 대하여 한번만 동작하도록 한다.
    async fn one_of_execute(&self, ctx: Arc<tokio::sync::RwLock<C>>, input: Option<Box<[u8]>>) -> Result<Vec<u8>, CuteError>;
    /// 한번의 Input 에 대하여 반복적으로 동작하도록 한다.
    async fn iter_execute(&self, ctx: Arc<tokio::sync::RwLock<C>>, input: Option<Box<[u8]>>) -> Result<DataStream<Vec<u8>>, CuteError>;
    /// 반복적으로 동작하는 iter 를 강제로 종료시킨다.
    ///
    /// task 의 error 를 통해 iter 를 관리하겠다고 한다면 따로 구현하지 않아도 된다.
    async fn iter_close(&self);
}
/// # Comment
/// 해당 Trait 은 Worker 를 통하여 특정 작업에 대한 동작을 정리한 요소들을 Collection 에 저장하여 사용하는 Trait 이다.
///
/// 해당 Trait 특성을 받은 Struct 는 반드시 Parameter 로 제공되는 key 와 매칭되도록 해야함.
#[async_trait::async_trait]
pub trait Procedure<C> {
    /// 해당 Collection 의 모든 service ( worker ) 이름들을 반환.
    async fn get_service_names(&self) -> Result<Vec<String>, CuteError>;
    /// Key 를 통해 Value (Worker) 를 추출하고 추출한 Worker 에 대하여 `one_of_execute` 수행함.
    async fn one_of_run(&self, key : Box<str>, ctx: Arc<tokio::sync::RwLock<C>>, input : Option<Box<[u8]>>) -> Result<Vec<u8>, CuteError>;
    /// Key 를 통해 Value (Worker) 를 추출하고 추출한 Worker 에 대하여 `iter_execute` 수행함.
    async fn iter_run(&self, key : Box<str>, ctx: Arc<tokio::sync::RwLock<C>>, input : Option<Box<[u8]>>) -> Result<DataStream<Vec<u8>>, CuteError>;
    /// Key 를 통해 Value (Worker) 를 추출하고 추출한 Worker 에 대하여 `iter_close` 수행함.
    async fn iter_close(&self , key : Box<str>) -> Result<(), CuteError>;
    /// Collection 을 iterator 시켜서 각 worker 마다 `iter_close` 수행함.
    async fn iter_all_close(&self);
}

mod serdes;
mod works;
mod tasks;
mod procs;
mod errors;