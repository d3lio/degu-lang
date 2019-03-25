use llvm::core::{LLVMAppendBasicBlockInContext, LLVMGetGlobalParent, LLVMGetModuleContext};
use llvm::prelude::LLVMBasicBlockRef;

use std::ffi::CStr;

use super::llvm_ref::LlvmRef;
use super::value::Function;

pub struct BasicBlock {
    pub(crate) ptr: LLVMBasicBlockRef,
}

impl BasicBlock {
    pub fn create_and_append(name: &CStr, value: &mut Function) -> Self {
        unsafe {
            let module = LLVMGetGlobalParent(value.llvm_ref());
            let context = LLVMGetModuleContext(module);

            Self {
                ptr: LLVMAppendBasicBlockInContext(context, value.llvm_ref(), name.as_ptr()),
            }
        }
    }
}
