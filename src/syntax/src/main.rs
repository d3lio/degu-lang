mod lexer;
mod parser;

const SRC: &str = r#"
let sum_three a b c =
    a + b + c

let main _ = print_u32 (sum_three 1 2 3)
"#;

pub fn run(source: &str) {
    let lexer = lexer::lexer();
    let iter = lexer.src_iter(source);
    let ast = parser::Parser::parse(iter);

    match ast {
        Ok(exprs) => {
            println!("{:?}", exprs);
        },
        Err(err) => println!("{:?}", err),
    }
}

fn main() {
    run(SRC);
}
