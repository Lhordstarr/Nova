use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    Semantic,
    Runtime,
    Cli,
    Io,
}

#[derive(Debug, Clone)]
pub struct NovaError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
}

impl NovaError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn lexical(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Lexical, message).with_span(span)
    }

    pub fn syntax(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Syntax, message).with_span(span)
    }

    pub fn semantic(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::new(ErrorKind::Semantic, message).with_span_if(span)
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Runtime, message)
    }
}

impl NovaError {
    fn with_span_if(mut self, span: Option<Span>) -> Self {
        if let Some(span) = span {
            self.span = Some(span);
        }
        self
    }
}

impl fmt::Display for NovaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(f, "{}: {}", span, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for NovaError {}

impl From<std::io::Error> for NovaError {
    fn from(value: std::io::Error) -> Self {
        Self::new(ErrorKind::Io, value.to_string())
    }
}
