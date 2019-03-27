use llvm_wrap::prelude::*;
use llvm_wrap::intern::CStringInternPool;

use std::collections::HashMap;

use super::parser::ast::{Ast, AstNode, BinOpKind, Prototype};

struct Environment {
    pub vars: HashMap<String, AnyValue>,
    pub defs: HashMap<String, Function>,
}

pub struct Compiler {
    // field order matters as it's the drop order as well
    // TODO: use lifetimes to try and create an explicit drop order
    builder: Builder,
    module: Module,
    context: Context,
    pool: CStringInternPool,

    env: Environment,
}

impl Compiler {
    pub fn new() -> Self {
        let mut pool = CStringInternPool::new();
        let mut context = Context::new();
        let module = context.create_module(pool.intern("main"));
        let builder = context.create_builder();
        let env = Environment {
            vars: HashMap::new(),
            defs: HashMap::new(),
        };

        Self {
            pool,
            context,
            module,
            builder,
            env,
        }
    }

    pub fn module(&self) -> &Module {
        &self.module
    }

    pub fn compile(&mut self, ast: &AstNode) -> &mut Compiler {
        self.init_std();
        self.codegen(ast);
        self
    }

    // explicit destroy order
    // pub fn destroy(self) {
    //     let Compiler { pool, context, module, builder, .. } = self;
    // }

    fn init_std(&mut self) {
        self.env.defs.insert("print_number".to_string(),
            self.module.function_prototype(
                Some(self.pool.intern("print_number")),
                Context::function_type(self.context.void_type(), &[self.context.f64_type()], false),
            )
        );
    }

    fn codegen(&mut self, ast: &AstNode) -> AnyValue {
        match &*ast.expr {
            Ast::Number(num) => self.builder.build_const_fp(self.context.f64_type(), *num),
            Ast::Ref(name) => {
                self.env.vars.get(name)
                    .expect(&format!("Unknown variable ref {:?} at {:?}", name, ast.span))
                    .clone()
            },
            Ast::Call { name, args } => {
                let values = args
                    .iter()
                    .map(|arg| self.codegen(arg))
                    .collect::<Vec<_>>();

                let f = self.env.defs.get(name)
                    .expect(&format!("Unknown function ref {:?} at {:?}", name, ast.span));

                self.builder
                    .build_call(f, &values, Some(self.pool.intern("anonymous_call_sight")))
                    .unwrap()
            },
            Ast::BinOp { kind, lhs, rhs } => {
                let lhs = self.codegen(&lhs);
                let rhs = self.codegen(&rhs);
                match kind {
                    BinOpKind::Add => self.builder.build_add(&lhs, &rhs, None),
                    BinOpKind::Sub => self.builder.build_sub(&lhs, &rhs, None),
                    BinOpKind::Mul => self.builder.build_mul(&lhs, &rhs, None),
                }
            },
            Ast::Function { prototype: Prototype { name, args }, body } => {
                let f64_type = self.context.f64_type();

                let mut f = self.module.function_prototype(
                    Some(self.pool.intern(name.as_ref())),
                    Context::function_type(f64_type, &vec![f64_type; args.len()], false),
                );

                self.env.vars = f.params()
                    .into_iter()
                    .zip(args)
                    .fold(HashMap::new(), |mut acc, (mut param, name)| {
                        param.set_name(self.pool.intern(name.as_ref()));
                        acc.insert(name.clone(), param);
                        acc
                    });

                let bb = BasicBlock::create_and_append(self.pool.intern("entry"), &mut f);
                self.builder.position_at_end(&bb);

                let ret = self.codegen(&body);
                self.builder.build_ret(&ret);

                self.env.defs.insert(name.clone(), f.clone());

                f.to_value()
            },
            Ast::Block(exprs) => {
                exprs
                    .iter()
                    .map(|expr| self.codegen(expr))
                    .last()
                    .expect(&format!("Found empty block which is invalid value! {:?}", ast.span))
            },
            _ => unimplemented!(),
        }
    }
}
