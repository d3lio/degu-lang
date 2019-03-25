use llvm::core::*;
use llvm::execution_engine::*;
use llvm::target::*;

use llvm_wrap::intern::CStringInternPool;
use llvm_wrap::llvm_ref::LlvmRef;
use llvm_wrap::prelude::{BasicBlock, Context, ExecutionEngine, Function, Module, Type};

fn create_function(
    pool: &mut CStringInternPool,
    module: &mut Module,
    name: &str,
    ty: Type,
    args: &[(&str, Type)],
) -> Function {
    let mut f = module.function_prototype(
        Some(pool.intern(name)),
        Context::function_type(
            ty,
            args.iter()
                .map(|(_, ty)| *ty)
                .collect::<Vec<_>>()
                .as_slice(),
            false,
        ),
    );

    for (ref mut param, &(name, _)) in f.params().iter_mut().zip(args) {
        param.set_name(pool.intern(name));
    }

    BasicBlock::create_and_append(pool.intern("entry"), &mut f);

    f
}

#[no_mangle]
extern "C" fn print_u32(value: u32) {
    println!("{}", value);
}

pub fn main() {
    let mut pool = CStringInternPool::new();

    println!("init llvm");
    let mut context = Context::new();
    let mut module = context.create_module(pool.intern("main"));
    let mut builder = context.create_builder();

    println!("create print");
    let print_f = module.function_prototype(
        Some(pool.intern("print_u32")),
        Context::function_type(context.void_type(), &[context.i32_type()], false),
    );

    println!("build sum_three");
    let sum_three_f = {
        let sum_three_f = create_function(
            &mut pool,
            &mut module,
            "sum_three",
            context.i32_type(),
            &[
                ("arg1", context.i32_type()),
                ("arg2", context.i32_type()),
                ("arg3", context.i32_type()),
            ],
        );

        let params = sum_three_f.params();

        builder.position_at_end(&sum_three_f.entry_block().unwrap());

        let temp = builder.build_add(&params[0], &params[1], Some(pool.intern("temp")));
        let temp = builder.build_add(&temp, &params[2], Some(pool.intern("temp")));
        builder.build_ret(&temp);

        sum_three_f
    };

    println!("build main");
    {
        let main_f = create_function(
            &mut pool,
            &mut module,
            "main",
            context.void_type(),
            &[],
        );

        builder.position_at_end(&main_f.entry_block().unwrap());

        let one = builder.build_const_int(context.i32_type(), 1, false);
        let two = builder.build_const_int(context.i32_type(), 2, false);
        let three = builder.build_const_int(context.i32_type(), 3, false);

        let sum = builder
            .build_call(&sum_three_f, &[one, two, three], Some(pool.intern("sum")))
            .unwrap();

        builder.build_call(&print_f, &[sum], None).unwrap();
        builder.build_ret_void();
    }

    println!("\nLLVM IR:\n{:?}", module);

    println!("build ee");
    let mut ee = unsafe {
        LLVMLinkInMCJIT();
        if LLVM_InitializeNativeTarget() == 1 {
            std::process::exit(1);
        }
        if LLVM_InitializeNativeAsmPrinter() == 1 {
            std::process::exit(1);
        }
        if LLVM_InitializeNativeAsmParser() == 1 {
            std::process::exit(1);
        }

        let pass_manager = LLVMCreateFunctionPassManagerForModule(module.llvm_ref());
        LLVMInitializeFunctionPassManager(pass_manager);
        println!(
            "pass mgr: {}",
            LLVMRunFunctionPassManager(pass_manager, sum_three_f.llvm_ref())
        );

        ExecutionEngine::new(module).unwrap()
    };

    unsafe {
        ee.add_global_mapping(print_f.as_value(), print_u32 as usize);
    }

    let main: extern "C" fn() = unsafe {
        std::mem::transmute(ee.function_address(pool.intern("main")))
    };

    main();
}
