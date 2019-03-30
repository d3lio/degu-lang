mod compiler;

use lexpar::lexer::Span;

use syntax::lexer;
use syntax::parser::Parser;
use syntax::parser::ast::{Ast, AstNode};

use std::fs::File;
use std::io::BufReader;
use std::io::{self, prelude::*};

use self::compiler::Compiler;

fn read_file(name: &str) -> io::Result<String> {
    let file = File::open(name)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

fn run(source: &str) {
    let lexer = lexer::lexer();
    let iter = lexer.src_iter(source);
    let ast = Parser::parse(iter).unwrap();
    // println!("== ast ==\n\n{:?}\n", ast);

    let mut compiler = Compiler::new();

    let ast = AstNode::new(Span::new(0, source.len(), 0), Ast::Block(ast));

    compiler.compile(&ast).unwrap();
    println!("== llvm ir ==\n\n{:?}", compiler.module());

    println!("== runtime ==\n");
    let mut runtime = compiler.into_runtime();
    runtime.run_main();
}

fn main() -> io::Result<()> {
    let name = "main.dg";
    let source = read_file(name)?;

    println!("== {} ==\n\n{}", name, source);

    run(&source);

    Ok(())
}
