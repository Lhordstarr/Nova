use crate::errors::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Newline,
    Indent,
    Dedent,
    Eof,

    Plus,
    Minus,
    Star,
    Slash,
    Colon,
    Equal,
    LParen,
    RParen,

    String(String),
    Number(f64),
    Ident(String),

    Keyword(Keyword),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Print,
    Create,
    Variable,
    If,
    Ask,
    And,
    Store,
    As,
    Is,
    Greater,
    Than,
    Less,
    Equal,
    Not,
    Or,
    To,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
