use llvm::core::{
    LLVMCreateFunctionPassManagerForModule,
    LLVMDisposePassManager,
    LLVMInitializeFunctionPassManager,
    LLVMRunFunctionPassManager,
};
use llvm::prelude::LLVMPassManagerRef;
use llvm::transforms::scalar::{
    LLVMAddCFGSimplificationPass,
    LLVMAddGVNPass,
    LLVMAddInstructionCombiningPass,
    LLVMAddReassociatePass,
};

use std::ops::Drop;

use super::llvm_ref::LlvmRef;
use super::module::Module;
use super::value::Function;

pub struct FunctionPassManager {
    pub(crate) ptr: LLVMPassManagerRef,
}

pub struct FunctionPassManagerBuilder {
    pub(crate) ptr: LLVMPassManagerRef,
    done: bool,
}

impl Drop for FunctionPassManager {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposePassManager(self.ptr);
        }
    }
}

impl Drop for FunctionPassManagerBuilder {
    fn drop(&mut self) {
        if !self.done {
            unsafe {
                LLVMDisposePassManager(self.ptr);
            }
        }
    }
}

impl FunctionPassManager {
    /// Runs the pass manager on a function.
    ///
    /// Returns true if any of the passes modified the function.
    pub fn run(&self, f: &mut Function) -> bool {
        unsafe {
            LLVMRunFunctionPassManager(self.ptr, f.llvm_ref()) != 0
        }
    }
}

impl FunctionPassManagerBuilder {
    pub(crate) fn new(m: &mut Module) -> Self {
        Self {
            ptr: unsafe { LLVMCreateFunctionPassManagerForModule(m.llvm_ref()) },
            done: false,
        }
    }

    /// Do simple "peephole" and bit-twiddling optimizations.
    pub fn add_instruction_combination_pass(self) -> Self {
        unsafe { LLVMAddInstructionCombiningPass(self.ptr) }
        self
    }

    /// Reassociate expressions.
    pub fn add_reassociate_pass(self) -> Self {
        unsafe { LLVMAddReassociatePass(self.ptr) }
        self
    }

    /// Eliminate Common SubExpressions.
    pub fn add_gvn_pass(self) -> Self {
        unsafe { LLVMAddGVNPass(self.ptr) }
        self
    }

    /// Simplify the control flow graph (deleting unreachable blocks, etc).
    pub fn add_cfg_simplification_pass(self) -> Self {
        unsafe { LLVMAddCFGSimplificationPass(self.ptr) }
        self
    }

    pub fn build(mut self) -> FunctionPassManager {
        self.done = true;
        unsafe {
            LLVMInitializeFunctionPassManager(self.ptr);
        }

        FunctionPassManager {
            ptr: self.ptr,
        }
    }
}
