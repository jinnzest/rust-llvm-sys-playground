use std::ffi::CStr;
use std::os::raw::c_char;


#[no_mangle]
pub extern fn hello_world(){
    println!("Hello world");
}

#[no_mangle]
pub extern fn hello_one(ptr: *const c_char){
    let cstr = unsafe { CStr::from_ptr(ptr) };

    match cstr.to_str() {
        Ok(s) => {
          println!("Hello, {}",s);
        }
        Err(_) => {
          panic!("broken c string!");
        }
    }

}

#[no_mangle]
pub extern fn create_str() -> *const u8 {
  "some string\0\0".as_ptr()
}

#[no_mangle]
pub extern fn create_i8() -> i8{
  123
}

#[no_mangle]
pub extern fn create_vector() -> Vec<i8>{
  vec![1,2,3]
}

#[repr(C)]
#[derive(Debug)]
pub struct TestS{
  num1: i32,
  num2: i32
}

#[no_mangle]
pub extern fn create_test() -> Box<TestS>{
    let d = TestS{num1: 1,num2: 23};
    Box::new(d)
}

#[no_mangle]
pub extern fn create_slice() -> Box<[u32;3]>{
    let d = [1,2,3];
    Box::new(d)
}