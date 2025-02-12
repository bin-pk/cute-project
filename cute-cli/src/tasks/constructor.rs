use cute_core::create_task_constructor;
use cute_embadded::*;
use crate::context::*;
use crate::tasks::*;

create_task_constructor!(EchoTask, EchoTaskConstructor, TestContext);
create_task_constructor!(TestTask, TestTaskConstructor, TestContext);
create_task_constructor!(EmbeddedEchoTask, EmbeddedEchoTaskConstructor, EmbeddedContext);