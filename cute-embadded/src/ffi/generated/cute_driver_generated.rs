/* automatically generated by rust-bindgen 0.69.5 */

pub const true_: u32 = 1;
pub const false_: u32 = 0;
pub const CUTE_STACK_MAXIMUM: u32 = 64;
pub type u8_ = ::std::os::raw::c_uchar;
pub type u16_ = ::std::os::raw::c_ushort;
pub type u32_ = ::std::os::raw::c_uint;
pub type u64_ = ::std::os::raw::c_ulonglong;
pub type i8_ = ::std::os::raw::c_char;
pub type i16_ = ::std::os::raw::c_short;
pub type i32_ = ::std::os::raw::c_int;
pub type i64_ = ::std::os::raw::c_longlong;
pub type f32_ = f32;
pub type f64_ = f64;
pub type bool_ = u8_;
pub const cute_error_code_CUTE_EMPTY: cute_error_code = 0;
pub const cute_error_code_CUTE_STACK_OK: cute_error_code = 1;
pub const cute_error_code_CUTE_HEAP_OK: cute_error_code = 2;
pub const cute_error_code_CUTE_INTERNAL_ERROR: cute_error_code = 3;
pub const cute_error_code_CUTE_DRIVER_ERROR: cute_error_code = 4;
pub type cute_error_code = ::std::os::raw::c_int;
pub use self::cute_error_code as CUTE_ERROR_CODE;
pub type destroy_function =
    ::std::option::Option<unsafe extern "C" fn(self_: *mut cute_driver_result)>;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct cute_driver_result {
    pub code: CUTE_ERROR_CODE,
    pub len: u32_,
    pub result: cute_driver_result__bindgen_ty_1,
    pub destroy: destroy_function,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union cute_driver_result__bindgen_ty_1 {
    pub heap_data: *mut ::std::os::raw::c_void,
    pub stack_data: [::std::os::raw::c_uchar; 64usize],
}
#[test]
fn bindgen_test_layout_cute_driver_result__bindgen_ty_1() {
    const UNINIT: ::std::mem::MaybeUninit<cute_driver_result__bindgen_ty_1> =
        ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<cute_driver_result__bindgen_ty_1>(),
        64usize,
        concat!("Size of: ", stringify!(cute_driver_result__bindgen_ty_1))
    );
    assert_eq!(
        ::std::mem::align_of::<cute_driver_result__bindgen_ty_1>(),
        8usize,
        concat!(
            "Alignment of ",
            stringify!(cute_driver_result__bindgen_ty_1)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).heap_data) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(cute_driver_result__bindgen_ty_1),
            "::",
            stringify!(heap_data)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).stack_data) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(cute_driver_result__bindgen_ty_1),
            "::",
            stringify!(stack_data)
        )
    );
}
#[test]
fn bindgen_test_layout_cute_driver_result() {
    const UNINIT: ::std::mem::MaybeUninit<cute_driver_result> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<cute_driver_result>(),
        80usize,
        concat!("Size of: ", stringify!(cute_driver_result))
    );
    assert_eq!(
        ::std::mem::align_of::<cute_driver_result>(),
        8usize,
        concat!("Alignment of ", stringify!(cute_driver_result))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).code) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(cute_driver_result),
            "::",
            stringify!(code)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).len) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(cute_driver_result),
            "::",
            stringify!(len)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).result) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(cute_driver_result),
            "::",
            stringify!(result)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).destroy) as usize - ptr as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(cute_driver_result),
            "::",
            stringify!(destroy)
        )
    );
}
extern "C" {
    pub fn cute_empty_ok() -> cute_driver_result;
}
extern "C" {
    pub fn cute_stack_ok(length: u32_, ptr: *mut ::std::os::raw::c_void) -> cute_driver_result;
}
extern "C" {
    pub fn cute_heap_ok(length: u32_, ptr: *mut ::std::os::raw::c_void) -> cute_driver_result;
}
extern "C" {
    pub fn cute_internal_err(err_str: *mut ::std::os::raw::c_char) -> cute_driver_result;
}
extern "C" {
    pub fn cute_driver_err(err_str: *mut ::std::os::raw::c_char) -> cute_driver_result;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct echo_input {
    pub not_used: i32_,
}
#[test]
fn bindgen_test_layout_echo_input() {
    const UNINIT: ::std::mem::MaybeUninit<echo_input> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<echo_input>(),
        4usize,
        concat!("Size of: ", stringify!(echo_input))
    );
    assert_eq!(
        ::std::mem::align_of::<echo_input>(),
        4usize,
        concat!("Alignment of ", stringify!(echo_input))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).not_used) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(echo_input),
            "::",
            stringify!(not_used)
        )
    );
}
pub type cute_echo_input = echo_input;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct echo_output {
    pub count: i32_,
}
#[test]
fn bindgen_test_layout_echo_output() {
    const UNINIT: ::std::mem::MaybeUninit<echo_output> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<echo_output>(),
        4usize,
        concat!("Size of: ", stringify!(echo_output))
    );
    assert_eq!(
        ::std::mem::align_of::<echo_output>(),
        4usize,
        concat!("Alignment of ", stringify!(echo_output))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).count) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(echo_output),
            "::",
            stringify!(count)
        )
    );
}
pub type cute_echo_output = echo_output;
extern "C" {
    pub fn init_driver() -> cute_driver_result;
}
extern "C" {
    pub fn create_driver_task(
        protocol: u32_,
        parameter: *mut ::std::os::raw::c_void,
    ) -> cute_driver_result;
}
extern "C" {
    pub fn execute_driver_task(
        protocol: u32_,
        self_: *mut cute_driver_result,
    ) -> cute_driver_result;
}
extern "C" {
    pub fn get_driver_version() -> u32_;
}
