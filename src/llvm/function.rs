use super::block::Block;
use super::str_to_cstring;
use super::value::Value;
use llvm_sys::core::LLVMAppendBasicBlock;
use llvm_sys::prelude::*;

/// Although functions are values too, it improves type safety if we treat them as different types
#[derive(Copy, Clone)]
pub struct Function {
    ptr: LLVMValueRef,
}

impl Function {
    pub fn append_block(&mut self, name: &str) -> Block {
        let name = str_to_cstring(name);
        unsafe { LLVMAppendBasicBlock(self.ptr, name.as_ptr()).into() }
    }
}

impl From<Function> for LLVMValueRef {
    fn from(f: Function) -> Self {
        f.ptr
    }
}

impl From<&Function> for LLVMValueRef {
    fn from(f: &Function) -> Self {
        f.ptr
    }
}

impl From<LLVMValueRef> for Function {
    fn from(ptr: LLVMValueRef) -> Self {
        Function { ptr }
    }
}

impl From<Value> for Function {
    fn from(val: Value) -> Self {
        let tmp: LLVMValueRef = val.into();
        tmp.into()
    }
}

impl From<Function> for Value {
    fn from(val: Function) -> Self {
        let tmp: LLVMValueRef = val.into();
        tmp.into()
    }
}
