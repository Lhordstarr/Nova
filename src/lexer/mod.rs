pub mod token;
pub use token::*;

use crate::errors::{NovaError, Span};

pub fn tokenize(source: &str) -> Result<Vec<Token>, NovaError> {
    let lines: Vec<&str> = source.lines().collect();
    let mut tokens = Vec::new();
    let mut indent_stack = vec![0usize];

    for (line_idx, raw_line) in lines.iter().enumerate() {
        let line_no = line_idx + 1;
        let chars: Vec<char> = raw_line.chars().collect();

        if chars.iter().all(|c| c.is_whitespace()) {
            continue;
        }

        let mut indent_units = 0usize;
        let mut i = 0usize;
        while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
            if chars[i] == '\t' {
                return Err(NovaError::lexical(
                    "tabs are not allowed in indentation; use spaces",
                    Span {
                        line: line_no,
                        column: i + 1,
                    },
                ));
            }
            indent_units += 1;
            i += 1;
        }

        if indent_units > *indent_stack.last().unwrap() {
            indent_stack.push(indent_units);
            tokens.push(Token {
                kind: TokenKind::Indent,
                span: Span {
                    line: line_no,
                    column: 1,
                },
            });
        } else {
            while indent_units < *indent_stack.last().unwrap() {
                indent_stack.pop();
                tokens.push(Token {
                    kind: TokenKind::Dedent,
                    span: Span {
                        line: line_no,
                        column: 1,
                    },
                });
                if indent_stack.is_empty() {
                    return Err(NovaError::lexical(
                        "invalid indentation",
                        Span {
                            line: line_no,
                            column: 1,
                        },
                    ));
                }
            }
            if indent_units != *indent_stack.last().unwrap() {
                return Err(NovaError::lexical(
                    "inconsistent indentation",
                    Span {
                        line: line_no,
                        column: 1,
                    },
                ));
            }
        }

        let mut col = i + 1;
        while i < chars.len() {
            let ch = chars[i];
            if ch.is_whitespace() {
                i += 1;
                col += 1;
                continue;
            }

            let span = Span {
                line: line_no,
                column: col,
            };

            match ch {
                '#' => break,
                quote if quote == '"' || quote == '\'' => {
                    i += 1;
                    col += 1;
                    let mut value = String::new();
                    let mut closed = false;
                    while i < chars.len() {
                        let c = chars[i];
                        if c == quote {
                            closed = true;
                            i += 1;
                            col += 1;
                            break;
                        }
                        if c == '\\' {
                            if i + 1 >= chars.len() {
                                return Err(NovaError::lexical(
                                    "unterminated escape sequence",
                                    Span {
                                        line: line_no,
                                        column: col,
                                    },
                                ));
                            }
                            let next = chars[i + 1];
                            match next {
                                '"' => value.push('"'),
                                '\'' => value.push('\''),
                                '\\' => value.push('\\'),
                                'n' => value.push('\n'),
                                't' => value.push('\t'),
                                _ => {
                                    return Err(NovaError::lexical(
                                        format!("unknown escape sequence '\\{}'", next),
                                        Span {
                                            line: line_no,
                                            column: col,
                                        },
                                    ))
                                }
                            }
                            i += 2;
                            col += 2;
                        } else {
                            value.push(c);
                            i += 1;
                            col += 1;
                        }
                    }
                    if !closed {
                        return Err(NovaError::lexical(
                            "unterminated string literal",
                            Span {
                                line: line_no,
                                column: span.column,
                            },
                        ));
                    }
                    tokens.push(Token {
                        kind: TokenKind::String(value),
                        span,
                    });
                }
                '+' => {
                    tokens.push(Token {
                        kind: TokenKind::Plus,
                        span,
                    });
                    i += 1;
                    col += 1;
                }
                '-' => {
                    tokens.push(Token {
                        kind: TokenKind::Minus,
                        span,
                    });
                    i += 1;
                    col += 1;
                }
                '*' => {
                    tokens.push(Token {
                        kind: TokenKind::Star,
                        span,
                    });
                    i += 1;
                    col += 1;
                }
                '/' => {
                    tokens.push(Token {
                        kind: TokenKind::Slash,
                        span,
                    });
                    i += 1;
                    col += 1;
                }
                ':' => {
                    tokens.push(Token {
                        kind: TokenKind::Colon,
                        span,
                    });
                    i += 1;
                    col += 1;
                }
                '=' => {
                    tokens.push(Token {
                        kind: TokenKind::Equal,
                        span,
                    });
                    i += 1;
                    col += 1;
                }
                '(' => {
                    tokens.push(Token {
                        kind: TokenKind::LParen,
                        span,
                    });
                    i += 1;
                    col += 1;
                }
                ')' => {
                    tokens.push(Token {
                        kind: TokenKind::RParen,
                        span,
                    });
                    i += 1;
                    col += 1;
                }
                _ => {
                    let start = i;
                    while i < chars.len()
                        && !chars[i].is_whitespace()
                        && !"+-*/:=()\"'#".contains(chars[i])
                    {
                        i += 1;
                    }
                    let run: String = chars[start..i].iter().collect();
                    if run.is_empty() {
                        i += 1;
                        col += 1;
                        continue;
                    }
                    let kind = match run.as_str() {
                        "print" => TokenKind::Keyword(Keyword::Print),
                        "create" => TokenKind::Keyword(Keyword::Create),
                        "variable" => TokenKind::Keyword(Keyword::Variable),
                        "if" => TokenKind::Keyword(Keyword::If),
                        "ask" => TokenKind::Keyword(Keyword::Ask),
                        "and" => TokenKind::Keyword(Keyword::And),
                        "store" => TokenKind::Keyword(Keyword::Store),
                        "as" => TokenKind::Keyword(Keyword::As),
                        "is" => TokenKind::Keyword(Keyword::Is),
                        "greater" => TokenKind::Keyword(Keyword::Greater),
                        "than" => TokenKind::Keyword(Keyword::Than),
                        "less" => TokenKind::Keyword(Keyword::Less),
                        "equal" => TokenKind::Keyword(Keyword::Equal),
                        "not" => TokenKind::Keyword(Keyword::Not),
                        "or" => TokenKind::Keyword(Keyword::Or),
                        "to" => TokenKind::Keyword(Keyword::To),
                        _ => {
                            if let Ok(number) = run.parse::<f64>() {
                                // Keep number tokens for numeric runs only.
                                if run.chars().all(|c| c.is_ascii_digit() || c == '.') {
                                    TokenKind::Number(number)
                                } else {
                                    TokenKind::Ident(run.clone())
                                }
                            } else {
                                TokenKind::Ident(run.clone())
                            }
                        }
                    };
                    tokens.push(Token { kind, span });
                    col += i - start;
                    continue;
                }
            }
        }

        tokens.push(Token {
            kind: TokenKind::Newline,
            span: Span {
                line: line_no,
                column: chars.len() + 1,
            },
        });
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token {
            kind: TokenKind::Dedent,
            span: Span {
                line: lines.len().max(1),
                column: 1,
            },
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span {
            line: lines.len().max(1),
            column: 1,
        },
    });

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_keywords_and_number() {
        let tokens = tokenize("print 25").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Print));
        assert_eq!(tokens[1].kind, TokenKind::Number(25.0));
        assert_eq!(tokens[2].kind, TokenKind::Newline);
    }

    #[test]
    fn tokenizes_variable_declaration() {
        let tokens = tokenize("create variable name = \"Sharif\"").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Create));
        assert_eq!(tokens[1].kind, TokenKind::Keyword(Keyword::Variable));
        assert_eq!(tokens[2].kind, TokenKind::Ident("name".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Equal);
        assert_eq!(tokens[4].kind, TokenKind::String("Sharif".to_string()));
    }

    #[test]
    fn tokenizes_indentation() {
        let source = "if age is greater than 18:\n    print age\n";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|token| token.kind == TokenKind::Indent));
        assert!(tokens.iter().any(|token| token.kind == TokenKind::Dedent));
    }

    #[test]
    fn reports_unterminated_string() {
        let result = tokenize("print \"unterminated");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    #[test]
    fn rejects_tabs_in_indentation() {
        let result = tokenize("if 1 is equal to 1:\n\tprint \"x\"\n");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_escape() {
        let result = tokenize("print \"bad \\q\"");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unclosed_string() {
        let result = tokenize("print \"unclosed");
        assert!(result.is_err());
    }
}
