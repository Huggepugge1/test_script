use crate::interpreter::InstructionResult;
use crate::r#type::Type;
use crate::variable::Variable;

use std::collections::HashMap;

pub struct ParseEnvironment {
    pub variables: Vec<HashMap<String, Type>>,
}

impl ParseEnvironment {
    pub fn new() -> ParseEnvironment {
        ParseEnvironment {
            variables: vec![HashMap::new()],
        }
    }

    pub fn add_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    pub fn remove_scope(&mut self) {
        self.variables.pop();
    }

    pub fn insert(&mut self, var: Variable) {
        self.variables
            .last_mut()
            .unwrap()
            .insert(var.name, var.r#type);
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        for scope in self.variables.iter().rev() {
            if let Some(r#type) = scope.get(name) {
                return Some(r#type);
            }
        }

        None
    }
}

pub struct Environment {
    pub variables: Vec<HashMap<String, InstructionResult>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            variables: vec![HashMap::new()],
        }
    }

    pub fn add_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    pub fn remove_scope(&mut self) {
        self.variables.pop();
    }

    pub fn insert(&mut self, name: String, value: InstructionResult) {
        self.variables.last_mut().unwrap().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&InstructionResult> {
        for scope in self.variables.iter().rev() {
            if let Some(r#type) = scope.get(name) {
                return Some(r#type);
            }
        }

        None
    }
}
