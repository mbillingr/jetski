use llvm_sys::prelude::*;
use llvm_sys::core::*;
use std::ffi::CString;

use super::context::Context;

pub struct Module {
    ptr: LLVMModuleRef,
}

impl Module {
    pub fn new(context: &Context, name: &str) -> Self {
        let name = CString::new(name).unwrap();
        unsafe {
            Module {
                ptr: LLVMModuleCreateWithNameInContext(name.as_ptr(), context.ptr),
            }
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.ptr)
        }
    }
}