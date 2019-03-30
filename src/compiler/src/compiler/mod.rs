use lexpar::lexer::Span;

use llvm_wrap::analysis::{VerifierFailureAction, verify_function, verify_module};
use llvm_wrap::builder::RealPredicate;
use llvm_wrap::execution_engine::initialize_jit;
use llvm_wrap::intern::CStringInternPool;
use llvm_wrap::prelude::*;

use syntax::parser::ast::{Ast, AstNode, BinOpKind, Prototype};

use std::collections::HashMap;

mod standard;

fn pretty_span(span: &Span) -> String {
    format!("{}:{}..{}", span.line, span.lo, span.hi)
}

struct Environment {
    pub vars: HashMap<String, AnyValue>,
    pub defs: HashMap<String, Function>,
}

pub struct Compiler {
    // Field order is drop order. Important for LLVM objects.
    // TODO: Figure out a way to not depend on the field order.
    // (Implementing Drop for Compiler is not an options since it prevents field move)
    builder: Builder,
    module: Module,
    context: Context,

    pool: CStringInternPool,
    env: Environment,
}

pub struct Runtime {
    // Field order is drop order. Important for LLVM objects.
    // TODO: Figure out a way to not depend on the field order.
    ee: ExecutionEngine,
    _builder: Builder,
    _context: Context,

    pool: CStringInternPool,
    env: Environment,
}

#[derive(Debug)]
pub struct CompilerError {
    message: String,
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

    pub fn compile(&mut self, ast: &AstNode) -> Result<&mut Compiler, CompilerError> {
        self.init_std();
        self.codegen(ast)?;

        let (is_bad, message) = verify_module(
            &self.module,
            VerifierFailureAction::PrintMessageAction);

        if is_bad {
            Err(CompilerError { message })
        } else {
            Ok(self)
        }
    }

    pub fn into_runtime(self) -> Runtime {
        initialize_jit();

        let mut runtime = Runtime {
            _context: self.context,
            _builder: self.builder,
            ee: ExecutionEngine::new(self.module).unwrap(),
            pool: self.pool,
            env: self.env,
        };

        runtime.init_std();
        runtime
    }

    // explicit destroy order
    // pub fn destroy(self) {
    //     let Compiler { pool, context, module, builder, .. } = self;
    // }

    fn codegen(&mut self, ast: &AstNode) -> Result<AnyValue, CompilerError> {
        match &*ast.expr {
            Ast::Number(num) => Ok(self.builder.build_const_fp(self.context.f64_type(), *num)),
            Ast::Ref(name) => {
                if name == "_" {
                    return Err(CompilerError {
                        message: format!(
                            "Illegal reference _ at {:?}",
                            pretty_span(&ast.span)),
                    });
                }

                self.env.vars.get(name)
                    .map(|var| var.clone())
                    .ok_or(CompilerError{
                        message: format!(
                            "Unknown variable ref {:?} at {:?}",
                            name,
                            pretty_span(&ast.span)),
                    })
            },
            Ast::Call { name, args } => {
                let values = args
                    .iter()
                    .map(|arg| self.codegen(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                let f = self.env.defs.get(name)
                    .ok_or(CompilerError{
                        message: format!(
                            "Unknown function ref {:?} at {:?}",
                            name,
                            pretty_span(&ast.span)),
                    })?;

                self.builder.build_call(f, &values, None)
                    .map_err(|err| CompilerError { message: format!("{:?}", err) })
            },
            Ast::BinOp { kind, lhs, rhs } => {
                use RealPredicate as RP;

                let lhs = self.codegen(&lhs)?;
                let rhs = self.codegen(&rhs)?;

                macro_rules! build_binop {
                    ($f:ident, $name: expr) => {
                        self.builder.$f(&lhs, &rhs, Some(self.pool.intern($name)))
                    };
                }

                Ok(match kind {
                    BinOpKind::Eq           => self.build_fp_cmp(RP::RealUEQ, &lhs, &rhs),
                    BinOpKind::NotEq        => self.build_fp_cmp(RP::RealUNE, &lhs, &rhs),
                    BinOpKind::GreaterThan  => self.build_fp_cmp(RP::RealUGT, &lhs, &rhs),
                    BinOpKind::GreaterEq    => self.build_fp_cmp(RP::RealUGE, &lhs, &rhs),
                    BinOpKind::LessThan     => self.build_fp_cmp(RP::RealULT, &lhs, &rhs),
                    BinOpKind::LessEq       => self.build_fp_cmp(RP::RealULE, &lhs, &rhs),
                    BinOpKind::Add          => build_binop!(build_fp_add, "addtmp"),
                    BinOpKind::Sub          => build_binop!(build_fp_sub, "subtmp"),
                    BinOpKind::Mul          => build_binop!(build_fp_mul, "multmp"),
                })
            },
            Ast::Block(exprs) => {
                exprs
                    .iter()
                    .map(|expr| self.codegen(expr))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .last()
                    .ok_or(CompilerError {
                        message: format!(
                            "Found empty block which is invalid value! {:?}",
                            pretty_span(&ast.span)),
                    })
            },
            Ast::Function { prototype: Prototype { name, args }, body } => {
                let f64_type = self.context.f64_type();
                let void_type = self.context.void_type();
                let is_main = name == "main";

                let mut f = {
                    let ret_type = if is_main { void_type } else { f64_type };
                    let arg_types = if is_main { vec![] } else { vec![f64_type; args.len()] };

                    self.module.function_prototype(
                        Some(self.pool.intern(name.as_ref())),
                        Context::function_type(ret_type, &arg_types, false),
                    )
                };

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

                let ret = self.codegen(&body)?;
                if is_main {
                    self.builder.build_ret_void();
                } else {
                    self.builder.build_ret(&ret);
                }

                if verify_function(&f, VerifierFailureAction::PrintMessageAction) {
                    return Err(CompilerError {
                        message: format!("{:?}", f),
                    });
                }

                self.env.defs.insert(name.clone(), f.clone());

                Ok(f.to_value())
            },
            _ => unimplemented!(),
        }
    }

    fn build_fp_cmp(&mut self, p: RealPredicate, l: &AnyValue, r: &AnyValue) -> AnyValue {
        let cmp = self.builder.build_fp_cmp(p, l, r, Some(self.pool.intern("cmptmp")));

        self.builder.build_cast_uint_to_fp(
            cmp,
            self.context.f64_type(),
            Some(self.pool.intern("booltmp")),
        )
    }
}

impl Runtime {
    pub fn run_main(&mut self) {
        let main: extern fn() = unsafe {
            std::mem::transmute(self.ee.function_address(self.pool.intern("main")))
        };

        main();
    }
}
