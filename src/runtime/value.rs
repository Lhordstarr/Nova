use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(number) => {
                if number.is_finite() && number.fract() == 0.0 {
                    write!(f, "{:.0}", number)
                } else {
                    write!(f, "{}", number)
                }
            }
            Value::String(text) => write!(f, "{}", text),
            Value::Boolean(value) => write!(f, "{}", value),
        }
    }
}
