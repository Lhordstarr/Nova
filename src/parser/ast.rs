use crate::errors::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print {
        span: Span,
        expression: Expr,
    },
    VarDecl {
        span: Span,
        name: String,
        value: Expr,
    },
    Ask {
        span: Span,
        prompt: String,
        variable: String,
    },
    If {
        span: Span,
        condition: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number {
        span: Span,
        value: f64,
    },
    String {
        span: Span,
        value: String,
    },
    Variable {
        span: Span,
        name: String,
    },
    Unary {
        span: Span,
        operator: UnaryOp,
        expression: Box<Expr>,
    },
    Binary {
        span: Span,
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Compare {
        span: Span,
        left: Box<Expr>,
        operator: CompareOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
}
