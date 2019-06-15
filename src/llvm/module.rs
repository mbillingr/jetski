use super::context::Context;
use super::function::Function;
use super::types::Type;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::ffi::{CStr, CString};

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

    pub fn add_function(&mut self, name: &str, func_type: Type) -> Function {
        let name = CString::new(name).unwrap();
        unsafe { LLVMAddFunction(self.ptr, name.as_ptr(), func_type.into()).into() }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.ptr) }
    }
}

impl From<&Module> for LLVMModuleRef {
    fn from(m: &Module) -> Self {
        m.ptr
    }
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            let cstr = LLVMPrintModuleToString(self.ptr);
            let s = CStr::from_ptr(cstr).to_str().unwrap();
            write!(f, "{}", s)?;
            LLVMDisposeMessage(cstr);
            Ok(())
        }
    }
}
