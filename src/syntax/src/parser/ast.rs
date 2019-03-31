use lexpar::lexer::Span;

#[derive(Debug)]
pub struct AstNode {
    pub span: Span,
    pub expr: Box<Ast>,
}

impl AstNode {
    pub fn new(span: Span, ast: Ast) -> Self {
        Self {
            span,
            expr: Box::new(ast),
        }
    }
}

#[derive(Debug)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Eq,
    NotEq,
    LessThan,
    LessEq,
    GreaterThan,
    GreaterEq,
}

#[derive(Debug)]
pub enum Ast {
    Number(f64),
    Ref(String),
    Block(Vec<AstNode>),
    Function {
        prototype: Prototype,
        body: AstNode,
    },
    Call {
        name: String,
        args: Vec<AstNode>,
    },
    Variable {
        name: String,
        expr: AstNode,
    },
    BinOp {
        kind: BinOpKind,
        lhs: AstNode,
        rhs: AstNode,
    },
    If {
        condition: AstNode,
        then: AstNode,
        el: Option<AstNode>,
    },
}
