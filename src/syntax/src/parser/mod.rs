pub mod ast;

mod transform;

use lexpar::parse_rules;
use lexpar::lexer::{LexIter, Span};
use lexpar::parser::{ParseError, UnexpectedKind};

use super::lexer::Term;
use super::lexer::token::Token;
use super::lexer::token::Token::*;

use self::transform::BlockIterator;

use self::ast::*;

pub struct Parser;

impl Parser {
    pub fn parse(iter: LexIter<Term>) -> lexpar::parser::Result<Vec<AstNode>, Term> {
        let iter = iter
            .blocks()
            .filter(|x| match *x {
                (_, Token::Whitespace(_)) => false,
                (_, Token::Comment(_)) => false,
                _ => true
            });

        top_level(&mut iter.into())
    }
}

fn merge(a: Span, b: Span) -> Span {
    a.extend(b.hi)
}

fn create_binop(kind: BinOpKind, lhs: AstNode, rhs: AstNode) -> AstNode {
    let span = Span::new(lhs.span.lo, rhs.span.hi, lhs.span.line);
    AstNode::new(span, Ast::BinOp {
        kind,
        lhs,
        rhs,
    })
}

parse_rules! {
    term: Term;

    top_level: Vec<AstNode> => |iter| {
        let items = _top_level(iter)?;

        // Explicit error handling because fold never fails with UnexpectedRoot because it must act
        // as a Kleene star but in this case we keep matching until eof otherwise it's an error
        if let Some(token) = iter.next() {
            Err(ParseError::Unexpected {
                kind: UnexpectedKind::Root,
                nonterm: "top_level",
                token,
            })
        } else {
            Ok(items)
        }
    },

    #[fold(nodes)]
    _top_level: Vec<AstNode> => {
        [node: __top_level] => {
            if let Some(node) = node {
                nodes.push(node);
            }
            nodes
        },
        [@] => Vec::new()
    },

    __top_level: Option<AstNode> => {
        [def: def] => Some(def),
        [expr: expr] => Some(expr),
        [(_, BlockCont)] => None
    },
}

// Definition and expression parsing
parse_rules! {
    term: Term;

    def: AstNode => {
        [(span, KwLet), (_, Ident(name)), params: params, (_, Assign), ex: expr] => {
            let span = span.extend(ex.span.hi);

            if params.is_empty() {
                AstNode::new(span, Ast::Variable {
                    name,
                    expr: ex,
                })
            } else {
                AstNode::new(span, Ast::Function {
                    prototype: Prototype {
                        name,
                        args: params,
                    },
                    body: ex,
                })
            }
        }
    },

    #[binop(infix)]
    expr: AstNode => _expr where u32 => |lhs, rhs| {
        &(_, Plus)       | 0 => create_binop(BinOpKind::Add, lhs, rhs),
        &(_, Minus)      | 0 => create_binop(BinOpKind::Sub, lhs, rhs),
        &(_, Asterisk)   | 1 => create_binop(BinOpKind::Mul, lhs, rhs),
    },

    _expr: AstNode => {
        [ex: __expr] => ex,

        // Block expression
        [(l, BlockStart), top: _top_level, (r, BlockEnd)] => {
            AstNode::new(merge(l, r), Ast::Block(top))
        },

        // Reference or function call
        [(span, Ident(name)), args: args] => {
            if let Some((call_span, args)) = args {
                AstNode::new(merge(span, call_span), Ast::Call { name, args })
            } else {
                AstNode::new(span, Ast::Ref(name))
            }
        },
    },

    __expr: AstNode => {
        // Parenthesis expression
        [(_, LParen), ex: expr, (_, RParen)] => ex,

        // Literal expression
        [literal: literal] => literal,
    },

    literal: AstNode => {
        // Number literal
        [(span, Number(num))] => AstNode::new(span, Ast::Number(num)),
    },
}

// Reference or function invocation
parse_rules! {
    term: Term;

    // Reference or function call
    args: Option<(Span, Vec<AstNode>)> => {
        [args: _args] => args
            .last()
            .map(|last| Span::new(args[0].span.lo, last.span.hi, args[0].span.line))
            .map(|span| (span, args))
    },

    // Function call arguments
    #[fold(args)]
    _args: Vec<AstNode> => {
        [ex: __expr] => {
            args.push(ex);
            args
        },
        [@] => Vec::new()
    },
}

// Functions
parse_rules! {
    term: Term;

    // Function parameters
    #[fold(params)]
    params: Vec<String> => {
        [(_, Ident(name))] => {
            params.push(name);
            params
        },
        [@] => Vec::new()
    },
}
