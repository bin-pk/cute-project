//
// Created by 박경빈 on 25. 1. 30.
//

#pragma once

#define true 1
#define false 0

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long long u64;
typedef char i8;
typedef short i16;
typedef int i32;
typedef long long i64;
typedef float f32;
typedef double f64;
typedef u8 bool;

// 기존 cdef.h 또는 feature.h define 제거.
#undef __BEGIN_DECLS
#undef __END_DECLS
// c++ 사용시 name 맹글링(컴파일러가 규칙에 따라 이름 변경) 방지하여 외부에서 사용가능하도록 하기.
// gpt 에서 가져옴.
#ifdef __cplusplus
#define __BEGIN_DECLS extern "C" {
#define __END_DECLS }
#else
#define __BEGIN_DECLS
#define __END_DECLS
#endif
