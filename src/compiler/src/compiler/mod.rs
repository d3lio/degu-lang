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
    function_optimizer: FunctionPassManager,
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

type CodegenResult = Result<AnyValue, CompilerError>;

impl Compiler {
    pub fn new() -> Self {
        let mut pool = CStringInternPool::new();
        let mut context = Context::new();
        let mut module = context.create_module(pool.intern("main"));
        let builder = context.create_builder();
        let function_optimizer = module.function_pass_manager_builder()
            .add_instruction_combination_pass()
            .add_reassociate_pass()
            .add_gvn_pass()
            .add_cfg_simplification_pass()
            .build();

        Self {
            pool,
            context,
            module,
            builder,
            function_optimizer,
            env: Environment {
                vars: HashMap::new(),
                defs: HashMap::new(),
            }
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

    fn codegen(&mut self, ast: &AstNode) -> CodegenResult {
        match &*ast.expr {
            Ast::Number(num) => Ok(self.builder.build_const_fp(self.context.f64_type(), *num)),
            Ast::Block(exprs) => self.build_block(&ast.span, exprs),
            Ast::Ref(name) => self.build_ref(&ast.span, name),
            Ast::Call { name, args } => self.build_call(&ast.span, name, args),
            Ast::If { condition, then, el } => self.build_if(condition, then, el),
            Ast::BinOp { kind, lhs, rhs } => self.build_binop(*kind, lhs, rhs),
            Ast::Function { prototype: Prototype { name, args }, body } => {
                self.build_function(name, args, body)
            },
            _ => unimplemented!(),
        }
    }

    fn build_block(&mut self, span: &Span, exprs: &Vec<AstNode>) -> CodegenResult {
        exprs
            .iter()
            .map(|expr| self.codegen(expr))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .last()
            .ok_or(CompilerError {
                message: format!(
                    "Found empty block which is invalid value! {:?}",
                    pretty_span(span)),
            })
    }

    fn build_ref(&mut self, span: &Span, name: &String) -> CodegenResult {
        if name == "_" {
            return Err(CompilerError {
                message: format!(
                    "Illegal reference _ at {:?}",
                    pretty_span(span)),
            });
        }

        self.env.vars.get(name)
            .map(|var| var.clone())
            .ok_or(CompilerError{
                message: format!(
                    "Unknown variable ref {:?} at {:?}",
                    name,
                    pretty_span(span)),
            })
    }

    fn build_call(&mut self, span: &Span, name: &String, args: &Vec<AstNode>) -> CodegenResult {
        let values = args
            .iter()
            .map(|arg| self.codegen(arg))
            .collect::<Result<Vec<_>, _>>()?;

        let f = self.env.defs.get(name)
            .ok_or(CompilerError{
                message: format!(
                    "Unknown function ref {:?} at {:?}",
                    name,
                    pretty_span(span)),
            })?;

        self.builder.build_call(f, &values, None)
            .map_err(|err| CompilerError { message: format!("{:?}", err) })
    }

    fn build_if(&mut self, cond: &AstNode, then: &AstNode, el: &Option<AstNode>) -> CodegenResult {
        let cond = {
            let cond = self.codegen(cond)?;
            let zero = self.builder.build_const_fp(self.context.f64_type(), 0.0);

            self.builder.build_fp_cmp(
                RealPredicate::ONE,
                &cond,
                &zero,
                Some(self.pool.intern("ifcond")))
        };

        let mut f = self.builder.get_insert_block().parent();

        let then_block = BasicBlock::new(self.pool.intern("then"), &mut f);
        let else_block = BasicBlock::new(self.pool.intern("else"), &mut f);
        let merge_block = BasicBlock::new(self.pool.intern("ifmerge"), &mut f);

        self.builder.build_conditional_branch(&cond, &then_block, &else_block);

        self.builder.position_at_end(&then_block);
        let then = self.codegen(then)?;
        self.builder.build_branch(&merge_block);

        // Codegen of `then` can change the current block so update `then_block` for the PHI.
        let then_block = self.builder.get_insert_block();

        self.builder.position_at_end(&else_block);
        let el = if let Some(ref el) = el {
            self.codegen(el)?
        } else {
            // FIXME: This is incorrect and should not be reached with a proper type system!
            // The only allowed follow ups should be the same type as the `then` block
            // or a `never` value - return/exit/unimplemented/unreachable.
            self.builder.build_const_fp(self.context.f64_type(), std::f64::INFINITY)
        };
        self.builder.build_branch(&merge_block);

        // Codegen of `else` can change the current block so update `else_block` for the PHI.
        let else_block = self.builder.get_insert_block();

        self.builder.position_at_end(&merge_block);
        let mut phi = self.builder.build_phi(
            self.context.f64_type(),
            Some(self.pool.intern("iftmp"))
        );

        phi.add_incoming(&vec![
            (then, then_block),
            (el, else_block),
        ]);

        Ok(phi.to_value())
    }

    fn build_binop(
        &mut self,
        kind: BinOpKind,
        lhs: &AstNode,
        rhs: &AstNode) -> CodegenResult
    {
        use RealPredicate as RP;
        use BinOpKind::*;

        fn build_fp_cmp(
            compiler: &mut Compiler,
            p: RealPredicate,
            l: &AnyValue,
            r: &AnyValue) -> AnyValue
        {
            let cmp = compiler.builder.build_fp_cmp(p, l, r, Some(compiler.pool.intern("cmptmp")));

            compiler.builder.build_cast_uint_to_fp(
                cmp,
                compiler.context.f64_type(),
                Some(compiler.pool.intern("booltmp")),
            )
        }

        let lhs = self.codegen(&lhs)?;
        let rhs = self.codegen(&rhs)?;

        Ok(match kind {
            Eq           => build_fp_cmp(self, RP::UEQ, &lhs, &rhs),
            NotEq        => build_fp_cmp(self, RP::UNE, &lhs, &rhs),
            GreaterThan  => build_fp_cmp(self, RP::UGT, &lhs, &rhs),
            GreaterEq    => build_fp_cmp(self, RP::UGE, &lhs, &rhs),
            LessThan     => build_fp_cmp(self, RP::ULT, &lhs, &rhs),
            LessEq       => build_fp_cmp(self, RP::ULE, &lhs, &rhs),
            Add          => self.builder.build_fp_add(&lhs, &rhs, Some(self.pool.intern("addtmp"))),
            Sub          => self.builder.build_fp_sub(&lhs, &rhs, Some(self.pool.intern("subtmp"))),
            Mul          => self.builder.build_fp_mul(&lhs, &rhs, Some(self.pool.intern("multmp"))),
        })
    }

    fn build_function(
        &mut self,
        name: &String,
        args: &Vec<String>,
        body: &AstNode) -> CodegenResult
    {
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

        let bb = BasicBlock::new(self.pool.intern("entry"), &mut f);
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

        self.function_optimizer.run(&mut f);

        self.env.defs.insert(name.clone(), f.clone());

        Ok(f.to_value())
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
