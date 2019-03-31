use libc::c_uint;

use llvm::LLVMRealPredicate;
use llvm::core::{
    LLVMBuildAdd,
    LLVMBuildBr,
    LLVMBuildCall,
    LLVMBuildCondBr,
    LLVMBuildFAdd,
    LLVMBuildFCmp,
    LLVMBuildFMul,
    LLVMBuildFSub,
    LLVMBuildMul,
    LLVMBuildPhi,
    LLVMBuildRet,
    LLVMBuildRetVoid,
    LLVMBuildSub,
    LLVMBuildUIToFP,
    LLVMConstInt,
    LLVMConstReal,
    LLVMCreateBuilderInContext,
    LLVMDisposeBuilder,
    LLVMGetInsertBlock,
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
use super::value::{AnyValue, Function, Phi};

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

// TODO: Expand the abbreviations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RealPredicate {
    False,
    OEQ,
    OGT,
    OGE,
    OLT,
    OLE,
    ONE,
    ORD,
    UNO,
    UEQ,
    UGT,
    UGE,
    ULT,
    ULE,
    UNE,
    True,
}

impl RealPredicate {
    fn to_llvm(self) -> LLVMRealPredicate {
        use LLVMRealPredicate::*;
        use RealPredicate::*;

        match self {
            False => LLVMRealPredicateFalse,
            OEQ => LLVMRealOEQ,
            OGT => LLVMRealOGT,
            OGE => LLVMRealOGE,
            OLT => LLVMRealOLT,
            OLE => LLVMRealOLE,
            ONE => LLVMRealONE,
            ORD => LLVMRealORD,
            UNO => LLVMRealUNO,
            UEQ => LLVMRealUEQ,
            UGT => LLVMRealUGT,
            UGE => LLVMRealUGE,
            ULT => LLVMRealULT,
            ULE => LLVMRealULE,
            UNE => LLVMRealUNE,
            True => LLVMRealPredicateTrue,
        }
    }
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
    pub(crate) fn new(context: &mut Context) -> Self {
        Self {
            ptr: unsafe { LLVMCreateBuilderInContext(context.llvm_ref()) },
        }
    }

    pub fn get_insert_block(&self) -> BasicBlock {
        BasicBlock {
            ptr: unsafe { LLVMGetInsertBlock(self.ptr) },
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

    pub fn build_cast_uint_to_fp(
        &mut self,
        value: AnyValue,
        ty: Type,
        name: Option<&CStr>) -> AnyValue
    {
        AnyValue {
            ptr: unsafe {
                LLVMBuildUIToFP(
                    self.ptr,
                    value.llvm_ref(),
                    ty.llvm_ref(),
                    name.map_or(EMPTY_C_STR, CStr::as_ptr),
                )
            }
        }
    }

    pub fn build_br(&mut self, dest: &BasicBlock) -> AnyValue {
        AnyValue {
            ptr: unsafe { LLVMBuildBr(self.ptr, dest.llvm_ref()) },
        }
    }

    pub fn build_cond_br(&mut self, cond: &AnyValue, then: &BasicBlock, el: &BasicBlock) -> AnyValue {
        AnyValue {
            ptr: unsafe {
                LLVMBuildCondBr(self.ptr, cond.llvm_ref(), then.llvm_ref(), el.llvm_ref())
            },
        }
    }

    pub fn build_phi(&mut self, ty: Type, name: Option<&CStr>) -> Phi {
        Phi {
            value: AnyValue {
                ptr: unsafe {
                    LLVMBuildPhi(self.ptr, ty.llvm_ref(), name.map_or(EMPTY_C_STR, CStr::as_ptr))
                },
            }
        }
    }

    pub fn build_fp_cmp(
        &mut self,
        op: RealPredicate,
        a: &AnyValue,
        b: &AnyValue,
        name: Option<&CStr>) -> AnyValue
    {
        AnyValue {
            ptr: unsafe {
                LLVMBuildFCmp(
                    self.ptr,
                    op.to_llvm(),
                    a.llvm_ref(),
                    b.llvm_ref(),
                    name.map_or(EMPTY_C_STR, CStr::as_ptr),
                )
            },
        }
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
