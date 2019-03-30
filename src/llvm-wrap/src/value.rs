use llvm::core::{
    LLVMCountParams,
    LLVMGetEntryBasicBlock,
    LLVMGetParams,
    LLVMSetValueName,
    LLVMPrintValueToString,
};

use std::ffi::CStr;
use std::fmt::{self, Debug, Formatter};

use super::basic_block::BasicBlock;
use super::llvm_ref::LlvmRef;

// TODO: improve this module to make a better use of Rust's type system to guard
// from invalid operations like integer add on floating point numbers and so on.

#[derive(Clone, PartialEq, Eq)]
pub struct AnyValue {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Function {
    pub(crate) value: AnyValue,
}

pub trait Value: Debug + LlvmRef {}

impl Value for AnyValue {}
impl Value for Function {}

impl Debug for AnyValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        unsafe {
            // TODO: Does this require LLVMDisposeMessage?
            let dump = LLVMPrintValueToString(self.ptr);
            write!(f, "{}", CStr::from_ptr(dump).to_str().unwrap())
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_value())
    }
}

impl AnyValue {
    pub fn set_name(&mut self, name: &CStr) {
        unsafe { LLVMSetValueName(self.ptr, name.as_ptr()) }
    }
}

impl Function {
    pub fn as_value(&self) -> &AnyValue {
        &self.value
    }

    pub fn to_value(self) -> AnyValue {
        self.value
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
