use std::ffi::*;
use std::ptr::null_mut;

extern "C" {
    fn mp_init(mp: *mut MpInt) -> i32;
    fn mp_read_radix(mp: *mut MpInt, s: *const i8, radix: i32) -> i32;
    fn mp_add(mp: *mut MpInt, mp2: *mut MpInt, mp3: *mut MpInt) -> i32;
    fn mp_radix_size(mp: *mut MpInt, radix: i32, size: *mut i32) -> i32;
    fn mp_toradix(mp: *mut MpInt, s: *mut i8, radix: i32) -> i32;
}

#[repr(C)]
struct MpInt {
    used: i32,
    alloc: i32,
    sign: i32,
    mp_digit: *mut MpInt,
}

pub fn load_bignum_symbols() {
    unsafe {
        let mut mp1 = MpInt {
            used: 1,
            alloc: 1,
            sign: 1,
            mp_digit: null_mut(),
        };
        let ptr1 = &mut mp1 as *mut MpInt;
        mp_init(ptr1);
        let cstring = CString::new("1").unwrap();
        let str_ptr = cstring.as_ptr() as *mut _;
        mp_read_radix(ptr1, str_ptr, 10);
        mp_add(ptr1, ptr1, ptr1);
        let mut size: i32 = 100;
        let ptr_size = &mut size as *mut i32;
        mp_radix_size(ptr1, 10, ptr_size);
        let mut s: Vec<i8> = vec![];
        s.reserve_exact(*ptr_size as usize);
        s.set_len(*ptr_size as usize);
        mp_toradix(ptr1, s.as_mut_ptr(), 10);
        if s[1] == 0 {
            println!("bigint library has been loaded");
        }
    }
}
