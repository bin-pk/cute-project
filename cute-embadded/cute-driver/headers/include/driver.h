//
// Created by 박경빈 on 25. 1. 30.
//
#pragma once

#include "../common/include/type.h"
#include "../common/include/error.h"
#include "constants.h"
#include "input.h"
#include "output.h"

__BEGIN_DECLS
/*
 * 임베디드 영역의 요소들을 초기화 합니다.
 *
 * 이작업은 초기화 할 필요가 없는 경우 생략합니다.
 */
cute_driver_result init_driver();

cute_driver_result create_driver_task(u32 protocol,void* parameter);

cute_driver_result execute_driver_task(u32 protocol,cute_driver_result* self);
/*
 * driver 의 version 을 가져옵니다. 숫자 0 은 _ 와 같습니다.
 */
u32 get_driver_version();
__END_DECLS