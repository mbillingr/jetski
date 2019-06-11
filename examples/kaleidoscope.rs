use jetski::{parser::parse_datum, Result, Object, ErrorKind};
use llvm_rs::Value;
use rustyline::{Editor, error::ReadlineError};

//use llvm_sys::{LLVMValue, LLVMValueKind};
//use llvm_sys::core::LLVMContextCreate;

fn compile(expr: Object) -> Result<()> {
    if is_self_evaluating(&expr) {
        compile_self_evaluating(expr)
    } else {
        Err(ErrorKind::UnknownExpressionType(expr).into())
    }
}

fn is_self_evaluating(expr: &Object) -> bool {
    expr.is_number() || expr.is_symbol() || expr.is_string()
}

fn compile_self_evaluating(expr: Object) -> Result<()> {
    println!("self evaluating: {}", expr);
    Ok(())
}

fn main() -> Result<()> {
    unsafe {
        //let context = LLVMContextCreate();
        let mut editor = Editor::<()>::new();
        loop {
            match editor.readline("ready> ") {
                Ok(line) => {
                    editor.add_history_entry(line.clone());
                    let expression = parse_datum(&line)?;
                    compile(expression)?;
                }
                Err(ReadlineError::Eof) => return Ok(()),
                Err(e) => eprintln!("{}", e.to_string())
            }
        }
    }
}