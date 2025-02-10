//
// Created by 박경빈 on 25. 1. 31.
//

# pragma once
#include "common/include/common.h"
#include "input.h"
#include "output.h"


__BEGIN_DECLS
cute_driver_result init_board_high();
void destroy_board_high(cute_driver_result* self);

cute_driver_result create_echo_task(void *parameter);
cute_driver_result execute_echo_task(cute_driver_result* self);
void destroy_echo_task(cute_driver_result* self);

char* get_board_high_name();
__END_DECLS