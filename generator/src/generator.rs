extern crate libc;
extern crate llvm_sys;

use self::llvm_sys::prelude::*;
use bignumloader::*;
use llvm::*;

pub fn llvm_exec() -> bool {
    load_bignum_symbols();

    let mut runner = LLVMRunner::new();
    let main = runner.mk_main_func(|ref mut r| {
        r.call_printf_func("Hello from JIT generated executable!\n", "");
        let i8_pt = r.llvm.ptr_t(r.llvm.i8_t());
        let i32_t = r.llvm.i32_t();
        let struct_mp1 = r.llvm.struct_mp();
        let struct_mp2 = r.llvm.struct_mp();
        let struct_mp3 = r.llvm.struct_mp();
        let num_ref1 = r.llvm.build_alloca("num1", struct_mp1);
        let num_ref2 = r.llvm.build_alloca("num2", struct_mp2);
        let res_str_ptr = r.llvm.build_alloca("res_str", i8_pt);
        let res_num_ref = r.llvm.build_alloca("res_num", struct_mp3);
        let str_size_ref = r.llvm.build_alloca("str_size", i32_t);
        let num_str_ref1 = r.llvm.mk_global_string("num1", "100");
        let num_str_ref2 = r.llvm.mk_global_string("num2", "10");
        r.call_mp_init(num_ref1);
        r.call_mp_init(num_ref2);
        r.call_mp_init(res_num_ref);
        r.call_mp_read_radix(num_ref1, num_str_ref1);
        r.call_mp_read_radix(num_ref2, num_str_ref2);
        r.call_mp_add(num_ref1, num_ref2, res_num_ref);
        r.call_radix_size(res_num_ref, str_size_ref);
        let str_ref = r.call_malloc(str_size_ref);
        r.call_mp_toradix(str_ref, res_num_ref, res_str_ptr);
        let loaded = r.llvm.build_load(res_str_ptr);
        r.call_printf_func_by_value("Number: %s\n", loaded);
        r.call_printf_func("Goodbye from JIT generated executable\n", "");
    });
    runner.llvm.exec_func(main)
}

pub fn llvm_compile(out_name: &str) -> bool {
    let mut runner = LLVMRunner::new();
    runner.mk_main_func(|ref mut r| {
        r.call_hello_world_function();
        r.call_printf_func("Hello, .\n", "");
        r.call_create_i8();
        r.call_create_str();
        r.call_test();
        r.call_slice();
        r.call_hello_one("Bob");
    });
    runner.llvm.dump(out_name);
    if runner.llvm.mk_object_file(out_name) {
        link()
    } else {
        false
    }
}

pub fn llvm_compile2(out_name: &str) -> bool {
    let mut runner = LLVMRunner::new();
    runner.mk_main_func(|ref mut r| {
        r.call_printf_func("Hello from compiled to a file executable!\n", "");
        let i8_pt = r.llvm.ptr_t(r.llvm.i8_t());
        let i32_t = r.llvm.i32_t();
        let struct_mp1 = r.llvm.struct_mp();
        let struct_mp2 = r.llvm.struct_mp();
        let struct_mp3 = r.llvm.struct_mp();
        let num_ref1 = r.llvm.build_alloca("num1", struct_mp1);
        let num_ref2 = r.llvm.build_alloca("num2", struct_mp2);
        let res_str_ptr = r.llvm.build_alloca("res_str", i8_pt);
        let res_num_ref = r.llvm.build_alloca("res_num", struct_mp3);
        let str_size_ref = r.llvm.build_alloca("str_size", i32_t);

        let num_str_ref1 = r.llvm.mk_global_string("num1", "100");
        let num_str_ref2 = r.llvm.mk_global_string("num2", "10");
        r.call_mp_init(num_ref1);
        r.call_mp_init(num_ref2);
        r.call_mp_init(res_num_ref);
        r.call_mp_read_radix(num_ref1, num_str_ref1);
        r.call_mp_read_radix(num_ref2, num_str_ref2);
        r.call_mp_add(num_ref1, num_ref2, res_num_ref);
        r.call_radix_size(res_num_ref, str_size_ref);
        let str_ref = r.call_malloc(str_size_ref);
        r.call_mp_toradix(str_ref, res_num_ref, res_str_ptr);
        let loaded = r.llvm.build_load(res_str_ptr);
        r.call_printf_func_by_value("Number: %s\n", loaded);
        r.call_printf_func("Googdbye from compiled to a file executable!\n", "");
    });
    runner.llvm.dump(out_name);
    if runner.llvm.mk_object_file(out_name) {
        link()
    } else {
        false
    }
}

struct LLVMRunner {
    llvm: LLVM,
    funcs: LLVMFuncs,
}

impl LLVMRunner {
    fn new() -> Self {
        let mut llvm = LLVM::new();
        let funcs = LLVMFuncs::new(&mut llvm);

        LLVMRunner { llvm, funcs }
    }

    fn mk_main_func(&mut self, f: fn(&mut LLVMRunner) -> ()) -> LLVMValueRef {
        let ret = self.llvm.void_t();
        let main_func_type = self.llvm.mk_func_type(ret, &mut vec![]);
        let main_func = self.llvm.mk_func("main", main_func_type);
        self.llvm.append_basic_block("entrypoint", main_func);

        f(self);

        self.llvm.ret_void();
        main_func
    }

    fn call_create_str(&mut self) {
        let func_name = "create_str";
        let i8_pt = self.llvm.ptr_t(self.llvm.i8_t());
        let create_type = self.llvm.mk_func_type(i8_pt, &mut vec![]);
        let create_func = self.llvm.mk_func(func_name, create_type);
        let res = self.llvm.call_func(func_name, create_func, &mut vec![]);
        self.call_printf_func("after calling to create str \n", "");
        self.call_printf_func_by_value("str value: %s\n", res);
    }

    fn call_create_i8(&mut self) {
        let func_name = "create_i8";
        let i8_t = self.llvm.i8_t();
        let create_type = self.llvm.mk_func_type(i8_t, &mut vec![]);
        let create_func = self.llvm.mk_func(func_name, create_type);
        let res = self.llvm.call_func(func_name, create_func, &mut vec![]);
        self.call_printf_func("after calling to create i8 \n", "");
        self.call_printf_func_by_value("i8 value: %d\n", res);
    }

    fn call_mp_init(&mut self, num: LLVMValueRef) {
        self.llvm
            .call_func("mp_init", self.funcs.mp_init, &mut vec![num]);
    }

    fn call_mp_read_radix(&mut self, num: LLVMValueRef, str_num: LLVMValueRef) {
        let const_10 = gen_const(&mut self.llvm, 10);
        self.llvm.call_func(
            "mp_read_radix",
            self.funcs.mp_read_radix,
            &mut vec![num, str_num, const_10],
        );
    }

    fn call_radix_size(&mut self, num: LLVMValueRef, str_size_ref: LLVMValueRef) -> LLVMValueRef {
        let const_10 = gen_const(&mut self.llvm, 10);
        self.llvm.call_func(
            "mp_radix_size",
            self.funcs.mp_radix_size,
            &mut vec![num, const_10, str_size_ref],
        )
    }

    fn call_malloc(&mut self, size: LLVMValueRef) -> LLVMValueRef {
        let sz = self.llvm.build_load(size);
        let from = self.llvm.i64_t();
        let extended = self.llvm.extend_32_to_64(sz, from);
        self.llvm
            .call_func("malloc", self.funcs.malloc, &mut vec![extended])
    }

    fn call_mp_toradix(
        &mut self,
        str_ptr: LLVMValueRef,
        num: LLVMValueRef,
        res_str: LLVMValueRef,
    ) -> LLVMValueRef {
        self.llvm.build_store(str_ptr, res_str);
        let loaded = self.llvm.build_load(res_str);
        let const_10 = gen_const(&mut self.llvm, 10);
        self.llvm.call_func(
            "mp_toradix",
            self.funcs.mp_toradix,
            &mut vec![num, loaded, const_10],
        )
    }

    fn call_mp_add(&mut self, num1: LLVMValueRef, num2: LLVMValueRef, res_num_ref: LLVMValueRef) {
        self.llvm.call_func(
            "mp_add",
            self.funcs.mp_add,
            &mut vec![num1, num2, res_num_ref],
        );
    }

    fn call_hello_world_function(&mut self) {
        let func_name = "hello_world";
        let ret = self.llvm.void_t();
        let hello_world_func_type = self.llvm.mk_func_type(ret, &mut vec![]);
        let hello_world_func = self.llvm.mk_func(func_name, hello_world_func_type);
        self.llvm
            .call_func(func_name, hello_world_func, &mut vec![]);
    }

    fn call_printf_func_by_value(&mut self, fmt: &str, value: LLVMValueRef) {
        let format_str = self.llvm.mk_global_string("format", fmt);
        let mut printf_args = vec![format_str, value];
        self.llvm
            .call_func("printf", self.funcs.printf, &mut printf_args);
    }

    fn call_printf_func(&mut self, fmt: &str, value: &str) {
        let format_str = self.llvm.mk_global_string("format", fmt);
        let world_str = self.llvm.mk_global_string("world", value);
        let mut printf_args = vec![format_str, world_str];
        self.llvm
            .call_func("printf", self.funcs.printf, &mut printf_args);
    }

    fn call_free(&mut self, addr: LLVMValueRef) {
        self.llvm
            .call_func("free", self.funcs.free, &mut vec![addr]);
    }

    fn call_test(&mut self) {
        let test_struct = self.llvm.struct_test();
        let func_type = self.llvm.mk_func_type(test_struct, &mut vec![]);
        let func = self.llvm.mk_func("create_test", func_type);
        let res = self.llvm.call_func("create_test", func, &mut vec![]);
        let field_ptr = self.llvm.get_struct_field_ptr(res, 1);
        let field_val = self.llvm.load_field_by_ptr(field_ptr);
        self.call_printf_func_by_value("create_test: %d\n", field_val);

        self.call_free(res);
    }

    fn call_slice(&mut self) {
        let array_test = self.llvm.array_test();
        let func_type = self.llvm.mk_func_type(array_test, &mut vec![]);
        let func = self.llvm.mk_func("create_slice", func_type);
        let res = self.llvm.call_func("create_slice", func, &mut vec![]);
        let field_ptr = self.llvm.get_struct_field_ptr(res, 0);
        let field_val = self.llvm.load_field_by_ptr(field_ptr);
        self.call_printf_func_by_value("create_array: %d\n", field_val);
        let field_ptr2 = self.llvm.get_struct_field_ptr(res, 1);
        let field_val2 = self.llvm.load_field_by_ptr(field_ptr2);
        self.call_printf_func_by_value("create_array: %d\n", field_val2);
        let field_ptr3 = self.llvm.get_struct_field_ptr(res, 2);
        let field_val3 = self.llvm.load_field_by_ptr(field_ptr3);
        self.call_printf_func_by_value("create_array: %d\n", field_val3);
    }

    fn call_hello_one(&mut self, name: &str) {
        let func_name = "hello_one";
        let mut argts = vec![self.llvm.ptr_t(self.llvm.i8_t())];
        let ret = self.llvm.void_t();
        let hello_one_type = self.llvm.mk_func_type(ret, &mut argts);
        let name = self.llvm.mk_global_string("name", name);
        let mut hello_one_args = vec![name];
        let hello_one_func = self.llvm.mk_func(func_name, hello_one_type);
        self.llvm
            .call_func(func_name, hello_one_func, &mut hello_one_args);
    }
}
