use llvm_sys::prelude::*;

pub struct Block {
    ptr: LLVMBasicBlockRef,
}

impl Block {}

impl From<Block> for LLVMBasicBlockRef {
    fn from(b: Block) -> Self {
        b.ptr
    }
}

impl From<&Block> for LLVMBasicBlockRef {
    fn from(b: &Block) -> Self {
        b.ptr
    }
}

impl From<LLVMBasicBlockRef> for Block {
    fn from(ptr: LLVMBasicBlockRef) -> Self {
        Block { ptr }
    }
}
