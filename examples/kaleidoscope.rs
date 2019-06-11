use jetski::{parser::parse_datum, Result, Object, ErrorKind};
use jetski::llvm;
use rustyline::{Editor, error::ReadlineError};
use std::ffi::{CString, CStr};
use jetski::llvm::{Module, Context, Builder};
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::core::{LLVMConstInt, LLVMInt64Type, LLVMPrintValueToString, LLVMDisposeMessage, LLVMConstStruct, LLVMInt8Type, LLVMConstReal, LLVMDoubleType};

//use llvm_sys::{LLVMValue, LLVMValueKind};
//use llvm_sys::core::LLVMContextCreate;

struct Compiler {
    module: Module,
    builder: Builder,
}

impl Compiler {
    fn new() -> Self {
        let context = llvm::Context::global();
        let module = context.create_module("abc");
        let builder = context.create_builder();
        Compiler {
            module, builder
        }
    }

    fn compile(&mut self, expr: Object) -> Result<LLVMValueRef> {
        if is_self_evaluating(&expr) {
            self.compile_self_evaluating(expr)
        } else {
            Err(ErrorKind::UnknownExpressionType(expr).into())
        }
    }

    fn compile_self_evaluating(&mut self, expr: Object) -> Result<LLVMValueRef> {
        println!("self evaluating: {}", expr);
        //unsafe { Ok(LLVMConstInt(LLVMInt64Type(), 42, 1)) }
        if expr.is_integer() {
            unsafe {
                let mut vals = vec![
                    LLVMConstInt(LLVMInt8Type(), 1, 1),
                    LLVMConstInt(LLVMInt64Type(), expr.try_as_integer().unwrap() as u64, 1)];
                Ok(LLVMConstStruct(vals.as_mut_ptr(), vals.len() as u32, 0))
            }
        } else if expr.is_float() {
            unsafe {
                let mut vals = vec![
                    LLVMConstInt(LLVMInt8Type(), 2, 1),
                    LLVMConstReal(LLVMDoubleType(), expr.try_as_float().unwrap())];
                Ok(LLVMConstStruct(vals.as_mut_ptr(), vals.len() as u32, 0))
            }
        } else {
            unimplemented!()
        }
    }
}

fn is_self_evaluating(expr: &Object) -> bool {
    expr.is_number() || expr.is_symbol() || expr.is_string()
}

fn main() -> Result<()> {
    /*let context = LLVMContextCreate();
    let module = LLVMModuleCreateWithName(c_str!("main"));
    let builder = LLVMCreateBuilderInContext(context);*/

    let mut compiler = Compiler::new();

    let mut editor = Editor::<()>::new();
    loop {
        match editor.readline("ready> ") {
            Ok(line) => {
                editor.add_history_entry(line.clone());
                let expression = parse_datum(&line)?;
                let value = compiler.compile(expression)?;
                unsafe {
                    let cstr = LLVMPrintValueToString(value);
                    let s = CStr::from_ptr(cstr).to_str().unwrap();
                    println!("{:?}", s);
                    LLVMDisposeMessage(cstr);
                }
            }
            Err(ReadlineError::Eof) => return Ok(()),
            Err(e) => eprintln!("{}", e.to_string())
        }
    }
}