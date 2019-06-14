use super::builder::Builder;
use super::module::Module;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

pub struct Context {
    pub(crate) ptr: LLVMContextRef,
}

impl Context {
    pub fn global() -> Self {
        unsafe {
            Context {
                ptr: LLVMGetGlobalContext(),
            }
        }
    }

    pub fn create_builder(&self) -> Builder {
        Builder::new(self)
    }

    pub fn create_module(&self, name: &str) -> Module {
        Module::new(self, name)
    }
}
