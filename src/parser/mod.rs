pub mod ast;
pub use ast::*;

use crate::errors::{NovaError, Span};
use crate::lexer::token::{Keyword, Token, TokenKind};

pub fn parse(tokens: &[Token]) -> Result<Program, NovaError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number { span, .. }
            | Expr::String { span, .. }
            | Expr::Variable { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Compare { span, .. } => *span,
        }
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse_program(&mut self) -> Result<Program, NovaError> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.is_eof() {
            if self.check(TokenKind::Dedent) {
                return Err(self.error_here("unexpected dedent"));
            }
            if self.check(TokenKind::Indent) {
                return Err(self.error_here("unexpected indent"));
            }
            let statement = self.parse_statement()?;
            statements.push(statement);
            self.skip_newlines();
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, NovaError> {
        let statement_span = self.current_span();
        if let Some(TokenKind::Ident(_)) = self.peek_kind() {
            if self.peek_next_kind() == Some(TokenKind::Equal) {
                return Err(NovaError::syntax(
                    "variable reassignment is not supported in NOVA 0.1; use 'create variable' to declare a variable",
                    self.current_span(),
                ));
            }
        }
        match self.peek_kind() {
            Some(TokenKind::Keyword(Keyword::Print)) => {
                self.advance();
                let print_span = statement_span;
                if self.check(TokenKind::Newline) || self.check(TokenKind::Dedent) || self.is_eof()
                {
                    return Ok(Stmt::Print {
                        span: print_span,
                        expression: Expr::String {
                            span: print_span,
                            value: String::new(),
                        },
                    });
                }

                if let Some(TokenKind::String(value)) = self.peek_kind().cloned() {
                    let token_span = self.current_span();
                    self.advance();
                    return Ok(Stmt::Print {
                        span: print_span,
                        expression: Expr::String {
                            span: token_span,
                            value,
                        },
                    });
                }

                let saved_position = self.position;
                match self.parse_expression() {
                    Ok(expression) if self.is_statement_boundary() => Ok(Stmt::Print {
                        span: print_span,
                        expression,
                    }),
                    _ => {
                        self.position = saved_position;
                        let mut has_special = false;
                        let mut text_tokens = Vec::new();
                        let text_span = self.current_span();
                        while !self.is_statement_boundary() {
                            match self.peek_kind() {
                                Some(TokenKind::Plus)
                                | Some(TokenKind::Minus)
                                | Some(TokenKind::Star)
                                | Some(TokenKind::Slash)
                                | Some(TokenKind::LParen)
                                | Some(TokenKind::RParen)
                                | Some(TokenKind::Equal)
                                | Some(TokenKind::Colon) => {
                                    has_special = true;
                                }
                                Some(_) => {}
                                None => break,
                            }
                            if let Some(kind) = self.peek_kind().cloned() {
                                text_tokens.push(kind);
                            }
                            self.advance();
                        }

                        if has_special {
                            return Err(NovaError::syntax(
                                "invalid expression in print statement",
                                text_span,
                            ));
                        }

                        if text_tokens.is_empty() {
                            return Ok(Stmt::Print {
                                span: print_span,
                                expression: Expr::String {
                                    span: print_span,
                                    value: String::new(),
                                },
                            });
                        }

                        let mut text = String::new();
                        for (index, kind) in text_tokens.iter().enumerate() {
                            if index > 0 {
                                text.push(' ');
                            }
                            match kind {
                                TokenKind::Ident(value) => text.push_str(value),
                                TokenKind::Keyword(keyword) => {
                                    text.push_str(keyword_as_text(*keyword))
                                }
                                TokenKind::Number(value) => text.push_str(&value.to_string()),
                                TokenKind::String(value) => text.push_str(value),
                                _ => text.push_str("?"),
                            }
                        }
                        Ok(Stmt::Print {
                            span: print_span,
                            expression: Expr::String {
                                span: text_span,
                                value: text,
                            },
                        })
                    }
                }
            }
            Some(TokenKind::Keyword(Keyword::Create)) => {
                self.advance();
                self.expect_keyword(Keyword::Variable, "expected 'variable' after 'create'")?;
                let name = self.expect_variable_name("expected variable name")?;
                self.expect(TokenKind::Equal, "expected '=' after variable name")?;
                let value = self.parse_expression()?;
                Ok(Stmt::VarDecl {
                    span: statement_span,
                    name,
                    value,
                })
            }
            Some(TokenKind::Keyword(Keyword::Ask)) => {
                self.advance();
                let prompt = self.expect_string("expected a string prompt after 'ask'")?;
                self.expect_keyword(Keyword::And, "expected 'and' after prompt")?;
                self.expect_keyword(Keyword::Store, "expected 'store' after 'and'")?;
                self.expect_keyword(Keyword::As, "expected 'as' after 'store'")?;
                let variable = self.expect_variable_name("expected variable name after 'as'")?;
                Ok(Stmt::Ask {
                    span: statement_span,
                    prompt,
                    variable,
                })
            }
            Some(TokenKind::Keyword(Keyword::If)) => {
                self.advance();
                let condition = self.parse_expression()?;
                self.expect(TokenKind::Colon, "expected ':' after condition")?;
                self.expect(TokenKind::Newline, "expected newline after ':'")?;
                self.expect(TokenKind::Indent, "expected an indented block")?;
                let body = self.parse_block()?;
                self.expect(TokenKind::Dedent, "expected end of indented block")?;
                Ok(Stmt::If {
                    span: statement_span,
                    condition,
                    body,
                })
            }
            _ => Err(self.error_here("expected a statement")),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, NovaError> {
        self.skip_newlines();
        if self.check(TokenKind::Dedent) {
            return Err(self.error_here("expected at least one statement in block"));
        }
        let mut statements = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_eof() {
            let statement = self.parse_statement()?;
            statements.push(statement);
            self.skip_newlines();
        }
        if statements.is_empty() {
            return Err(self.error_here("expected at least one statement in block"));
        }
        Ok(statements)
    }

    fn parse_expression(&mut self) -> Result<Expr, NovaError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, NovaError> {
        let left = self.parse_additive()?;
        if let Some(operator) = self.parse_compare_op() {
            let right = self.parse_additive()?;
            let span = left.span();
            return Ok(Expr::Compare {
                span,
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_compare_op(&mut self) -> Option<CompareOp> {
        if !self.check_keyword(Keyword::Is) {
            return None;
        }
        self.advance();
        match self.peek_keyword() {
            Some(Keyword::Greater) => {
                self.advance();
                if !self.check_keyword(Keyword::Than) {
                    // Back up so the caller can report a useful error if needed.
                    return None;
                }
                self.advance();
                if self.check_keyword(Keyword::Or) {
                    self.advance();
                    if !self.check_keyword(Keyword::Equal) {
                        return None;
                    }
                    self.advance();
                    if !self.check_keyword(Keyword::To) {
                        return None;
                    }
                    self.advance();
                    Some(CompareOp::GreaterEqual)
                } else {
                    Some(CompareOp::Greater)
                }
            }
            Some(Keyword::Less) => {
                self.advance();
                if !self.check_keyword(Keyword::Than) {
                    return None;
                }
                self.advance();
                if self.check_keyword(Keyword::Or) {
                    self.advance();
                    if !self.check_keyword(Keyword::Equal) {
                        return None;
                    }
                    self.advance();
                    if !self.check_keyword(Keyword::To) {
                        return None;
                    }
                    self.advance();
                    Some(CompareOp::LessEqual)
                } else {
                    Some(CompareOp::Less)
                }
            }
            Some(Keyword::Equal) => {
                self.advance();
                if !self.check_keyword(Keyword::To) {
                    return None;
                }
                self.advance();
                Some(CompareOp::Equal)
            }
            Some(Keyword::Not) => {
                self.advance();
                if !self.check_keyword(Keyword::Equal) {
                    return None;
                }
                self.advance();
                if !self.check_keyword(Keyword::To) {
                    return None;
                }
                self.advance();
                Some(CompareOp::NotEqual)
            }
            _ => None,
        }
    }

    fn parse_additive(&mut self) -> Result<Expr, NovaError> {
        let mut expression = self.parse_multiplicative()?;
        while let Some(op) = self.match_binary_additive() {
            let right = self.parse_multiplicative()?;
            let span = expression.span();
            expression = Expr::Binary {
                span,
                left: Box::new(expression),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn match_binary_additive(&mut self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Plus) => {
                self.advance();
                Some(BinaryOp::Add)
            }
            Some(TokenKind::Minus) => {
                self.advance();
                Some(BinaryOp::Sub)
            }
            _ => None,
        }
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, NovaError> {
        let mut expression = self.parse_unary()?;
        while let Some(op) = self.match_binary_multiplicative() {
            let right = self.parse_unary()?;
            let span = expression.span();
            expression = Expr::Binary {
                span,
                left: Box::new(expression),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn match_binary_multiplicative(&mut self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Star) => {
                self.advance();
                Some(BinaryOp::Mul)
            }
            Some(TokenKind::Slash) => {
                self.advance();
                Some(BinaryOp::Div)
            }
            _ => None,
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, NovaError> {
        if self.check(TokenKind::Minus) {
            let span = self.current_span();
            self.advance();
            let expression = self.parse_unary()?;
            return Ok(Expr::Unary {
                span,
                operator: UnaryOp::Neg,
                expression: Box::new(expression),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, NovaError> {
        match self.peek_kind().cloned() {
            Some(TokenKind::Number(value)) => {
                let span = self.current_span();
                self.advance();
                Ok(Expr::Number { span, value })
            }
            Some(TokenKind::String(value)) => {
                let span = self.current_span();
                self.advance();
                Ok(Expr::String { span, value })
            }
            Some(TokenKind::Ident(name)) => {
                let span = self.current_span();
                self.advance();
                Ok(Expr::Variable { span, name })
            }
            Some(TokenKind::LParen) => {
                self.advance();
                let expression = self.parse_expression()?;
                self.expect(TokenKind::RParen, "expected ')' after expression")?;
                Ok(expression)
            }
            _ => Err(self.error_here("expected a number, string, variable, or '('")),
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.advance();
        }
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.position).map(|token| &token.kind)
    }

    fn peek_next_kind(&self) -> Option<TokenKind> {
        self.tokens
            .get(self.position + 1)
            .map(|token| token.kind.clone())
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(&kind)
    }

    fn check_keyword(&self, keyword: Keyword) -> bool {
        self.peek_keyword() == Some(keyword)
    }

    fn peek_keyword(&self) -> Option<Keyword> {
        match self.peek_kind() {
            Some(TokenKind::Keyword(keyword)) => Some(*keyword),
            _ => None,
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<(), NovaError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn expect_keyword(&mut self, keyword: Keyword, message: &str) -> Result<(), NovaError> {
        if self.check_keyword(keyword) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn expect_string(&mut self, message: &str) -> Result<String, NovaError> {
        match self.peek_kind().cloned() {
            Some(TokenKind::String(value)) => {
                self.advance();
                Ok(value)
            }
            _ => Err(self.error_here(message)),
        }
    }

    fn expect_variable_name(&mut self, message: &str) -> Result<String, NovaError> {
        match self.peek_kind().cloned() {
            Some(TokenKind::Ident(name)) => {
                self.advance();
                Ok(name)
            }
            Some(TokenKind::Keyword(keyword)) => Err(NovaError::syntax(
                format!(
                    "'{}' is a reserved keyword and cannot be used as a variable name",
                    keyword_as_text(keyword)
                ),
                self.current_span(),
            )),
            _ => Err(self.error_here(message)),
        }
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.position)
            .map(|token| token.span)
            .unwrap_or(Span { line: 1, column: 1 })
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Eof) | None)
    }

    fn is_statement_boundary(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Newline) | Some(TokenKind::Dedent) | Some(TokenKind::Eof) | None
        )
    }

    fn error_here(&self, message: &str) -> NovaError {
        NovaError::syntax(message, self.current_span())
    }
}

fn keyword_as_text(keyword: Keyword) -> &'static str {
    match keyword {
        Keyword::Print => "print",
        Keyword::Create => "create",
        Keyword::Variable => "variable",
        Keyword::If => "if",
        Keyword::Ask => "ask",
        Keyword::And => "and",
        Keyword::Store => "store",
        Keyword::As => "as",
        Keyword::Is => "is",
        Keyword::Greater => "greater",
        Keyword::Than => "than",
        Keyword::Less => "less",
        Keyword::Equal => "equal",
        Keyword::Not => "not",
        Keyword::Or => "or",
        Keyword::To => "to",
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    #[test]
    fn supports_unary_minus() {
        let tokens = crate::lexer::tokenize("print -(2 + 3)").unwrap();
        let program = parse(&tokens).unwrap();
        assert!(matches!(program.statements[0], Stmt::Print { .. }));
    }

    #[test]
    fn rejects_reassignment() {
        let tokens = crate::lexer::tokenize("create variable x = 1\nx = 2\n").unwrap();
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn rejects_reserved_keyword_variable() {
        let tokens = crate::lexer::tokenize("create variable print = 1\n").unwrap();
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn rejects_empty_block() {
        let tokens = crate::lexer::tokenize("if 1 is equal to 1:\n").unwrap();
        assert!(parse(&tokens).is_err());
    }
}
