use cute_core::create_task_constructor;
use crate::context::*;
use crate::tasks::*;

create_task_constructor!(EchoTask, EchoTaskConstructor, TestContext);
create_task_constructor!(TestTask, TestTaskConstructor, TestContext);