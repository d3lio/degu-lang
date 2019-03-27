mod lexer;
mod parser;
mod compiler;

use lexpar::lexer::Span;
use lexpar::parser::ParseError;

use lexer::Term;
use compiler::Compiler;
use parser::ast::{Ast, AstNode};

const SRC: &str = r#"
let sum_three a b c =
    a + b + c

let main _ = print_number (sum_three 1 2 3)
"#;

pub fn run(source: &str) -> Result<(), ParseError<Term>> {
    println!("== main.dg ==\n{}", source);

    let lexer = lexer::lexer();
    let iter = lexer.src_iter(source);
    let ast = parser::Parser::parse(iter)?;
    println!("== ast ==\n\n{:?}\n", ast);

    let mut compiler = Compiler::new();

    let ast = AstNode::new(Span::new(0, source.len(), 0), Ast::Block(ast));

    compiler.compile(&ast);
    println!("== llvm ir ==\n\n{:?}", compiler.module());

    Ok(())
}

fn main() {
    if let Err(err) = run(SRC) {
        println!("{:?}", err);
    }
}
