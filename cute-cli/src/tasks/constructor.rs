use crate::context::*;
use crate::tasks::*;

create_task_constructor!(EchoTask, EchoTaskConstructor, TestContext);
create_task_constructor!(TestTask, TestTaskConstructor, TestContext);