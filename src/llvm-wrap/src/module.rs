use llvm::core::{
    LLVMAddFunction,
    LLVMDisposeModule,
    LLVMModuleCreateWithNameInContext,
    LLVMPrintModuleToString,
};

use std::ffi::CStr;
use std::fmt::{self, Debug, Formatter};
use std::ops::Drop;

use super::context::Context;
use super::llvm_ref::LlvmRef;
use super::types::Type;
use super::util::EMPTY_C_STR;
use super::value::{AnyValue, Function};

pub struct Module {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}

impl Debug for Module {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        unsafe {
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
    pub(crate) fn new(name: &CStr, context: &Context) -> Self {
        Self {
            ptr: unsafe { LLVMModuleCreateWithNameInContext(name.as_ptr(), context.llvm_ref()) },
        }
    }

    pub fn function_prototype(&mut self, name: Option<&CStr>, fn_type: Type) -> Function {
        unsafe {
            Function {
                value: AnyValue {
                    ptr: LLVMAddFunction(
                        self.ptr,
                        name.map_or(EMPTY_C_STR, CStr::as_ptr),
                        fn_type.ptr,
                    ),
                },
            }
        }
    }
}
