//
// Created by 박경빈 on 25. 1. 30.
//

#pragma once

#include "type.h"

#define CUTE_STACK_MAXIMUM 256

typedef enum cute_error_code {
    /*
     * 아무것도 없는 데이터인 경우
     */
    CUTE_EMPTY = 0,
    /*
     * stack 영역에 데이터가 있는 경우
     */
    CUTE_STACK_OK = 1,
    /*
     * heap 영역에 데이터가 있는 경우
     */
    CUTE_HEAP_OK = 2,
    /*
     * 기타 알수 없는 내부적 상황에 대한 에러
     */
    CUTE_INTERNAL_ERROR,
    /*
     * driver 등이 동작하면서 일어난 경우
     */
    CUTE_DRIVER_ERROR,
}CUTE_ERROR_CODE;

/*
 * create , execute 의 작업에 대한 결과로 무조건적으로 반환됩니다.
 *
 * 문자 열의 경우 최대 256 이상 넘어설 수 없습니다.
 */
typedef struct {
    CUTE_ERROR_CODE code;
    u32 len;
    union {
        /*
         * heap 영역에 데이터.
         */
        void* heap_data;
        /*
         * stack 영역에 데이터 또는 에러 문자열.
         */
        unsigned char stack_data[CUTE_STACK_MAXIMUM];
    } result;
} cute_driver_result;

__BEGIN_DECLS
cute_driver_result cute_empty_ok();
cute_driver_result cute_stack_ok(u32 length, void* ptr);
cute_driver_result cute_heap_ok(u32 length, void* ptr);
cute_driver_result cute_internal_err(char* err_str);
cute_driver_result cute_driver_err(char* err_str);
__END_DECLS