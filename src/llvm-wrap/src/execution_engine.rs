use llvm::execution_engine::{
    LLVMAddGlobalMapping,
    LLVMCreateExecutionEngineForModule,
    LLVMDisposeExecutionEngine,
    LLVMGetFunctionAddress,
};

use std::ffi::CStr;
use std::ptr;
use std::mem;
use std::ops::Drop;

use super::module::Module;
use super::llvm_ref::LlvmRef;
use super::value::AnyValue;

pub struct ExecutionEngine {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}

impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeExecutionEngine(self.ptr);
        }
    }
}

impl ExecutionEngine {
    pub fn new(module: Module) -> Result<Self, String> {
        let ee = &mut ptr::null_mut();
        let err = &mut ptr::null_mut();

        let result = unsafe {
            LLVMCreateExecutionEngineForModule(ee, module.llvm_ref(), err)
        };

        // The execution engine consumes the module so we must avoid disposing it.
        mem::forget(module);

        if result == 0 {
            Ok(Self { ptr: *ee })
        } else {
            unsafe {
                Err(CStr::from_ptr(*err).to_str().unwrap().to_string())
            }
        }
    }

    pub fn function_address(&self, name: &CStr) -> usize {
        unsafe {
            LLVMGetFunctionAddress(self.ptr, name.as_ptr()) as usize
        }
    }

    pub unsafe fn add_global_mapping(&mut self, value: &AnyValue, addr: usize) {
        LLVMAddGlobalMapping(self.ptr, value.llvm_ref(), mem::transmute(addr));
    }
}
