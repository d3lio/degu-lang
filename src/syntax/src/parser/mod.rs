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

    // Statements
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

    // T0 expr (Binary operations with a precedence algorithm)
    #[binop(infix)]
    expr: AstNode => _expr where u32 => |lhs, rhs| {
        &(_, Eq)            | 0 => create_binop(BinOpKind::Eq, lhs, rhs),
        &(_, NotEq)         | 0 => create_binop(BinOpKind::NotEq, lhs, rhs),
        &(_, GreaterThan)   | 0 => create_binop(BinOpKind::GreaterThan, lhs, rhs),
        &(_, GreaterEq)     | 0 => create_binop(BinOpKind::GreaterEq, lhs, rhs),
        &(_, LessThan)      | 0 => create_binop(BinOpKind::LessThan, lhs, rhs),
        &(_, LessEq)        | 0 => create_binop(BinOpKind::LessEq, lhs, rhs),
        &(_, Plus)          | 1 => create_binop(BinOpKind::Add, lhs, rhs),
        &(_, Minus)         | 1 => create_binop(BinOpKind::Sub, lhs, rhs),
        &(_, Asterisk)      | 2 => create_binop(BinOpKind::Mul, lhs, rhs),
    },

    // T1 expr (Compound expressions)
    _expr: AstNode => {
        // TODO: move this out of binary operations.
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

        [_if: _if] => _if,
        [ex: __expr] => ex,
    },

    // T2 expr (Simple expressions)
    __expr: AstNode => {
        // Reference (Function call argument)
        [(span, Ident(name))] => AstNode::new(span, Ast::Ref(name)),

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

// If-else expressions
parse_rules! {
    term: Term;

    _if: AstNode => {
        [(mut span, KwIf), condition: expr, (_, KwThen), then: expr, el: _else] => {
            if let Some(ref el) = el {
                span.hi = el.span.hi;
            } else {
                span.hi = then.span.hi;
            }

            AstNode::new(span, Ast::If {
                condition,
                then,
                el,
            })
        },
    },

    _else: Option<AstNode> => {
        [(_, KwElse), ex: expr] => Some(ex),
        [@] => None,
    },
}

// Reference or function invocation helpers
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

// Function definition helpers
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
