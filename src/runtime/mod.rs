mod value;
pub use value::*;

use crate::errors::NovaError;
use std::collections::HashMap;

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn declare(&mut self, name: String, value: Value) -> Result<(), NovaError> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name) {
            return Err(NovaError::runtime(format!(
                "variable '{}' is already declared",
                name
            )));
        }
        scope.insert(name, value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Value, NovaError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(NovaError::runtime(format!("undefined variable '{}'", name)))
    }
}
