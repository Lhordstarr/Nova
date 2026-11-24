pub mod ast;
pub use ast::*;

use crate::errors::{NovaError, Span};
use crate::lexer::token::{Keyword, Token, TokenKind};

pub fn parse(tokens: &[Token]) -> Result<Program, NovaError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
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
        match self.peek_kind() {
            Some(TokenKind::Keyword(Keyword::Print)) => {
                self.advance();
                if self.check(TokenKind::Newline) || self.check(TokenKind::Dedent) || self.is_eof()
                {
                    return Ok(Stmt::Print(Expr::String(String::new())));
                }

                if let Some(TokenKind::String(value)) = self.peek_kind().cloned() {
                    self.advance();
                    return Ok(Stmt::Print(Expr::String(value)));
                }

                let saved_position = self.position;
                match self.parse_expression() {
                    Ok(expression) if self.is_statement_boundary() => Ok(Stmt::Print(expression)),
                    _ => {
                        self.position = saved_position;
                        let mut has_special = false;
                        let mut text_tokens = Vec::new();
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
                            return Err(self.error_here("invalid expression in print statement"));
                        }

                        if text_tokens.is_empty() {
                            return Ok(Stmt::Print(Expr::String(String::new())));
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
                        Ok(Stmt::Print(Expr::String(text)))
                    }
                }
            }
            Some(TokenKind::Keyword(Keyword::Create)) => {
                self.advance();
                self.expect_keyword(Keyword::Variable, "expected 'variable' after 'create'")?;
                let name = self.expect_ident("expected variable name")?;
                self.expect(TokenKind::Equal, "expected '=' after variable name")?;
                let value = self.parse_expression()?;
                Ok(Stmt::VarDecl { name, value })
            }
            Some(TokenKind::Keyword(Keyword::Ask)) => {
                self.advance();
                let prompt = self.expect_string("expected a string prompt after 'ask'")?;
                self.expect_keyword(Keyword::And, "expected 'and' after prompt")?;
                self.expect_keyword(Keyword::Store, "expected 'store' after 'and'")?;
                self.expect_keyword(Keyword::As, "expected 'as' after 'store'")?;
                let variable = self.expect_ident("expected variable name after 'as'")?;
                Ok(Stmt::Ask { prompt, variable })
            }
            Some(TokenKind::Keyword(Keyword::If)) => {
                self.advance();
                let condition = self.parse_condition()?;
                self.expect(TokenKind::Colon, "expected ':' after condition")?;
                self.expect(TokenKind::Newline, "expected newline after ':'")?;
                self.expect(TokenKind::Indent, "expected an indented block")?;
                let body = self.parse_block()?;
                self.expect(TokenKind::Dedent, "expected end of indented block")?;
                Ok(Stmt::If { condition, body })
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

    fn parse_condition(&mut self) -> Result<Condition, NovaError> {
        let left = self.parse_expression()?;
        let operator = self.parse_compare_op()?;
        let right = self.parse_expression()?;
        Ok(Condition {
            left,
            operator,
            right,
        })
    }

    fn parse_compare_op(&mut self) -> Result<CompareOp, NovaError> {
        self.expect_keyword(Keyword::Is, "expected 'is' in condition")?;
        match self.peek_keyword() {
            Some(Keyword::Greater) => {
                self.advance();
                self.expect_keyword(Keyword::Than, "expected 'than' after 'greater'")?;
                if self.check_keyword(Keyword::Or) {
                    self.advance();
                    self.expect_keyword(Keyword::Equal, "expected 'equal' after 'or'")?;
                    self.expect_keyword(Keyword::To, "expected 'to' after 'equal'")?;
                    Ok(CompareOp::GreaterEqual)
                } else {
                    Ok(CompareOp::Greater)
                }
            }
            Some(Keyword::Less) => {
                self.advance();
                self.expect_keyword(Keyword::Than, "expected 'than' after 'less'")?;
                if self.check_keyword(Keyword::Or) {
                    self.advance();
                    self.expect_keyword(Keyword::Equal, "expected 'equal' after 'or'")?;
                    self.expect_keyword(Keyword::To, "expected 'to' after 'equal'")?;
                    Ok(CompareOp::LessEqual)
                } else {
                    Ok(CompareOp::Less)
                }
            }
            Some(Keyword::Equal) => {
                self.advance();
                self.expect_keyword(Keyword::To, "expected 'to' after 'equal'")?;
                Ok(CompareOp::Equal)
            }
            Some(Keyword::Not) => {
                self.advance();
                self.expect_keyword(Keyword::Equal, "expected 'equal' after 'not'")?;
                self.expect_keyword(Keyword::To, "expected 'to' after 'equal'")?;
                Ok(CompareOp::NotEqual)
            }
            _ => Err(self.error_here("expected a comparison operator after 'is'")),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, NovaError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expr, NovaError> {
        let mut expression = self.parse_multiplicative()?;
        while let Some(op) = self.match_binary_additive() {
            let right = self.parse_multiplicative()?;
            expression = Expr::Binary {
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
            expression = Expr::Binary {
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
            self.advance();
            let expression = self.parse_unary()?;
            return Ok(Expr::Unary {
                operator: UnaryOp::Neg,
                expression: Box::new(expression),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, NovaError> {
        match self.peek_kind().cloned() {
            Some(TokenKind::Number(value)) => {
                self.advance();
                Ok(Expr::Number(value))
            }
            Some(TokenKind::String(value)) => {
                self.advance();
                Ok(Expr::String(value))
            }
            Some(TokenKind::Ident(name)) => {
                self.advance();
                Ok(Expr::Variable(name))
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

    fn peek_keyword(&self) -> Option<Keyword> {
        match self.peek_kind() {
            Some(TokenKind::Keyword(keyword)) => Some(*keyword),
            _ => None,
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(&kind)
    }

    fn check_keyword(&self, keyword: Keyword) -> bool {
        self.peek_keyword() == Some(keyword)
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

    fn expect_ident(&mut self, message: &str) -> Result<String, NovaError> {
        match self.peek_kind().cloned() {
            Some(TokenKind::Ident(name)) => {
                self.advance();
                Ok(name)
            }
            _ => Err(self.error_here(message)),
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
        let span = self
            .tokens
            .get(self.position)
            .map(|token| token.span)
            .unwrap_or(Span { line: 1, column: 1 });
        NovaError::syntax(message, span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_print_statement() {
        let tokens = crate::lexer::tokenize("print 42").unwrap();
        let program = parse(&tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(
            program.statements[0],
            Stmt::Print(Expr::Number(_))
        ));
    }

    #[test]
    fn parses_variable_declaration() {
        let tokens = crate::lexer::tokenize("create variable x = 10").unwrap();
        let program = parse(&tokens).unwrap();
        assert!(matches!(program.statements[0], Stmt::VarDecl { .. }));
    }

    #[test]
    fn parses_if_block() {
        let source = "if x is greater than 1:\n    print x\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parse(&tokens).unwrap();
        assert!(matches!(program.statements[0], Stmt::If { .. }));
    }

    #[test]
    fn parses_operator_precedence() {
        let tokens = crate::lexer::tokenize("print 2 + 3 * 4").unwrap();
        let program = parse(&tokens).unwrap();
        match &program.statements[0] {
            Stmt::Print(Expr::Binary {
                operator: BinaryOp::Add,
                right,
                ..
            }) => match right.as_ref() {
                Expr::Binary {
                    operator: BinaryOp::Mul,
                    ..
                } => {}
                _ => panic!("expected multiplication on right side"),
            },
            _ => panic!("expected additive expression"),
        }
    }

    #[test]
    fn reports_invalid_syntax() {
        let tokens = crate::lexer::tokenize("print +").unwrap();
        assert!(parse(&tokens).is_err());
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
