use llvm_sys::core::*;
use llvm_sys::prelude::*;

use super::block::Block;
use super::context::Context;
use super::function::Function;
use super::str_to_cstring;
use super::types::Type;
use super::value::Value;

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

    pub fn position_at_end(&mut self, block: Block) {
        unsafe { LLVMPositionBuilderAtEnd(self.ptr, block.into()) }
    }

    pub fn add(&mut self, lhs: Value, rhs: Value, name: &str) -> Value {
        let name = str_to_cstring(name);
        unsafe { LLVMBuildAdd(self.ptr, lhs.into(), rhs.into(), name.as_ptr()).into() }
    }

    pub fn sub(&mut self, lhs: Value, rhs: Value, name: &str) -> Value {
        let name = str_to_cstring(name);
        unsafe { LLVMBuildSub(self.ptr, lhs.into(), rhs.into(), name.as_ptr()).into() }
    }

    pub fn extract_value(&mut self, aggval: Value, index: usize, name: &str) -> Value {
        let name = str_to_cstring(name);
        unsafe {
            LLVMBuildExtractValue(self.ptr, aggval.into(), index as u32, name.as_ptr()).into()
        }
    }

    pub fn call(&mut self, func: Function, args: &[Value], name: &str) -> Value {
        let mut args: Vec<_> = args.iter().map(Into::into).collect();
        let name = str_to_cstring(name);
        unsafe {
            let call_inst = LLVMBuildCall(
                self.ptr,
                func.into(),
                args.as_mut_ptr(),
                args.len() as u32,
                name.as_ptr(),
            );
            LLVMSetTailCall(call_inst, 1);
            call_inst.into()
        }
    }

    pub fn ptr_to_int(&mut self, val: Value, dest_type: Type, name: &str) -> Value {
        let name = str_to_cstring(name);
        unsafe { LLVMBuildPtrToInt(self.ptr, val.into(), dest_type.into(), name.as_ptr()).into() }
    }

    pub fn int_to_ptr(&mut self, val: Value, dest_type: Type, name: &str) -> Value {
        let name = str_to_cstring(name);
        unsafe { LLVMBuildIntToPtr(self.ptr, val.into(), dest_type.into(), name.as_ptr()).into() }
    }

    pub fn ret(&mut self, val: Value) -> Value {
        unsafe { LLVMBuildRet(self.ptr, val.into()).into() }
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.ptr) }
    }
}
