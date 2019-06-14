use llvm_sys::core::{
    LLVMConstInt, LLVMConstReal, LLVMConstStruct, LLVMDisposeMessage, LLVMDoubleType,
    LLVMInt64Type, LLVMInt8Type, LLVMPrintValueToString,
};
use llvm_sys::prelude::*;
use std::ffi::CStr;

#[derive(Copy, Clone)]
pub struct Value {
    ptr: LLVMValueRef,
}

impl Value {
    pub fn const_i8(x: i8) -> Self {
        unsafe { LLVMConstInt(LLVMInt8Type(), std::mem::transmute::<_, u8>(x) as u64, 1).into() }
    }
    pub fn const_u8(x: u8) -> Self {
        unsafe { LLVMConstInt(LLVMInt8Type(), x as u64, 0).into() }
    }

    pub fn const_i64(x: i64) -> Self {
        unsafe { LLVMConstInt(LLVMInt64Type(), std::mem::transmute::<_, u64>(x), 1).into() }
    }

    pub fn const_u64(x: u64) -> Self {
        unsafe { LLVMConstInt(LLVMInt64Type(), x, 0).into() }
    }

    pub fn const_f64(x: f64) -> Self {
        unsafe { LLVMConstReal(LLVMDoubleType(), x).into() }
    }

    pub fn const_struct(elements: &[Value], packed: bool) -> Value {
        let mut elements: Vec<_> = elements.iter().map(Into::into).collect();
        unsafe {
            LLVMConstStruct(elements.as_mut_ptr(), elements.len() as u32, packed as i32).into()
        }
    }
}

impl From<Value> for LLVMValueRef {
    fn from(val: Value) -> Self {
        val.ptr
    }
}

impl From<&Value> for LLVMValueRef {
    fn from(val: &Value) -> Self {
        val.ptr
    }
}

impl From<LLVMValueRef> for Value {
    fn from(ptr: LLVMValueRef) -> Self {
        Value { ptr }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            let cstr = LLVMPrintValueToString(self.ptr);
            let s = CStr::from_ptr(cstr).to_str().unwrap();
            write!(f, "{}", s)?;
            LLVMDisposeMessage(cstr);
            Ok(())
        }
    }
}
