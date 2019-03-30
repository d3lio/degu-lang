use llvm::core::{
    LLVMAddFunction,
    LLVMGetNamedFunction,
    LLVMDisposeModule,
    LLVMModuleCreateWithNameInContext,
    LLVMPrintModuleToString,
};

use std::ffi::CStr;
use std::fmt::{self, Debug, Formatter};
use std::ops::Drop;

use super::context::Context;
use super::llvm_ref::LlvmRef;
use super::transformation::FunctionPassManagerBuilder;
use super::types::Type;
use super::util::EMPTY_C_STR;
use super::value::{AnyValue, Function};

pub struct Module {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}

impl Debug for Module {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        unsafe {
            // TODO: Does this require LLVMDisposeMessage?
            let dump = LLVMPrintModuleToString(self.ptr);
            write!(f, "{}", CStr::from_ptr(dump).to_str().unwrap())
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.ptr);
        }
    }
}

impl Module {
    pub(crate) fn new(name: &CStr, context: &mut Context) -> Self {
        Self {
            ptr: unsafe { LLVMModuleCreateWithNameInContext(name.as_ptr(), context.llvm_ref()) },
        }
    }

    pub fn function_prototype(&mut self, name: Option<&CStr>, fn_type: Type) -> Function {
        Function {
            value: AnyValue {
                ptr: unsafe {
                    LLVMAddFunction(
                        self.ptr,
                        name.map_or(EMPTY_C_STR, CStr::as_ptr),
                        fn_type.ptr,
                    )
                },
            },
        }
    }

    pub fn get_function(&self, name: &CStr) -> Option<Function> {
        let ptr = unsafe { LLVMGetNamedFunction(self.ptr, name.as_ptr()) };

        if ptr.is_null() {
            None
        } else {
            Some(Function {
                value: AnyValue {
                    ptr,
                },
            })
        }
    }

    pub fn function_pass_manager_builder(&mut self) -> FunctionPassManagerBuilder {
        FunctionPassManagerBuilder::new(self)
    }
}
