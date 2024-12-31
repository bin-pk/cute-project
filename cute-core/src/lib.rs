use std::sync::Arc;

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

    /// heap 영역의 자기자신을 반환해 준다.
    fn new(input : Option<Box<[u8]>>) -> Result<Box<Self>, std::io::Error>
    where Self: Sized;

    /// 생성자에서 생성한 후에 동작을 수행후 결과를 반환한다.
    ///
    /// mut 가능하도록 한 것은 자기 자신 내부에서 Task 를 생성해 그 Task 결과가 동적 프로그래밍과 같이 작동할 수 있기에 mutable 하도록 함.
    async fn execute(&mut self, ctc: Arc<tokio::sync::RwLock<C>>) -> Result<Self::Output, std::io::Error>;

    /// 경우에 따라 생성자에서 생성시에 나온 Member 변수를 할당해제등을 수행하거나 생존주기를 종료시킬떄 사용.
    async fn destroy(&mut self);
}