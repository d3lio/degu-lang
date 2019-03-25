use libc::c_uint;

use llvm::core::{
    LLVMContextCreate,
    LLVMContextDispose,
    LLVMDoubleTypeInContext,
    LLVMFP128TypeInContext,
    LLVMFloatTypeInContext,
    LLVMFunctionType,
    LLVMInt128TypeInContext,
    LLVMInt16TypeInContext,
    LLVMInt32TypeInContext,
    LLVMInt64TypeInContext,
    LLVMInt8TypeInContext,
    LLVMVoidTypeInContext,
};
use llvm::prelude::LLVMBool;

use std::ffi::CStr;
use std::ops::Drop;

use super::builder::Builder;
use super::llvm_ref::LlvmRef;
use super::module::Module;
use super::types::Type;

pub struct Context {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.ptr);
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            ptr: unsafe { LLVMContextCreate() },
        }
    }

    pub fn create_builder(&mut self) -> Builder {
        Builder::new(self)
    }

    pub fn create_module(&mut self,
    name: &CStr) -> Module {
        Module::new(name, self)
    }
}

macro_rules! impl_basic_types {
    ($($name:ident => $f:ident),*$(,)?) => {
        impl Context {$(
            pub fn $name(&self) -> Type {
                unsafe {
                    Type {
                        ptr: $f(self.ptr)
                    }
                }
            }
        )*}
    };
}

impl_basic_types! {
    void_type => LLVMVoidTypeInContext,
    i8_type => LLVMInt8TypeInContext,
    i16_type => LLVMInt16TypeInContext,
    i32_type => LLVMInt32TypeInContext,
    i64_type => LLVMInt64TypeInContext,
    i128_type => LLVMInt128TypeInContext,
    f32_type => LLVMFloatTypeInContext,
    f64_type => LLVMDoubleTypeInContext,
    f128_type => LLVMFP128TypeInContext,
}

impl Context {
    // The context is determined by the return type's context.
    // We don't need to keep the slice because its elements are cloned internally.
    pub fn function_type(ret: Type, args: &[Type], is_argv: bool) -> Type {
        unsafe {
            Type {
                ptr: LLVMFunctionType(
                    ret.ptr,
                    args.as_ptr() as *mut _,
                    args.len() as c_uint,
                    is_argv as LLVMBool,
                ),
            }
        }
    }
}
