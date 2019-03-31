use llvm::execution_engine::{
    LLVMAddGlobalMapping,
    LLVMCreateExecutionEngineForModule,
    LLVMDisposeExecutionEngine,
    LLVMGetFunctionAddress,
    LLVMLinkInMCJIT,
    LLVMGetExecutionEngineTargetMachine,
};
use llvm::target::{
    LLVM_InitializeNativeAsmParser,
    LLVM_InitializeNativeAsmPrinter,
    LLVM_InitializeNativeTarget,
};
use llvm::target_machine::{
    LLVMTargetMachineRef,
    LLVMCreateTargetDataLayout,
};

use std::ffi::CStr;
use std::ptr;
use std::mem;
use std::ops::Drop;

use super::module::Module;
use super::llvm_ref::LlvmRef;
use super::value::AnyValue;

pub fn initialize_jit() {
    use std::process;

    unsafe {
        LLVMLinkInMCJIT();
        if LLVM_InitializeNativeTarget() == 1 {
            process::exit(1);
        }
        if LLVM_InitializeNativeAsmPrinter() == 1 {
            process::exit(1);
        }
        if LLVM_InitializeNativeAsmParser() == 1 {
            process::exit(1);
        }
    }
}

pub struct ExecutionEngine {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}

pub struct TargetMachine {
    pub(crate) ptr: LLVMTargetMachineRef,
}

pub struct TargetData {
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
            // TODO: Does this require LLVMDisposeMessage?
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

    pub fn target_machine(&self) -> TargetMachine {
        TargetMachine {
            ptr: unsafe {
                LLVMGetExecutionEngineTargetMachine(self.ptr)
            },
        }
    }
}

impl TargetMachine {
    pub fn create_data_layout(&mut self) -> TargetData {
        TargetData {
            ptr: unsafe {
                LLVMCreateTargetDataLayout(self.ptr)
            },
        }
    }
}
