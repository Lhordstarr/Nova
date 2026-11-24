use std::io::{Read, Write};

use crate::errors::NovaError;
use crate::parser::ast::{BinaryOp, CompareOp, Expr, Program, Stmt, UnaryOp};
use crate::runtime::{Environment, Value};

pub fn execute(program: &Program) -> Result<(), NovaError> {
    let mut input = std::io::stdin();
    let mut output = std::io::stdout();
    execute_io(program, &mut input, &mut output)
}

pub fn execute_io<R: Read, W: Write>(
    program: &Program,
    input: &mut R,
    output: &mut W,
) -> Result<(), NovaError> {
    let mut interpreter = Interpreter {
        environment: Environment::new(),
        input,
        output,
    };
    interpreter.run_program(program)
}

struct Interpreter<'a, R: Read, W: Write> {
    environment: Environment,
    input: &'a mut R,
    output: &'a mut W,
}

impl<'a, R: Read, W: Write> Interpreter<'a, R, W> {
    fn run_program(&mut self, program: &Program) -> Result<(), NovaError> {
        for statement in &program.statements {
            self.run_statement(statement)?;
        }
        Ok(())
    }

    fn run_statement(&mut self, statement: &Stmt) -> Result<(), NovaError> {
        match statement {
            Stmt::Print { expression, .. } => {
                let value = self.eval_expression(expression)?;
                writeln!(self.output, "{}", value)?;
                self.output.flush()?;
                Ok(())
            }
            Stmt::VarDecl { name, value, .. } => {
                let value = self.eval_expression(value)?;
                self.environment.declare(name.clone(), value)
            }
            Stmt::Ask {
                prompt, variable, ..
            } => {
                write!(self.output, "{}", prompt)?;
                self.output.flush()?;
                let mut line = read_one_line(self.input)?;
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                let value = match line.trim().parse::<f64>() {
                    Ok(number) if number.is_finite() => Value::Number(number),
                    _ => Value::String(line),
                };
                self.environment.declare(variable.clone(), value)
            }
            Stmt::If {
                condition, body, ..
            } => {
                let value = self.eval_expression(condition)?;
                let is_true = match value {
                    Value::Boolean(value) => value,
                    _ => return Err(NovaError::runtime("if condition must be a boolean")),
                };
                if is_true {
                    self.environment.push_scope();
                    for statement in body {
                        self.run_statement(statement)?;
                    }
                    self.environment.pop_scope();
                }
                Ok(())
            }
        }
    }

    fn eval_expression(&mut self, expression: &Expr) -> Result<Value, NovaError> {
        match expression {
            Expr::Number { value, .. } => Ok(Value::Number(*value)),
            Expr::String { value, .. } => Ok(Value::String(value.clone())),
            Expr::Variable { name, .. } => self.environment.get(name),
            Expr::Unary {
                operator: UnaryOp::Neg,
                expression,
                ..
            } => {
                let value = self.eval_expression(expression)?;
                match value {
                    Value::Number(number) => Ok(Value::Number(-number)),
                    _ => Err(NovaError::runtime("unary '-' requires a number")),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let left_value = self.eval_expression(left)?;
                let right_value = self.eval_expression(right)?;
                match (left_value, right_value) {
                    (Value::Number(left), Value::Number(right)) => match operator {
                        BinaryOp::Add => Ok(Value::Number(left + right)),
                        BinaryOp::Sub => Ok(Value::Number(left - right)),
                        BinaryOp::Mul => Ok(Value::Number(left * right)),
                        BinaryOp::Div => {
                            if right == 0.0 {
                                Err(NovaError::runtime("division by zero"))
                            } else {
                                Ok(Value::Number(left / right))
                            }
                        }
                    },
                    _ => Err(NovaError::runtime("arithmetic requires numeric values")),
                }
            }
            Expr::Compare {
                left,
                operator,
                right,
                ..
            } => {
                let left_value = self.eval_expression(left)?;
                let right_value = self.eval_expression(right)?;
                match operator {
                    CompareOp::Greater => match (left_value, right_value) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                        _ => Err(NovaError::runtime("comparison requires numbers")),
                    },
                    CompareOp::GreaterEqual => match (left_value, right_value) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
                        _ => Err(NovaError::runtime("comparison requires numbers")),
                    },
                    CompareOp::Less => match (left_value, right_value) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                        _ => Err(NovaError::runtime("comparison requires numbers")),
                    },
                    CompareOp::LessEqual => match (left_value, right_value) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
                        _ => Err(NovaError::runtime("comparison requires numbers")),
                    },
                    CompareOp::Equal => Ok(Value::Boolean(left_value == right_value)),
                    CompareOp::NotEqual => Ok(Value::Boolean(left_value != right_value)),
                }
            }
        }
    }
}

fn read_one_line<R: std::io::Read>(input: &mut R) -> Result<String, NovaError> {
    let mut bytes = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match input.read(&mut byte)? {
            0 => break,
            _ => {
                bytes.push(byte[0]);
                if byte[0] == b'\n' {
                    break;
                }
            }
        }
    }
    String::from_utf8(bytes).map_err(|_| NovaError::runtime("input is not valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use std::io::Cursor;

    #[test]
    fn prints_expected_output() {
        let source = "print \"Hello, World!\"\ncreate variable name = \"Sharif\"\nprint name\ncreate variable first = 20\ncreate variable second = 5\nprint first + second\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        crate::semantic::analyze(&program).unwrap();
        let mut input = Cursor::new("");
        let mut output = Vec::new();
        execute_io(&program, &mut input, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Hello, World!\nSharif\n25\n"
        );
    }

    #[test]
    fn handles_user_input() {
        let source = "ask \"What is your name?\" and store as username\nprint username\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        crate::semantic::analyze(&program).unwrap();
        let mut input = Cursor::new("Sharif\n");
        let mut output = Vec::new();
        execute_io(&program, &mut input, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "What is your name?Sharif\n"
        );
    }

    #[test]
    fn reports_division_by_zero() {
        let source = "print 1 / 0\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        crate::semantic::analyze(&program).unwrap();
        let mut input = Cursor::new("");
        let mut output = Vec::new();
        assert!(execute_io(&program, &mut input, &mut output).is_err());
    }

    #[test]
    fn reports_undefined_variable_at_runtime_if_needed() {
        let source = "print x + 1\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        // Semantic analysis should catch this, but runtime is also defensive.
        assert!(crate::semantic::analyze(&program).is_err());
    }
}

#[cfg(test)]
mod bare_text_tests {
    use super::*;
    use crate::parser;
    use std::io::Cursor;

    fn run(source: &str, input: &str) -> (Result<(), NovaError>, String) {
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        crate::semantic::analyze(&program).unwrap();
        let mut input = Cursor::new(input);
        let mut output = Vec::new();
        let result = execute_io(&program, &mut input, &mut output);
        (result, String::from_utf8(output).unwrap())
    }

    #[test]
    fn bare_text_prints_text() {
        let (result, output) = run("print Hello, World!\n", "");
        assert!(result.is_ok());
        assert_eq!(output, "Hello, World!\n");
    }

    #[test]
    fn bare_identifier_variable_prints_value() {
        let (result, output) = run(
            "create variable username = \"Sharif\"\nprint username\n",
            "",
        );
        assert!(result.is_ok());
        assert_eq!(output, "Sharif\n");
    }

    #[test]
    fn bare_identifier_unknown_is_error() {
        let tokens = crate::lexer::tokenize("print unknownname\n").unwrap();
        let program = crate::parser::parse(&tokens).unwrap();
        assert!(crate::semantic::analyze(&program).is_err());
    }

    #[test]
    fn expression_still_evaluates() {
        let (result, output) = run("create variable first = 20\ncreate variable second = 5\nprint first + second\nprint (20 + 5) * 2\n", "");
        assert!(result.is_ok());
        assert_eq!(output, "25\n50\n");
    }

    #[test]
    fn quoted_string_still_works() {
        let (result, output) = run("print \"Hello, World!\"\n", "");
        assert!(result.is_ok());
        assert_eq!(output, "Hello, World!\n");
    }

    #[test]
    fn malformed_expression_errors() {
        let cases = [
            "print unknown + 5\n",
            "print 20 +\n",
            "print (20\n",
            "print 20 )\n",
        ];
        for case in cases {
            let tokens = match crate::lexer::tokenize(case) {
                Ok(tokens) => tokens,
                Err(_) => continue,
            };
            let program = match parser::parse(&tokens) {
                Ok(program) => program,
                Err(_) => continue,
            };
            if crate::semantic::analyze(&program).is_err() {
                continue;
            }
            let mut input = Cursor::new("");
            let mut output = Vec::new();
            assert!(
                execute_io(&program, &mut input, &mut output).is_err(),
                "{}",
                case
            );
        }
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;
    use crate::parser;
    use std::io::Cursor;

    #[test]
    fn ask_parses_number() {
        let source = "ask \"n\" and store as x\nprint x + 1\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        crate::semantic::analyze(&program).unwrap();
        let mut input = Cursor::new("10\n");
        let mut output = Vec::new();
        execute_io(&program, &mut input, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "n11\n");
    }

    #[test]
    fn ask_rejects_nan_as_number() {
        let source = "ask \"n\" and store as x\nprint x\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        crate::semantic::analyze(&program).unwrap();
        let mut input = Cursor::new("NaN\n");
        let mut output = Vec::new();
        execute_io(&program, &mut input, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "nNaN\n");
    }

    #[test]
    fn prints_boolean() {
        let source =
            "create variable age = 20\ncreate variable a = age is greater than 18\nprint a\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        crate::semantic::analyze(&program).unwrap();
        let mut input = Cursor::new("");
        let mut output = Vec::new();
        execute_io(&program, &mut input, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "true\n");
    }

    #[test]
    fn unary_minus_works() {
        let source = "print -5\nprint -(2 + 3)\n";
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        crate::semantic::analyze(&program).unwrap();
        let mut input = Cursor::new("");
        let mut output = Vec::new();
        execute_io(&program, &mut input, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "-5\n-5\n");
    }
}
