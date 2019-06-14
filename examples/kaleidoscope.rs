use jetski::llvm::{self, Function, Type, Value};
use jetski::llvm::{Builder, Module};
use jetski::{parser::parse_datum, ErrorKind, Object, Result};
use llvm_sys::core::{LLVMDisposeMessage, LLVMPrintModuleToString, LLVMPrintValueToString};
use rustyline::{error::ReadlineError, Editor};
use std::collections::HashMap;
use std::ffi::CStr;

//use llvm_sys::{LLVMValue, LLVMValueKind};
//use llvm_sys::core::LLVMContextCreate;

enum Tag {
    _Null,
    Integer,
    Float,
    Symbol,
    Function,
}

struct Compiler {
    module: Module,
    builder: Builder,
    builtins: HashMap<&'static str, Function>,

    obj_type: Type,
}

impl Compiler {
    fn new() -> Self {
        let context = llvm::Context::global();
        let module = context.create_module("abc");
        let builder = context.create_builder();
        let mut compiler = Compiler {
            module,
            builder,
            builtins: HashMap::new(),
            obj_type: Type::structure(&[Type::I8, Type::I64]),
        };
        compiler.initialize_builtins();
        compiler
    }

    fn initialize_builtins(&mut self) {
        self.builtins.insert(
            "lookup",
            self.module
                .add_function("lookup", Type::function(self.obj_type, &[self.obj_type])),
        );
    }

    fn compile_top_level(&mut self, expr: &Object) -> Result<Function> {
        self.compile_function(&Object::nil(), expr, "main")
    }

    fn compile_expression(&mut self, expr: &Object) -> Result<Value> {
        if is_self_evaluating(expr) {
            self.compile_self_evaluating(expr)
        } else if is_variable(expr) {
            self.compile_variable(expr)
        } else if is_hardcoded(expr) {
            self.compile_hardcoded(expr)
        } else if is_lambda(expr) {
            self.compile_lambda(expr)
        } else if is_application(expr) {
            self.compile_application(expr)
        } else {
            Err(ErrorKind::UnknownExpressionType(expr.clone()).into())
        }
    }

    fn compile_self_evaluating(&mut self, expr: &Object) -> Result<Value> {
        if expr.is_integer() {
            Ok(self.make_integer(expr.try_as_integer().unwrap()))
        } else if expr.is_float() {
            Ok(self.make_float(expr.try_as_float().unwrap()))
        } else {
            unimplemented!()
        }
    }

    fn compile_variable(&mut self, expr: &Object) -> Result<Value> {
        let symbol = self.make_symbol(expr.try_as_symbol_name().unwrap());
        Ok(self.builder.call(self.builtins["lookup"], &[symbol], "var"))
    }

    fn compile_hardcoded(&mut self, expr: &Object) -> Result<Value> {
        let lhs = self.compile_expression(expr.get_ref(1).unwrap())?;
        let rhs = self.compile_expression(expr.get_ref(2).unwrap())?;

        let a = self.builder.extract_value(lhs, 1, "lhs");
        let b = self.builder.extract_value(rhs, 1, "rhs");

        let result = match expr.get_ref(0).and_then(Object::try_as_symbol_name) {
            Some("+") => self.builder.add(a, b, "sum"),
            Some("-") => self.builder.sub(a, b, "diff"),
            Some("*") => self.builder.mul(a, b, "prod"),
            Some("/") => self.builder.div(a, b, "quot"),
            _ => unreachable!(),
        };

        let tmp = self.make_integer(0);
        Ok(self.builder.insert_value(tmp, result, 1, "var"))
    }

    fn compile_lambda(&mut self, expr: &Object) -> Result<Value> {
        let func = self.compile_function(lambda_params(&expr), lambda_body(&expr), "lambda")?;
        Ok(self.make_function(func))
    }

    fn compile_function(&mut self, params: &Object, body: &Object, name: &str) -> Result<Function> {
        let arg_types = vec![self.obj_type; params.list_len().unwrap()];
        let fnty = Type::function(self.obj_type, &arg_types);
        let mut func = self.module.add_function(name, fnty);

        let block = func.append_block("entry");

        let mut builder = llvm::Context::global().create_builder();
        builder.position_at_end(block);

        std::mem::swap(&mut self.builder, &mut builder);

        let body = self.compile_expression(body)?;
        self.builder.ret(body);

        std::mem::swap(&mut self.builder, &mut builder);

        Ok(func)
    }

    fn compile_application(&mut self, expr: &Object) -> Result<Value> {
        // TODO: it seems a bit hackish to determine the function type from the number of arguments
        //       passed in the call. Would it be better to encode the signature in the function tag?
        let arg_types = vec![self.obj_type; get_operands(expr).len()];
        let fnty = Type::function(self.obj_type, &arg_types);

        let proc = self.compile_expression(get_operator(expr))?;
        let func = self.builder.int_to_ptr(proc, Type::pointer(fnty), "func");
        let args: Vec<_> = get_operands(expr)
            .iter()
            .map(|x| self.compile_expression(x))
            .collect::<Result<_>>()?;
        Ok(self.builder.call(func.into(), &args, "var"))
    }

    fn make_integer(&self, value: i64) -> Value {
        Value::const_struct(
            &[Value::const_u8(Tag::Integer as u8), Value::const_i64(value)],
            false,
        )
    }

    fn make_float(&self, value: f64) -> Value {
        Value::const_struct(
            &[Value::const_u8(Tag::Float as u8), Value::const_f64(value)],
            false,
        )
    }

    fn make_symbol(&self, name: &str) -> Value {
        // TODO: get pointer or something that uniquely identifies the symbol
        // temporary workaround: use first character
        Value::const_struct(
            &[
                Value::const_u8(Tag::Symbol as u8),
                Value::const_u64(name.chars().next().unwrap() as u64),
            ],
            false,
        )
    }

    fn make_function(&mut self, func: Function) -> Value {
        let fptr = self.builder.ptr_to_int(func.into(), Type::I64, "fptr");
        Value::const_struct(&[Value::const_u8(Tag::Function as u8), fptr], false)
    }
}

fn is_self_evaluating(expr: &Object) -> bool {
    expr.is_number() || expr.is_string()
}

fn is_variable(expr: &Object) -> bool {
    expr.is_symbol()
}

fn is_hardcoded(expr: &Object) -> bool {
    expr.car()
        .and_then(Object::try_as_symbol_name)
        .map(|name| ["+", "-", "*", "/"].contains(&name))
        .unwrap_or(false)
}

fn is_lambda(expr: &Object) -> bool {
    expr.car()
        .and_then(Object::try_as_symbol_name)
        .map(|name| name == "lambda")
        .unwrap_or(false)
}

fn lambda_params(expr: &Object) -> &Object {
    expr.get_ref(1).unwrap()
}

fn lambda_body(expr: &Object) -> &Object {
    expr.get_ref(2).unwrap()
}

fn is_application(expr: &Object) -> bool {
    expr.is_list()
}

fn get_operator(expr: &Object) -> &Object {
    expr.car().unwrap()
}

fn get_operands(expr: &Object) -> &[Object] {
    &expr.try_as_slice().unwrap()[1..]
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
                let value = compiler.compile_top_level(&expression)?;
                unsafe {
                    let cstr = LLVMPrintValueToString(value.into());
                    let s = CStr::from_ptr(cstr).to_str().unwrap();
                    println!("{}", s);
                    LLVMDisposeMessage(cstr);

                    let cstr = LLVMPrintModuleToString((&compiler.module).into());
                    let s = CStr::from_ptr(cstr).to_str().unwrap();
                    println!("{}", s);
                    LLVMDisposeMessage(cstr);
                }
            }
            Err(ReadlineError::Eof) => return Ok(()),
            Err(e) => eprintln!("{}", e.to_string()),
        }
    }
}
