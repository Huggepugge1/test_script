use crate::cli::Args;
use crate::error::{ParseWarning, ParseWarningType};
use crate::interpreter::InstructionResult;
use crate::variable::Variable;

use std::collections::HashMap;

pub struct ParseEnvironment {
    pub variables: Vec<HashMap<String, Variable>>,
    pub args: Args,
}

impl ParseEnvironment {
    pub fn new(args: Args) -> ParseEnvironment {
        ParseEnvironment {
            variables: vec![HashMap::new()],
            args,
        }
    }

    pub fn add_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    pub fn remove_scope(&mut self) {
        self.check_unused();
        self.variables.pop();
    }

    pub fn insert(&mut self, variable: Variable) {
        self.variables
            .last_mut()
            .unwrap()
            .insert(variable.name.clone(), variable);
    }

    pub fn get(&self, name: &str) -> Option<&Variable> {
        for scope in self.variables.iter().rev() {
            if let Some(variable) = scope.get(name) {
                return Some(variable);
            }
        }

        None
    }

    fn check_unused(&self) {
        for variable in &self.variables[self.variables.len() - 1] {
            if !variable.1.used && variable.1.name.chars().nth(0).unwrap() != '_' {
                ParseWarning::new(ParseWarningType::UnusedVariable, variable.1.token.clone())
                    .print(self.args.disable_warnings);
            }
        }
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
