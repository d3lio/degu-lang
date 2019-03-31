use llvm::core::{
    LLVMAppendBasicBlockInContext,
    LLVMGetBasicBlockParent,
    LLVMGetGlobalParent,
    LLVMGetModuleContext,
};
use llvm::prelude::LLVMBasicBlockRef;

use std::ffi::CStr;

use super::llvm_ref::LlvmRef;
use super::value::{AnyValue, Function};

pub struct BasicBlock {
    pub(crate) ptr: LLVMBasicBlockRef,
}

impl BasicBlock {
    pub fn new(name: &CStr, value: &mut Function) -> Self {
        unsafe {
            let module = LLVMGetGlobalParent(value.llvm_ref());
            let context = LLVMGetModuleContext(module);

            Self {
                ptr: LLVMAppendBasicBlockInContext(context, value.llvm_ref(), name.as_ptr()),
            }
        }
    }

    pub fn parent(&self) -> Function {
        Function {
            value: AnyValue {
                ptr: unsafe { LLVMGetBasicBlockParent(self.ptr) },
            }
        }
    }
}
