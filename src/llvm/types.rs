use llvm_sys::core::{
    LLVMFunctionType, LLVMInt16Type, LLVMInt32Type, LLVMInt64Type, LLVMInt8Type, LLVMPointerType,
    LLVMStructType,
};
use llvm_sys::prelude::*;

#[derive(Copy, Clone)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    Function(LLVMTypeRef),
    Structure(LLVMTypeRef),
    Pointer(LLVMTypeRef),
}

impl Type {
    pub fn pointer(inner: Type) -> Type {
        Type::Pointer(inner.into())
    }

    pub fn function(return_type: Type, param_types: &[Type]) -> Type {
        let mut param_types: Vec<_> = param_types.iter().map(Into::into).collect();
        unsafe {
            Type::Function(LLVMFunctionType(
                return_type.into(),
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                0,
            ))
        }
    }

    pub fn structure(element_types: &[Type]) -> Type {
        let mut element_types: Vec<_> = element_types.iter().map(Into::into).collect();
        unsafe {
            Type::Structure(LLVMStructType(
                element_types.as_mut_ptr(),
                element_types.len() as u32,
                0,
            ))
        }
    }
}

impl From<Type> for LLVMTypeRef {
    fn from(t: Type) -> Self {
        (&t).into()
    }
}

impl From<&Type> for LLVMTypeRef {
    fn from(t: &Type) -> Self {
        unsafe {
            match *t {
                Type::Function(type_ref) | Type::Structure(type_ref) => type_ref,
                Type::I8 => LLVMInt8Type(),
                Type::I16 => LLVMInt16Type(),
                Type::I32 => LLVMInt32Type(),
                Type::I64 => LLVMInt64Type(),
                Type::Pointer(target) => LLVMPointerType(target, 0),
            }
        }
    }
}
