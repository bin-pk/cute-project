use cute_core::{Task,TaskConstructor,CuteError};

pub use self::test::*;
pub use self::constructor::*;
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

mod test;
mod constructor;