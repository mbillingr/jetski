use cranelift::codegen::write_function;
use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_preopt::optimize;
use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};
use jetski::{jit::Tag, parser::parse_datum, ErrorKind, Object, Result};
use rustyline::{error::ReadlineError, Editor};
use std::collections::HashMap;

fn lookup(tag: i8, val: i64) -> (i8, i64) {
    println!("looking up {:?}", (tag, val));
    (0, 0)
}

fn compile_top_level(
    module: &mut Module<SimpleJITBackend>,
    expr: &Object,
) -> Result<fn() -> (Tag, i64)> {
    let fn_id = compile_function(module, &Object::nil(), expr, "main")?;
    let fn_code = module.get_finalized_function(fn_id);
    let fn_ptr = unsafe { std::mem::transmute::<_, fn() -> (Tag, i64)>(fn_code) };
    Ok(fn_ptr)
}

fn compile_function(
    module: &mut Module<SimpleJITBackend>,
    params: &Object,
    body: &Object,
    name: &str,
) -> Result<FuncId> {
    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let signature = make_dynamic_signature(module, params.list_len().unwrap());

    let func = module
        .declare_function(name, Linkage::Local, &signature)
        .unwrap();

    ctx.func.signature = signature;
    ctx.func.name = ExternalName::user(0, func.as_u32());
    {
        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let ebb = bcx.create_ebb();

        bcx.switch_to_block(ebb);
        bcx.append_ebb_params_for_function_params(ebb);

        let mut compiler = Compiler::new(module, &mut bcx);

        let (tag, val) = compiler.compile_expression(body)?;

        bcx.ins().return_(&[tag, val]);
        bcx.seal_all_blocks();
        bcx.finalize();
    }
    optimize(&mut ctx, module.isa()).unwrap();

    let mut s = String::new();
    write_function(&mut s, &ctx.func, None).unwrap();
    println!("{}", s);

    module.define_function(func, &mut ctx).unwrap();
    module.clear_context(&mut ctx);

    module.finalize_definitions();

    Ok(func)
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

        let lookup_decl = self
            .module
            .declare_function("lookup", Linkage::Import, &sig)
            .unwrap();
        let lookup = self
            .module
            .declare_func_in_func(lookup_decl, self.builder.func);
        let call = self.builder.ins().call(lookup, &[tag, val]);
        let results = self.builder.inst_results(call);
        Ok((results[0], results[1]))
    }

    fn compile_hardcoded(&mut self, expr: &Object) -> Result<(Value, Value)> {
        let lhs = self.compile_expression(expr.get_ref(1).unwrap())?;
        let rhs = self.compile_expression(expr.get_ref(2).unwrap())?;

        let result = match expr.get_ref(0).and_then(Object::try_as_symbol_name) {
            Some("+") => self.builder.ins().iadd(lhs.1, rhs.1),
            Some("-") => self.builder.ins().isub(lhs.1, rhs.1),
            Some("*") => self.builder.ins().imul(lhs.1, rhs.1),
            Some("/") => self.builder.ins().sdiv(lhs.1, rhs.1),
            _ => unreachable!(),
        };

        Ok(self.cast_integer(result))
    }

    fn compile_lambda(&mut self, expr: &Object) -> Result<(Value, Value)> {
        let func_id = compile_function(
            self.module,
            lambda_params(&expr),
            lambda_body(&expr),
            "lambda",
        )?;
        let func_ref = self.module.declare_func_in_func(func_id, self.builder.func);
        let addr = self
            .builder
            .ins()
            .func_addr(Type::int(64).unwrap(), func_ref); // TODO: automatically detect pointer size of target
        Ok(self.cast_function(addr))
    }

    fn compile_application(&mut self, expr: &Object) -> Result<(Value, Value)> {
        let signature = make_dynamic_signature(self.module, get_operands(expr).len());
        let sig = self.builder.func.import_signature(signature);

        let proc = self.compile_expression(get_operator(expr))?;
        let args = self.compile_args(get_operands(expr))?;

        // TODO: check if proc is a function

        // TODO: check that signatures match

        let call = self.builder.ins().call_indirect(sig, proc.1, &args);

        let results = self.builder.inst_results(call);
        Ok((results[0], results[1]))
    }

    fn compile_args(&mut self, args: &[Object]) -> Result<Vec<Value>> {
        let mut compiled_args = vec![];
        for op in args {
            let (tag, val) = self.compile_expression(op)?;
            compiled_args.push(tag);
            compiled_args.push(val);
        }
        Ok(compiled_args)
    }

    fn make_integer(&mut self, value: i64) -> (Value, Value) {
        let tag = self.builder.ins().iconst(types::I8, Tag::Integer as i64);
        let val = self.builder.ins().iconst(types::I64, value);
        (tag, val)
    }

    fn cast_integer(&mut self, val: Value) -> (Value, Value) {
        let tag = self.builder.ins().iconst(types::I8, Tag::Integer as i64);
        (tag, val)
    }

    fn make_float(&mut self, value: f64) -> (Value, Value) {
        let tag = self.builder.ins().iconst(types::I8, Tag::Float as i64);
        let val = self.builder.ins().iconst(types::I64, unsafe {
            std::mem::transmute::<f64, i64>(value)
        });
        (tag, val)
    }

    fn make_symbol(&mut self, name: &str) -> (Value, Value) {
        // TODO: get pointer or something that uniquely identifies the symbol
        // temporary workaround: use first character
        let tag = self.builder.ins().iconst(types::I8, Tag::Symbol as i64);
        let val = self
            .builder
            .ins()
            .iconst(types::I64, name.chars().next().unwrap() as i64);
        (tag, val)
    }

    fn cast_function(&mut self, func: Value) -> (Value, Value) {
        let tag = self.builder.ins().iconst(types::I8, Tag::Function as i64);
        (tag, func)
    }
}

fn make_dynamic_signature(module: &mut Module<SimpleJITBackend>, nargs: usize) -> Signature {
    let mut signature = module.make_signature();
    for _ in 0..nargs {
        signature.params.push(AbiParam::new(types::I8));
        signature.params.push(AbiParam::new(types::I64));
    }
    signature.returns.push(AbiParam::new(types::I8));
    signature.returns.push(AbiParam::new(types::I64));
    signature
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

                // TODO: I'm not sure if we should reuse the module or create a new one every time.
                //       Since functions cannot be dropped from modules, reusing would require
                //       unique function names. I'm not sure if/how functions from different modules
                //       can call each other, which is a requirement of recreating...
                let mut jb = SimpleJITBuilder::new();
                jb.symbol("lookup", lookup as *const _);
                let mut module = Module::new(jb);

                let top_fn = compile_top_level(&mut module, &expression)?;
                let result: Object = top_fn().into();
                println!("{:?}", result);
            }
            Err(ReadlineError::Eof) => return Ok(()),
            Err(e) => eprintln!("{}", e.to_string()),
        }
    }
}
