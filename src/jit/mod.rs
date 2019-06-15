use crate::Object;
use cranelift::prelude::*;

#[derive(Debug)]
#[repr(u8)]
pub enum Tag {
    Undef,
    Null,
    Integer,
    Float,
    Symbol,
    Function,
}

impl From<(Tag, i64)> for Object {
    fn from((tag, val): (Tag, i64)) -> Object {
        match tag {
            Tag::Undef => Object::undef(),
            Tag::Null => Object::nil(),
            Tag::Integer => Object::integer(val),
            Tag::Float => Object::float(unsafe { std::mem::transmute::<i64, f64>(val) }),
            Tag::Symbol => {
                Object::symbol(&unsafe { std::mem::transmute::<_, char>(val as u32).to_string() })
            }
            Tag::Function => Object::function(val as *const _),
            _ => unimplemented!("Convert {:?} to object", (tag, val)),
        }
    }
}

#[cfg(test)]
mod learning_tests {
    use cranelift::codegen::{write, write_function};
    use cranelift::prelude::*;
    use cranelift_module::{Linkage, Module};
    use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};
    use std::sync::mpsc::TrySendError::Full;

    #[test]
    fn it_works() {
        let mut module: Module<SimpleJITBackend> = Module::new(SimpleJITBuilder::new());
        let mut ctx = module.make_context();
        let mut func_ctx = FunctionBuilderContext::new();

        let mut sig_a = module.make_signature();
        sig_a.params.push(AbiParam::new(types::I32));
        sig_a.returns.push(AbiParam::new(types::I32));

        let func_a = module
            .declare_function("a", Linkage::Local, &sig_a)
            .unwrap();

        ctx.func.signature = sig_a;
        ctx.func.name = ExternalName::user(0, func_a.as_u32());
        {
            let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let ebb = bcx.create_ebb();

            bcx.switch_to_block(ebb);
            bcx.append_ebb_params_for_function_params(ebb);
            let param = bcx.ebb_params(ebb)[0];
            let cst = bcx.ins().iconst(types::I32, 37);
            let add = bcx.ins().iadd(cst, param);
            bcx.ins().return_(&[add]);
            bcx.seal_all_blocks();
            bcx.finalize();

            let mut s = String::new();
            write_function(&mut s, &bcx.func, None).unwrap();
            println!("{}", s);
        }
        module.define_function(func_a, &mut ctx).unwrap();
        module.clear_context(&mut ctx);

        module.finalize_definitions();

        let code_a = module.get_finalized_function(func_a);

        let ptr_a = unsafe { std::mem::transmute::<_, fn(u32) -> u32>(code_a) };

        assert_eq!(ptr_a(5), 42);
    }

    #[test]
    fn multi_return() {
        let mut module: Module<SimpleJITBackend> = Module::new(SimpleJITBuilder::new());
        let mut ctx = module.make_context();
        let mut func_ctx = FunctionBuilderContext::new();

        let mut sig_a = module.make_signature();
        sig_a.returns.push(AbiParam::new(types::I32));
        sig_a.returns.push(AbiParam::new(types::I32));

        let func_a = module
            .declare_function("a", Linkage::Local, &sig_a)
            .unwrap();

        ctx.func.signature = sig_a;
        ctx.func.name = ExternalName::user(0, func_a.as_u32());
        {
            let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let ebb = bcx.create_ebb();

            bcx.switch_to_block(ebb);
            bcx.append_ebb_params_for_function_params(ebb);
            let a = bcx.ins().iconst(types::I32, 3);
            let b = bcx.ins().iconst(types::I32, 7);
            bcx.ins().return_(&[a, b]);
            bcx.seal_all_blocks();
            bcx.finalize();

            let mut s = String::new();
            write_function(&mut s, &bcx.func, None).unwrap();
            println!("{}", s);
        }
        module.define_function(func_a, &mut ctx).unwrap();
        module.clear_context(&mut ctx);

        module.finalize_definitions();

        let code_a = module.get_finalized_function(func_a);

        let ptr_a = unsafe { std::mem::transmute::<_, fn() -> (u32, u32)>(code_a) };

        assert_eq!(ptr_a(), (3, 7));
    }
}
