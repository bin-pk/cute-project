use std::pin::Pin;
use std::sync::Arc;
pub use self::procs::*;
pub use self::serdes::*;
pub use self::errors::{CuteError, CuteErrorCode};
pub type DataStream<T> = Pin<Box<dyn tokio_stream::Stream<Item = Result<T, CuteError>> + Send>>;

/// # Comment
/// 작업 내용을 정의
///
/// 생성자에 의해 생성될 것을 생각하여 trait 에 new 를 추가하였다.
///
/// 내부 `execute` 작업시에 해당 객체에 메모리 등을 생성하여 해제 할 경우를 대비하여 destroy 도 추가함.
#[async_trait::async_trait]
pub trait Task<C> {
    /// 생성자.
    ///
    /// 입력받은 input 을 `serde::Deserialize` 해도 되게 안해도 되도록 구현.
    fn new(input : Option<Box<[u8]>>) -> Result<Box<dyn Task<C> + Send>, CuteError>
    where Self: Sized;

    /// 생성자에서 생성한 후에 동작을 수행후 결과를 반환한다.
    ///
    /// mut 가능하도록 한 것은 자기 자신 내부에서 Task 를 생성해 그 Task 결과가 동적 프로그래밍과 같이 작동할 수 있기에 mutable 하도록 함.
    async fn execute(&mut self, ctx: Arc<tokio::sync::RwLock<C>>) -> Result<Option<Vec<u8>>, CuteError>;

    /// 경우에 따라 생성자에서 생성시에 나온 Member 변수를 할당해제등을 수행하거나 생존주기를 종료시킬떄 사용.
    async fn destroy(&mut self);
}

/// # Comment
/// Task 생성자.
///
/// 현재는 사라진 worker 의 역솰을 다른 곳에서 하도록 하고
///
/// 생성자만 받도록 추가함.
///
/// 해당 작업은 macro 를 통해 만들기 쉽게 구현함.
pub trait TaskConstructor<C> : Send + Sync
where C : Send + Sync + 'static,
{
    /// 사용시 Task::new() 를 통해 Task 를 생성한다.
    ///
    /// Impl 하지 않고 dynamic 하게 하여 std::collection 등에 추가하기 쉽게 하였다.
    fn create(&self, input : Option<Box<[u8]>>) -> Result<Box<dyn Task<C> + Send>, CuteError>;
}

/// # Comment
/// 특정한 작업들을 재활용 하도록 hash_map 와 같은 std::collection 에 기록하여 관리
///
/// 실세 작업에 대한 절차는 `get_task` 를 통해 받은 곳에서 절차를 작업함.
///
/// 여기서는 `TaskConstructor`에 의해 생성된 `Task` 만을 반환함.
#[async_trait::async_trait]
pub trait Procedure<C> {
    /// 현재 std::collection 에 기록된 모든 작업 내용을 반환.
    async fn get_service_protocols(&self) -> Result<Vec<u32>, CuteError>;
    /// 해당 이름을 가진 작업을 반환.
    async fn get_task(&self, key : u32, input : Option<Box<[u8]>>) -> Result<Box<dyn Task<C> + Send>, CuteError>;
}

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

mod serdes;
mod procs;
mod errors;