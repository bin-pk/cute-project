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
/*
 * 임베디드 영역의 요소를 해제합니다.
 *
 * 초기화떄 사용했던 요소를 반환받아 수행이 가능합니다.
 */
void destroy_driver(cute_driver_result* self);

/*
 * driver 의 작업을 생성합니다.
 *
 * parameter 및 task 를 입력 받습니다.
 */
cute_driver_result create_driver_task(u32 protocol,void* parameter);
/*
 * 생성한 작업을 실행합니다.
 *
 * 생성한 작업물을 cute_driver_result 로 반환할지 또는 파라미터로 제공된 self 내에 담을지는 알아서 결정하기 바람.
 */
cute_driver_result execute_driver_task(u32 protocol,cute_driver_result* self);
/*
 * 생성한 작업 및 실행한 작업 물을 할당 해제 합니다.
 *
 * 이떄 dangling pointer 문제를 피하기 위해 이중 포인터로 받고 해당 포인터 자체를 null 처리 가능하도록 합니다.
 */
void destroy_driver_task(u32 protocol,cute_driver_result* self);

/*
 * driver 의 version 을 가져옵니다. 숫자 0 은 _ 와 같습니다.
 */
u32 get_driver_version();
__END_DECLS