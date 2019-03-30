use llvm::analysis::{
    LLVMVerifierFailureAction,
    LLVMVerifyFunction,
    LLVMVerifyModule,
};
use llvm::core::LLVMDisposeMessage;

use std::ffi::CStr;
use std::ptr;

use super::llvm_ref::LlvmRef;
use super::module::Module;
use super::value::Function;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VerifierFailureAction {
    /// Print to stderr and abort the process.
    AbortProcessAction,
    /// Print to stderr and return true.
    PrintMessageAction,
    /// Return true and print nothing.
    ReturnStatusAction,
}

impl VerifierFailureAction {
    fn map_to_llvm(self) -> LLVMVerifierFailureAction {
        match self {
            AbortProcessAction => LLVMAbortProcessAction,
            PrintMessageAction => LLVMPrintMessageAction,
            ReturnStatusAction => LLVMReturnStatusAction,
        }
    }
}

use LLVMVerifierFailureAction::*;
use VerifierFailureAction::*;

pub fn verify_function(f: &Function, action: VerifierFailureAction) -> bool {
    unsafe {
        LLVMVerifyFunction(f.llvm_ref(), action.map_to_llvm()) != 0
    }
}

// TODO: think of a convenient way to use Result
pub fn verify_module(m: &Module, action: VerifierFailureAction) -> (bool, String) {
    let msg = &mut ptr::null_mut();
    unsafe {
        let res = LLVMVerifyModule(m.llvm_ref(), action.map_to_llvm(), msg) != 0;
        let message = CStr::from_ptr(*msg).to_str().unwrap().to_string();
        LLVMDisposeMessage(*msg);
        (res, message)
    }
}
