use llvm_sys::prelude::*;
use llvm_sys::core::*;

use super::context::Context;

pub struct Builder {
    ptr: LLVMBuilderRef,
}

impl Builder {
    pub fn new(context: &Context) -> Self {
        unsafe {
            Builder {
                ptr: LLVMCreateBuilderInContext(context.ptr),
            }
        }
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.ptr)
        }
    }
}