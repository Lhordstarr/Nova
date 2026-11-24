use std::collections::HashMap;

use crate::errors::NovaError;
use crate::parser::ast::{CompareOp, Expr, Program, Stmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Unknown,
}

pub fn analyze(program: &Program) -> Result<(), NovaError> {
    let mut analyzer = Analyzer {
        scopes: vec![HashMap::new()],
    };
    analyzer.analyze_program(program)
}

struct Analyzer {
    scopes: Vec<HashMap<String, Type>>,
}

impl Analyzer {
    fn analyze_program(&mut self, program: &Program) -> Result<(), NovaError> {
        for statement in &program.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, statement: &Stmt) -> Result<(), NovaError> {
        match statement {
            Stmt::Print { expression, .. } => {
                self.analyze_expression(expression)?;
                Ok(())
            }
            Stmt::VarDecl { name, value, .. } => {
                let value_type = self.analyze_expression(value)?;
                if self.current_scope_contains(name) {
                    return Err(NovaError::semantic(
                        format!("variable '{}' is already declared", name),
                        None,
                    ));
                }
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(name.clone(), value_type);
                Ok(())
            }
            Stmt::Ask { variable, .. } => {
                if self.current_scope_contains(variable) {
                    return Err(NovaError::semantic(
                        format!("variable '{}' is already declared", variable),
                        None,
                    ));
                }
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(variable.clone(), Type::Unknown);
                Ok(())
            }
            Stmt::If {
                condition, body, ..
            } => {
                let condition_type = self.analyze_expression(condition)?;
                if condition_type != Type::Boolean && condition_type != Type::Unknown {
                    return Err(NovaError::semantic(
                        "if condition must be a boolean expression".to_string(),
                        None,
                    ));
                }
                self.scopes.push(HashMap::new());
                for statement in body {
                    self.analyze_statement(statement)?;
                }
                self.scopes.pop();
                Ok(())
            }
        }
    }

    fn analyze_expression(&mut self, expression: &Expr) -> Result<Type, NovaError> {
        match expression {
            Expr::Number { .. } => Ok(Type::Number),
            Expr::String { .. } => Ok(Type::String),
            Expr::Variable { name, .. } => self
                .lookup_type(name)
                .ok_or_else(|| NovaError::semantic(format!("undefined variable '{}'", name), None)),
            Expr::Unary {
                operator: _,
                expression,
                ..
            } => {
                let operand_type = self.analyze_expression(expression)?;
                if operand_type != Type::Number && operand_type != Type::Unknown {
                    return Err(NovaError::semantic(
                        "unary '-' requires a numeric operand".to_string(),
                        None,
                    ));
                }
                if operand_type == Type::Unknown {
                    Ok(Type::Unknown)
                } else {
                    Ok(Type::Number)
                }
            }
            Expr::Compare {
                left,
                operator,
                right,
                ..
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;
                match operator {
                    CompareOp::Greater
                    | CompareOp::GreaterEqual
                    | CompareOp::Less
                    | CompareOp::LessEqual => {
                        if (left_type != Type::Number && left_type != Type::Unknown)
                            || (right_type != Type::Number && right_type != Type::Unknown)
                        {
                            return Err(NovaError::semantic(
                                "comparison requires numeric operands".to_string(),
                                None,
                            ));
                        }
                    }
                    CompareOp::Equal | CompareOp::NotEqual => {
                        if left_type != Type::Unknown
                            && right_type != Type::Unknown
                            && left_type != right_type
                        {
                            return Err(NovaError::semantic(
                                "equality comparison requires operands of the same type"
                                    .to_string(),
                                None,
                            ));
                        }
                    }
                }
                Ok(Type::Boolean)
            }
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;
                match operator {
                    crate::parser::ast::BinaryOp::Add
                    | crate::parser::ast::BinaryOp::Sub
                    | crate::parser::ast::BinaryOp::Mul
                    | crate::parser::ast::BinaryOp::Div => {
                        if (left_type == Type::String && left_type != Type::Unknown)
                            || (right_type == Type::String && right_type != Type::Unknown)
                        {
                            return Err(NovaError::semantic(
                                "arithmetic requires numeric operands".to_string(),
                                None,
                            ));
                        }
                        if left_type == Type::Unknown || right_type == Type::Unknown {
                            Ok(Type::Unknown)
                        } else {
                            Ok(Type::Number)
                        }
                    }
                }
            }
        }
    }

    fn current_scope_contains(&self, name: &str) -> bool {
        self.scopes.last().unwrap().contains_key(name)
    }

    fn lookup_type(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(value_type) = scope.get(name) {
                return Some(*value_type);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn accepts_valid_program() {
        let source = "create variable x = 10\nprint x\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        assert!(analyze(&program).is_ok());
    }

    #[test]
    fn rejects_undefined_variable() {
        let source = "print x + 1\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        assert!(analyze(&program).is_err());
    }

    #[test]
    fn rejects_duplicate_variable() {
        let source = "create variable x = 1\ncreate variable x = 2\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        assert!(analyze(&program).is_err());
    }

    #[test]
    fn rejects_type_mismatch() {
        let source = "create variable x = \"a\"\nprint x + 1\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        assert!(analyze(&program).is_err());
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;
    use crate::parser;

    #[test]
    fn stores_boolean_comparison() {
        let source =
            "create variable age = 20\ncreate variable is_adult = age is greater than 18\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        assert!(analyze(&program).is_ok());
    }

    #[test]
    fn rejects_non_boolean_if() {
        let source = "if 5:\n    print \"x\"\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        assert!(analyze(&program).is_err());
    }

    #[test]
    fn rejects_undefined_in_expression() {
        let source = "print first + unknown\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        assert!(analyze(&program).is_err());
    }
}
