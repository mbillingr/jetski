
use cranelift::prelude::*;
use jetski::{Object, Result, ErrorKind, parser::parse_datum, jit::Tag};
use rustyline::{error::ReadlineError, Editor};
use std::collections::HashMap;
use cranelift_module::{Module, Linkage};
use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};
use cranelift::codegen::write_function;

fn lookup(tag: i8, val: i64) -> (i8, i64) {
    println!("Hello from the Rust function <lookup> :)");
    (0, 0)
}

fn compile_top_level(expr: &Object) -> Result<fn()->(Tag, i64)> {
    let mut jb = SimpleJITBuilder::new();
    jb.symbol("lookup", lookup as *const _);
    let mut module = Module::new(jb);

    let fn_code = compile_function(&mut module, &Object::nil(), expr, "main")?;
    let fn_ptr = unsafe {std::mem::transmute::<_, fn()->(Tag, i64)>(fn_code)};
    Ok(fn_ptr)
}

fn compile_function(module: &mut Module<SimpleJITBackend>, params: &Object, body: &Object, name: &str) -> Result<*const u8> {
    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let mut signature = module.make_signature();
    for _ in 0..params.list_len().unwrap() {
        signature.params.push(AbiParam::new(types::I8));
        signature.params.push(AbiParam::new(types::I64));
    }
    signature.returns.push(AbiParam::new(types::I8));
    signature.returns.push(AbiParam::new(types::I64));

    let func = module.declare_function(name, Linkage::Local, &signature).unwrap();

    ctx.func.signature = signature;
    ctx.func.name = ExternalName::user(0, func.as_u32()); {
        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let ebb = bcx.create_ebb();

        bcx.switch_to_block(ebb);
        bcx.append_ebb_params_for_function_params(ebb);

        let mut compiler = Compiler::new(module, &mut bcx);

        let (tag, val) = compiler.compile_expression(body)?;

        bcx.ins().return_(&[tag, val]);
        bcx.seal_all_blocks();
        bcx.finalize();

        let mut s = String::new();
        write_function(&mut s, &bcx.func, None).unwrap();
        println!("{}", s);
    }
    module.define_function(func, &mut ctx).unwrap();
    module.clear_context(&mut ctx);

    module.finalize_definitions();

    Ok(module.get_finalized_function(func))
}

struct Compiler<'a, 'b> {
    module: &'a mut Module<SimpleJITBackend>,
    builder: &'a mut FunctionBuilder<'b>,
    //builtins: HashMap<&'static str, Function>,
}

impl<'a, 'b> Compiler<'a, 'b> {
    fn new(module: &'a mut Module<SimpleJITBackend>, builder: &'a mut FunctionBuilder<'b>) -> Self {
        Compiler {
            module,
            builder,
            //builtins: HashMap::new()
        }
    }

    fn compile_expression(&mut self, expr: &Object) -> Result<(Value, Value)> {
        if is_self_evaluating(expr) {
            self.compile_self_evaluating(expr)
        } else if is_variable(expr) {
            self.compile_variable(expr)
        /*} else if is_hardcoded(expr) {
            self.compile_hardcoded(expr)
        } else if is_lambda(expr) {
            self.compile_lambda(expr)
        } else if is_application(expr) {
            self.compile_application(expr)*/
        } else {
            Err(ErrorKind::UnknownExpressionType(expr.clone()).into())
        }
    }

    fn compile_self_evaluating(&mut self, expr: &Object) -> Result<(Value, Value)> {
        if expr.is_integer() {
            Ok(self.make_integer(expr.try_as_integer().unwrap()))
        } else if expr.is_float() {
            Ok(self.make_float(expr.try_as_float().unwrap()))
        } else {
            unimplemented!()
        }
    }

    fn compile_variable(&mut self, expr: &Object) -> Result<(Value, Value)> {
        let (tag, val) = self.make_symbol(expr.try_as_symbol_name().unwrap());

        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I8));
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I8));
        sig.returns.push(AbiParam::new(types::I64));

        let lookup_decl = self.module.declare_function("lookup", Linkage::Import, &sig).unwrap();
        let lookup = self.module.declare_func_in_func(lookup_decl, self.builder.func);
        let call = self.builder.ins().call(lookup, &[tag, val]);
        //Ok(self.builder.call(self.builtins["lookup"], &[symbol], "var"))
        let results = self.builder.inst_results(call);
        Ok((results[0], results[1]))
    }

    /*fn compile_hardcoded(&mut self, expr: &Object) -> Result<Value> {
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
    }*/

    /*fn compile_application(&mut self, expr: &Object) -> Result<Value> {
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
    }*/

    fn make_integer(&mut self, value: i64) -> (Value, Value) {
        let tag = self.builder.ins().iconst(types::I8, Tag::Integer as i64);
        let val = self.builder.ins().iconst(types::I64, value);
        (tag, val)
    }

    fn make_float(&mut self, value: f64) -> (Value, Value) {
        let tag = self.builder.ins().iconst(types::I8, Tag::Float as i64);
        let val = self.builder.ins().iconst(types::I64, unsafe { std::mem::transmute::<f64, i64>(value) } );
        (tag, val)
    }

    fn make_symbol(&mut self, name: &str) -> (Value, Value) {
        // TODO: get pointer or something that uniquely identifies the symbol
        // temporary workaround: use first character
        let tag = self.builder.ins().iconst(types::I8, Tag::Symbol as i64);
        let val = self.builder.ins().iconst(types::I64, name.chars().next().unwrap() as i64 );
        (tag, val)
    }

    /*fn make_function(&mut self, func: Function) -> Value {
        let fptr = self.builder.ptr_to_int(func.into(), Type::I64, "fptr");
        Value::const_struct(&[Value::const_u8(Tag::Function as u8), fptr], false)
    }*/
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

    //use llvm_sys::execution_engine::
    //ee = LLVMEng

    let mut editor = Editor::<()>::new();
    loop {
        match editor.readline("ready> ") {
            Ok(line) => {
                editor.add_history_entry(line.clone());
                let expression = parse_datum(&line)?;
                let top_fn = compile_top_level(&expression)?;
                let result: Object = top_fn().into();
                println!("{:?}", result);
            }
            Err(ReadlineError::Eof) => return Ok(()),
            Err(e) => eprintln!("{}", e.to_string()),
        }
    }

    Ok(())
}
