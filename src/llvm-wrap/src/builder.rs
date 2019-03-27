use libc::c_uint;

use llvm::core::{
    LLVMBuildAdd,
    LLVMBuildCall,
    LLVMBuildFAdd,
    LLVMBuildFMul,
    LLVMBuildFSub,
    LLVMBuildMul,
    LLVMBuildRet,
    LLVMBuildRetVoid,
    LLVMBuildSub,
    LLVMConstInt,
    LLVMConstReal,
    LLVMCreateBuilderInContext,
    LLVMDisposeBuilder,
    LLVMPositionBuilderAtEnd,
};
use llvm::prelude::LLVMBool;

use std::ffi::CStr;
use std::ops::Drop;

use super::basic_block::BasicBlock;
use super::context::Context;
use super::llvm_ref::LlvmRef;
use super::types::Type;
use super::util::EMPTY_C_STR;
use super::value::{AnyValue, Function};

pub struct Builder {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}

#[derive(Debug)]
pub enum BuilderError {
    WrongArgumentsCount {
        expected: usize,
        actual: usize,
    },
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.ptr);
        }
    }
}

macro_rules! impl_bin_op {
    ($($name:ident => $llvm_op:ident),*$(,)?) => {
        $(pub fn $name(&mut self, a: &AnyValue, b: &AnyValue, name: Option<&CStr>) -> AnyValue {
            AnyValue {
                ptr: unsafe {
                    $llvm_op(
                        self.ptr,
                        a.llvm_ref(),
                        b.llvm_ref(),
                        name.map_or(EMPTY_C_STR, CStr::as_ptr),
                    )
                },
            }
        })*
    };
}

impl Builder {
    pub(crate) fn new(context: &Context) -> Self {
        Self {
            ptr: unsafe { LLVMCreateBuilderInContext(context.llvm_ref()) },
        }
    }

    pub fn position_at_end(&mut self, bb: &BasicBlock) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.ptr, bb.llvm_ref());
        }
    }

    pub fn build_const_int(&mut self, ty: Type, value: u64, signed: bool) -> AnyValue {
        unsafe {
            AnyValue {
                ptr: LLVMConstInt(ty.llvm_ref(), value, signed as LLVMBool)
            }
        }
    }

    pub fn build_const_fp(&mut self, ty: Type, value: f64) -> AnyValue {
        unsafe {
            AnyValue {
                ptr: LLVMConstReal(ty.llvm_ref(), value)
            }
        }
    }
    pub fn build_ret_void(&mut self) -> AnyValue {
        unsafe {
            AnyValue {
                ptr: LLVMBuildRetVoid(self.ptr),
            }
        }
    }

    pub fn build_ret(&mut self, value: &AnyValue) -> AnyValue {
        unsafe {
            AnyValue {
                ptr: LLVMBuildRet(self.ptr, value.llvm_ref()),
            }
        }
    }

    pub fn build_call(
        &mut self,
        f: &Function,
        args: &[AnyValue],
        name: Option<&CStr>) -> Result<AnyValue, BuilderError>
    {
        let args_count = args.len();
        let params_count = f.params().len();

        if args_count != params_count {
            return Err(BuilderError::WrongArgumentsCount {
                expected: params_count,
                actual: args_count,
            });
        }

        let args = args.into_iter().map(LlvmRef::llvm_ref).collect::<Vec<_>>();
        Ok(AnyValue {
            ptr: unsafe {
                LLVMBuildCall(
                    self.ptr,
                    f.llvm_ref(),
                    args.as_ptr() as *mut _,
                    args.len() as c_uint,
                    name.map_or(EMPTY_C_STR, CStr::as_ptr),
                )
            },
        })
    }

    impl_bin_op!{
        build_add => LLVMBuildAdd,
        build_sub => LLVMBuildSub,
        build_mul => LLVMBuildMul,
    }

    impl_bin_op!{
        build_fp_add => LLVMBuildFAdd,
        build_fp_sub => LLVMBuildFSub,
        build_fp_mul => LLVMBuildFMul,
    }
}
