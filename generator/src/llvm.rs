extern crate llvm_sys;

use self::llvm_sys::core::*;
use self::llvm_sys::execution_engine::*;
use self::llvm_sys::prelude::*;
use self::llvm_sys::target::*;
use self::llvm_sys::target_machine::*;
use llvm::llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm::llvm_sys::analysis::LLVMVerifyModule;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::path::Path;
use std::process::Command;
use std::ptr;
use std::ptr::null_mut;

macro_rules! empty_mut_c_str {
    ($s:expr) => {
        "\0".as_ptr() as *mut i8
    };
}

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

pub struct CStrOwner {
    strings: Vec<CString>,
}

pub struct LLVM {
    pub context: LLVMContextRef,
    pub module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    pub cstr_owner: CStrOwner,
}

pub struct LLVMFuncs {
    pub printf: LLVMValueRef,
    pub scanf: LLVMValueRef,
    pub free: LLVMValueRef,
    pub malloc: LLVMValueRef,
    pub mp_init: LLVMValueRef,
    pub mp_read_radix: LLVMValueRef,
    pub mp_radix_size: LLVMValueRef,
    pub mp_toradix: LLVMValueRef,
    pub mp_add: LLVMValueRef,
}

pub struct LLVMStructs {
    pub mp_struct: LLVMTypeRef,
}

impl LLVMStructs {
    pub fn new(llvm: &mut LLVM) -> Self {
        LLVMStructs {
            mp_struct: export_mp_struct(llvm),
        }
    }
}

fn export_mp_struct(llvm: &mut LLVM) -> LLVMTypeRef {
    unsafe {
        let ns = LLVMStructCreateNamed(llvm.context, llvm.cstr_owner.new_str_ptr("mp_struct"));
        LLVMStructSetBody(
            ns,
            [
                llvm.i32_t(),
                llvm.i32_t(),
                llvm.i32_t(),
                llvm.ptr_t(llvm.i64_t()),
            ]
            .as_mut_ptr(),
            4,
            LLVM_FALSE,
        );
        ns
    }
}

impl CStrOwner {
    fn new() -> Self {
        CStrOwner { strings: vec![] }
    }

    pub fn new_str_ptr(&mut self, s: &str) -> *mut i8 {
        let cstring = CString::new(s).unwrap();
        let ptr = cstring.as_ptr() as *mut _;
        self.strings.push(cstring);
        ptr
    }
}

impl LLVM {
    pub fn new() -> Self {
        println!("Initializing LLVM");
        unsafe {
            let context = LLVMContextCreate();
            let mut cstr_owner = CStrOwner::new();
            let module =
                LLVMModuleCreateWithNameInContext(cstr_owner.new_str_ptr("module"), context);
            let builder = LLVMCreateBuilderInContext(context);
            LLVM {
                context,
                module,
                builder,
                cstr_owner,
            }
        }
    }

    pub fn void_t(&self) -> LLVMTypeRef {
        unsafe { LLVMVoidTypeInContext(self.context) }
    }

    pub fn i8_t(&self) -> LLVMTypeRef {
        unsafe { LLVMInt8TypeInContext(self.context) }
    }

    pub fn arr_t(&self, t: LLVMTypeRef, cnt: ::libc::c_uint) -> LLVMTypeRef {
        unsafe { LLVMArrayType(t, cnt) }
    }

    pub fn ptr_t(&self, t: LLVMTypeRef) -> LLVMTypeRef {
        unsafe { LLVMPointerType(t, 0) }
    }

    pub fn i32_t(&self) -> LLVMTypeRef {
        unsafe { LLVMInt32TypeInContext(self.context) }
    }

    pub fn i64_t(&self) -> LLVMTypeRef {
        unsafe { LLVMInt64TypeInContext(self.context) }
    }

    pub fn struct_test(&self) -> LLVMTypeRef {
        unsafe {
            LLVMPointerType(
                LLVMStructTypeInContext(
                    self.context,
                    [self.i32_t(), self.i32_t()].as_mut_ptr(),
                    2,
                    LLVM_FALSE,
                ),
                0,
            )
        }
    }

    pub fn array_test(&self) -> LLVMTypeRef {
        unsafe {
            LLVMPointerType(
                LLVMStructTypeInContext(
                    self.context,
                    [self.i32_t(), self.i32_t(), self.i32_t()].as_mut_ptr(),
                    3,
                    LLVM_FALSE,
                ),
                0,
            )
        }
    }

    pub fn dump(&self, name: &str) {
        let file_name = format!("./target/{}.ll", name);
        println!("Dumping LLVM IR to the file: {}", file_name);
        if Path::new(&file_name).exists() {
            match fs::remove_file(&file_name) {
                Err(e) => println!(
                    "The file '{}' can't be removed because of the error: {}",
                    file_name, e
                ),
                Ok(_) => {
                    writing_dump(&file_name, self.module);
                }
            }
        } else {
            writing_dump(&file_name, self.module);
        }
    }

    pub fn get_struct_field_ptr(&mut self, struct_ref: LLVMValueRef, index: u32) -> LLVMValueRef {
        unsafe {
            LLVMBuildStructGEP(
                self.builder,
                struct_ref,
                index,
                self.cstr_owner.new_str_ptr(&format!("idx{}", index)),
            )
        }
    }
    pub fn load_field_by_ptr(&mut self, field_ptr: LLVMValueRef) -> LLVMValueRef {
        unsafe {
            LLVMBuildLoad(
                self.builder,
                field_ptr,
                self.cstr_owner.new_str_ptr("field"),
            )
        }
    }

    pub fn extend_32_to_64(&mut self, value: LLVMValueRef, dest_type: LLVMTypeRef) -> LLVMValueRef {
        unsafe {
            LLVMBuildSExt(
                self.builder,
                value,
                dest_type,
                self.cstr_owner.new_str_ptr("extended"),
            )
        }
    }

    pub fn build_alloca(&mut self, name: &str, type_ref: LLVMTypeRef) -> LLVMValueRef {
        unsafe { LLVMBuildAlloca(self.builder, type_ref, self.cstr_owner.new_str_ptr(name)) }
    }

    pub fn build_load(&mut self, struct_ref: LLVMValueRef) -> LLVMValueRef {
        unsafe {
            LLVMBuildLoad(
                self.builder,
                struct_ref,
                self.cstr_owner.new_str_ptr("struct"),
            )
        }
    }

    pub fn build_store(&mut self, value: LLVMValueRef, ptr: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildStore(self.builder, value, ptr) }
    }

    pub fn mk_func(&mut self, name: &str, function_type: LLVMTypeRef) -> LLVMValueRef {
        unsafe {
            LLVMAddFunction(
                self.module,
                self.cstr_owner.new_str_ptr(name),
                function_type,
            )
        }
    }
    pub fn mk_global_string(&mut self, name: &str, value: &str) -> LLVMValueRef {
        unsafe {
            LLVMBuildGlobalString(
                self.builder,
                self.cstr_owner.new_str_ptr(value),
                (&mut self.cstr_owner).new_str_ptr(name),
            )
        }
    }
    pub fn mk_func_type(
        &mut self,
        return_type: LLVMTypeRef,
        args_type: &mut [LLVMTypeRef],
    ) -> LLVMTypeRef {
        unsafe {
            let param_types = match args_type.len() {
                0 => ptr::null_mut(),
                _ => args_type.as_mut_ptr(),
            };
            LLVMFunctionType(return_type, param_types, args_type.len() as u32, LLVM_FALSE)
        }
    }
    pub fn mk_func_type_varargs(
        &mut self,
        return_type: LLVMTypeRef,
        args_type: &mut [LLVMTypeRef],
    ) -> LLVMTypeRef {
        unsafe {
            let param_types = match args_type.len() {
                0 => ptr::null_mut(),
                _ => args_type.as_mut_ptr(),
            };
            LLVMFunctionType(return_type, param_types, args_type.len() as u32, LLVM_TRUE)
        }
    }
    pub fn call_func(
        &mut self,
        name: &str,
        func: LLVMValueRef,
        call_args: &mut Vec<LLVMValueRef>,
    ) -> LLVMValueRef {
        let args = match call_args.len() {
            0 => ptr::null_mut(),
            _ => call_args.as_mut_ptr(),
        };
        unsafe {
            LLVMBuildCall(
                self.builder,
                func,
                args,
                call_args.len() as u32,
                self.cstr_owner.new_str_ptr(name),
            )
        }
    }
    pub fn append_basic_block(&mut self, name: &str, function: LLVMValueRef) {
        unsafe {
            let block = LLVMAppendBasicBlockInContext(
                self.context,
                function,
                self.cstr_owner.new_str_ptr(name),
            );
            LLVMPositionBuilderAtEnd(self.builder, block);
        }
    }
    pub fn ret_void(&mut self) {
        unsafe {
            LLVMBuildRetVoid(self.builder);
        }
    }
    pub fn mk_object_file(&mut self, name: &str) -> bool {
        unsafe {
            println!("initializing LLVM to generate object file\n");
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargets();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllAsmParsers();
            LLVM_InitializeAllAsmPrinters();

            let mut module_verification_error = empty_mut_c_str!("");
            LLVMVerifyModule(
                self.module,
                LLVMVerifierFailureAction::LLVMPrintMessageAction,
                &mut module_verification_error,
            );

            let triple = LLVMGetDefaultTargetTriple();
            println!("Triple: {:?}", from_c(triple));
            let cpu = LLVMGetHostCPUName();
            println!("CPU: {:?}", from_c(cpu));
            let features = LLVMGetHostCPUFeatures();
            println!("Features: {:?}", from_c(features));

            LLVMSetTarget(self.module, triple);

            let mut target = LLVMGetFirstTarget();
            println!("{:?}", from_c(LLVMGetTargetName(target)));

            let mut getting_target_error = empty_mut_c_str!("");

            if LLVMGetTargetFromTriple(triple, &mut target, &mut getting_target_error) == 1 {
                panic!("can't get target");
            }

            let getting_target_err_str = from_c(getting_target_error);

            if !getting_target_err_str.is_empty() {
                println!("Error getting target: {}", getting_target_err_str);
            }
            println!("creating target machine");
            let target_machine = LLVMCreateTargetMachine(
                target,
                triple,
                cpu,
                features,
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelDefault,
            );

            LLVMCreateTargetDataLayout(target_machine);

            let file_name = format!("./target/{}.o\0", name).as_ptr() as *mut i8;
            println!("file name = {}", from_c(file_name));

            let mut error_emitting_obj = empty_mut_c_str!("");
            println!("creating object file");

            LLVMTargetMachineEmitToFile(
                target_machine,
                self.module,
                file_name,
                LLVMCodeGenFileType::LLVMObjectFile,
                &mut error_emitting_obj,
            );

            let emitting_obj_err_str = from_c(error_emitting_obj);

            LLVMDisposeTargetMachine(target_machine);

            if !emitting_obj_err_str.is_empty() {
                println!("ERROR generating file: {}", emitting_obj_err_str);
                false
            } else {
                true
            }
        }
    }
    pub fn exec_func(&mut self, func: LLVMValueRef) -> bool {
        unsafe {
            let mut ee = mem::uninitialized();
            LLVMLinkInMCJIT();
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();

            let mut module_verification_error = empty_mut_c_str!("");
            LLVMVerifyModule(
                self.module,
                LLVMVerifierFailureAction::LLVMReturnStatusAction,
                &mut module_verification_error,
            );

            let mut getting_target_error = empty_mut_c_str!("");
            LLVMCreateExecutionEngineForModule(&mut ee, self.module, &mut getting_target_error);

            let emitting_obj_err_str = from_c(getting_target_error);

            if !emitting_obj_err_str.is_empty() {
                println!("ERROR generating file: {}", emitting_obj_err_str);
                false
            } else {
                println!("running main");
                self.dump("output");
                LLVMRunFunction(ee, func, 0, null_mut());
                // LLVMRunFunctionAsMain(ee, main, 0, null_mut(), null_mut());
                true
            }
        }
    }
}

impl LLVMFuncs {
    pub fn new(llvm: &mut LLVM, llvm_structs: &LLVMStructs) -> Self {
        LLVMFuncs {
            printf: export_printf_func(llvm),
            scanf: export_scanf_func(llvm),
            free: export_free_func(llvm),
            malloc: export_malloc_func(llvm),
            mp_init: export_mp_init_func(llvm, llvm_structs),
            mp_read_radix: export_mp_read_radix(llvm, llvm_structs),
            mp_radix_size: export_mp_radix_size(llvm, llvm_structs),
            mp_toradix: export_mp_to_radix(llvm, llvm_structs),
            mp_add: export_mp_add(llvm, llvm_structs),
        }
    }
}

pub fn export_mp_add(llvm: &mut LLVM, llvm_structs: &LLVMStructs) -> LLVMValueRef {
    let mp_s_p = llvm.ptr_t(llvm_structs.mp_struct);
    let mut args = [mp_s_p, mp_s_p, mp_s_p];
    let ret = llvm.i32_t();
    let func_type = llvm.mk_func_type(ret, &mut args);
    llvm.mk_func("mp_add", func_type)
}

pub fn export_mp_to_radix(llvm: &mut LLVM, llvm_structs: &LLVMStructs) -> LLVMValueRef {
    let mut args = [
        llvm.ptr_t(llvm_structs.mp_struct),
        llvm.ptr_t(llvm.i8_t()),
        llvm.i32_t(),
    ];
    let ret = llvm.i32_t();
    let mp_toradix_type = llvm.mk_func_type(ret, &mut args);
    llvm.mk_func("mp_toradix", mp_toradix_type)
}

pub fn export_mp_radix_size(llvm: &mut LLVM, llvm_structs: &LLVMStructs) -> LLVMValueRef {
    let mut args = [
        llvm.ptr_t(llvm_structs.mp_struct),
        llvm.i32_t(),
        llvm.ptr_t(llvm.i32_t()),
    ];
    let ret = llvm.i32_t();
    let mp_read_radix = llvm.mk_func_type(ret, &mut args);
    llvm.mk_func("mp_radix_size", mp_read_radix)
}

pub fn export_mp_read_radix(llvm: &mut LLVM, llvm_structs: &LLVMStructs) -> LLVMValueRef {
    let func_name2 = "mp_read_radix";
    let mut args = [
        llvm.ptr_t(llvm_structs.mp_struct),
        llvm.ptr_t(llvm.i8_t()),
        llvm.i32_t(),
    ];
    let ret = llvm.i32_t();
    let mp_read_radix = llvm.mk_func_type(ret, &mut args);
    llvm.mk_func(func_name2, mp_read_radix)
}

pub fn export_mp_init_func(llvm: &mut LLVM, llvm_structs: &LLVMStructs) -> LLVMValueRef {
    let func_name = "mp_init";
    let ret = llvm.i32_t();
    let mp_sp = llvm.ptr_t(llvm_structs.mp_struct);
    let create_bigint = llvm.mk_func_type(ret, &mut [mp_sp]);
    llvm.mk_func(func_name, create_bigint)
}

pub fn gen_const(llvm: &mut LLVM, v: u64) -> LLVMValueRef {
    unsafe { LLVMConstInt(llvm.i32_t(), v, 0) }
}

fn export_printf_func(llvm: &mut LLVM) -> LLVMValueRef {
    let mut argts = [llvm.ptr_t(llvm.i8_t())];
    let ret = llvm.i32_t();
    let printf_type = llvm.mk_func_type_varargs(ret, &mut argts);
    llvm.mk_func("printf", printf_type)
}

fn export_scanf_func(llvm: &mut LLVM) -> LLVMValueRef {
    let mut argts = [llvm.ptr_t(llvm.i8_t())];
    let ret = llvm.i32_t();
    let printf_type = llvm.mk_func_type_varargs(ret, &mut argts);
    llvm.mk_func("scanf", printf_type)
}

fn export_free_func(llvm: &mut LLVM) -> LLVMValueRef {
    let mut argts = [llvm.ptr_t(llvm.i8_t())];
    let ret = llvm.void_t();
    let func_type = llvm.mk_func_type(ret, &mut argts);
    llvm.mk_func("free", func_type)
}

fn export_malloc_func(llvm: &mut LLVM) -> LLVMValueRef {
    let mut argts = [llvm.i64_t()];
    let ret = llvm.ptr_t(llvm.i8_t());
    let func_type = llvm.mk_func_type(ret, &mut argts);
    llvm.mk_func("malloc", func_type)
}

impl Drop for LLVM {
    fn drop(&mut self) {
        println!("shutting down LLVM...");
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}

fn from_c(c_str: *const i8) -> String {
    let c_str: &CStr = unsafe { CStr::from_ptr(c_str) };
    let s = c_str.to_str().unwrap();
    s.to_owned()
}

#[cfg(target_os = "macos")]
fn lib_ext() -> String {
    "dylib".to_owned()
}

pub fn link() -> bool {
    let cc = Command::new("cc")
        .arg("./target/output.o")
        .arg(format!(
            "../test-lib/target/debug/libtest_lib.{}",
            lib_ext()
        ))
        .arg("/Users/jinnzest/Documents/nulljinn/tests/libtommath/libtommath.a")
        .arg("-o")
        .arg("./target/out")
        .output()
        .expect("");
    println!("status: {}", cc.status);
    println!("stdout: {}", String::from_utf8_lossy(&cc.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&cc.stderr));
    if cc.stderr.is_empty() {
        true
    } else {
        false
    }
}

fn writing_dump(file_name: &str, module: LLVMModuleRef) {
    unsafe {
        let llvm_ir_ptr = LLVMPrintModuleToString(module);
        let llvm_ir = CStr::from_ptr(llvm_ir_ptr as *const _);
        match File::create(&file_name) {
            Ok(mut f) => match f.write_all(llvm_ir.to_bytes()) {
                Ok(_) => {}
                Err(e) => println!(
                    "The file '{}' can't be written because of the error: {}",
                    file_name, e
                ),
            },
            Err(e) => println!(
                "The file '{}' can't be created because of the error: {}",
                file_name, e
            ),
        }
    }
}
