//
// Created by 박경빈 on 25. 1. 31.
//

#pragma once

#include "common/include/common.h"

__BEGIN_DECLS
cute_driver_result init_board();
void destroy_board(cute_driver_result* self);

char* get_board_low_name();
__END_DECLS