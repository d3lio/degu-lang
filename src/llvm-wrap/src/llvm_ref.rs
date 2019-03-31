use llvm::execution_engine::LLVMExecutionEngineRef;
use llvm::prelude::*;
use llvm::target::LLVMTargetDataRef;

use super::basic_block::BasicBlock;
use super::builder::Builder;
use super::context::Context;
use super::execution_engine::{ExecutionEngine, TargetData};
use super::module::Module;
use super::transformation::FunctionPassManagerBuilder;
use super::types::Type;
use super::value::{AnyValue, Function};

pub trait LlvmRef {
    type Ref;

    fn llvm_ref(&self) -> Self::Ref;
}

macro_rules! imp_llvm_ref {
    ($($name:ident, $ref:ident, |$self:ident| $field:expr);*$(;)?) => {
        $(impl LlvmRef for $name {
            type Ref = $ref;

            fn llvm_ref(&$self) -> <Self as LlvmRef>::Ref {
                $field
            }
        })*
    };
}

imp_llvm_ref! {
    BasicBlock, LLVMBasicBlockRef, |self| self.ptr;
    Builder, LLVMBuilderRef, |self| self.ptr;
    Context, LLVMContextRef, |self| self.ptr;
    ExecutionEngine, LLVMExecutionEngineRef, |self| self.ptr;
    Module, LLVMModuleRef, |self| self.ptr;
    Type, LLVMTypeRef, |self| self.ptr;
    AnyValue, LLVMValueRef, |self| self.ptr;
    Function, LLVMValueRef, |self| self.value.ptr;
    FunctionPassManagerBuilder, LLVMPassManagerRef, |self| self.ptr;
    TargetData, LLVMTargetDataRef, |self| self.ptr;
}
