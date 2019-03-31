pub mod token;

use lexpar::lex_rules;
use lexpar::lexer::{Lexer, Span};
use self::token::Token;

pub type Term = (Span, Token);

pub fn lexer() -> Lexer<(Span, Token)> {
    use self::Token::*;

    Lexer::new(lex_rules![
        r"[ \t\n]+"                 => |span, text, _| (span, Whitespace(text.to_owned())),
        r"/\*([^(?:*/)]*)\*/"       => |span, _, text| (span, Comment(text[0].to_owned())),
        r"//([^\n]*)"               => |span, _, text| (span, Comment(text[0].to_owned())),

        r"\bfn\b"                       => |span, _, _| (span, KwFn),
        r"\blet\b"                      => |span, _, _| (span, KwLet),
        r"\bif\b"                       => |span, _, _| (span, KwIf),
        r"\bthen\b"                     => |span, _, _| (span, KwThen),
        r"\belse\b"                     => |span, _, _| (span, KwElse),
        r"\bfor\b"                      => |span, _, _| (span, KwFor),
        r"\bin\b"                       => |span, _, _| (span, KwIn),
        r"\bdo\b"                       => |span, _, _| (span, KwDo),
        r"\bor\b"                       => |span, _, _| (span, KwOr),
        r"\band\b"                      => |span, _, _| (span, KwAnd),

        r"[_a-zA-Z][_a-zA-Z0-9]*"   => |span, text, _| (span, Ident(text.to_owned())),
        r"-?[0-9]+(?:\.[0-9]+)?"    => |span, text, _| (span, Number(text.parse().unwrap())),

        r"\("                       => |span, _, _| (span, LParen),
        r"\)"                       => |span, _, _| (span, RParen),
        r"\["                       => |span, _, _| (span, LBracket),
        r"\]"                       => |span, _, _| (span, RBracket),
        r"\{"                       => |span, _, _| (span, LBrace),
        r"\}"                       => |span, _, _| (span, RBrace),

        r"\+"                       => |span, _, _| (span, Plus),
        r"\-"                       => |span, _, _| (span, Minus),
        r"\*"                       => |span, _, _| (span, Asterisk),
        r"/"                        => |span, _, _| (span, FSlash),
        r"=="                       => |span, _, _| (span, Eq),
        r"!="                       => |span, _, _| (span, NotEq),
        r">"                        => |span, _, _| (span, GreaterThan),
        r">="                       => |span, _, _| (span, GreaterEq),
        r"<"                        => |span, _, _| (span, LessThan),
        r"<="                       => |span, _, _| (span, LessEq),
        r"\.\."                     => |span, _, _| (span, Range),

        r"!"                        => |span, _, _| (span, Excl),

        r"->"                       => |span, _, _| (span, Arrow),
        r"="                        => |span, _, _| (span, Assign),
        r":"                        => |span, _, _| (span, Colon),
        r","                        => |span, _, _| (span, Comma),
        r"\|"                       => |span, _, _| (span, Pipe),
        r";"                        => |span, _, _| (span, Semicolon),
        r"'((?:\\'|[^'])*)'"        => |span, _, text| (span, SingleQuote(text[0].to_owned())),
        r#""((?:\\"|[^"])*)""#      => |span, _, text| (span, DoubleQuote(text[0].to_owned())),
    ], |span, text| (span, Unknown(text.to_owned())))
}
