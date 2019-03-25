use llvm::core::{LLVMCountParams, LLVMGetEntryBasicBlock, LLVMGetParams, LLVMSetValueName};

use std::ffi::CStr;

use super::basic_block::BasicBlock;
use super::llvm_ref::LlvmRef;

#[derive(PartialEq, Eq)]
pub struct AnyValue {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}

#[derive(PartialEq, Eq)]
pub struct Function {
    pub(crate) value: AnyValue,
}

pub trait Value: LlvmRef {}

impl Value for AnyValue {}
impl Value for Function {}

impl AnyValue {
    pub fn set_name(&mut self, name: &CStr) {
        unsafe { LLVMSetValueName(self.ptr, name.as_ptr()) }
    }
}

impl Function {
    pub fn as_value(&self) -> &AnyValue {
        &self.value
    }

    pub fn entry_block(&self) -> Option<BasicBlock> {
        // TODO: check if it exists
        unsafe {
            Some(BasicBlock {
                ptr: LLVMGetEntryBasicBlock(self.llvm_ref()),
            })
        }
    }

    pub fn params(&self) -> Vec<AnyValue> {
        unsafe {
            let count = LLVMCountParams(self.llvm_ref()) as usize;
            let mut storage = Vec::with_capacity(count);
            LLVMGetParams(self.llvm_ref(), storage[..].as_mut_ptr());
            storage.set_len(count);
            // TODO: is transmute going to optimize this?
            storage
                .into_iter()
                .map(|ptr| AnyValue { ptr })
                .collect::<Vec<_>>()
        }
    }
}
