use crate::cli::Args;
use crate::error::{ParseWarning, ParseWarningType};
use crate::interpreter::InstructionResult;
use crate::variable::Variable;

use indexmap::IndexMap;

pub struct ParseEnvironment {
    pub variables: Vec<IndexMap<String, Variable>>,
    pub args: Args,
}

impl ParseEnvironment {
    pub fn new(args: Args) -> ParseEnvironment {
        ParseEnvironment {
            variables: vec![IndexMap::new()],
            args,
        }
    }

    pub fn add_scope(&mut self) {
        self.variables.push(IndexMap::new());
    }

    pub fn remove_scope(&mut self) {
        self.check_unused();
        self.check_assigned();
        self.variables.pop();
    }

    pub fn insert(&mut self, variable: Variable) {
        self.variables
            .last_mut()
            .unwrap()
            .insert(variable.name.clone(), variable);
    }

    pub fn get(&mut self, name: &str) -> Option<&mut Variable> {
        for scope in &mut self.variables.iter_mut().rev() {
            if let Some(r#type) = scope.get_mut(name) {
                return Some(r#type);
            }
        }

        None
    }

    fn check_unused(&self) {
        for variable in &self.variables[self.variables.len() - 1] {
            if !variable.1.read && variable.1.name.chars().nth(0).unwrap() != '_' {
                if variable.1.declaration_token != variable.1.last_assignment_token {
                    ParseWarning::new(
                        ParseWarningType::VariableNotRead,
                        variable.1.last_assignment_token.clone(),
                    )
                    .print(self.args.disable_warnings);
                } else {
                    ParseWarning::new(
                        ParseWarningType::UnusedVariable,
                        variable.1.identifier_token.clone(),
                    )
                    .print(self.args.disable_warnings);
                }
            }
        }
    }

    fn check_assigned(&self) {
        for variable in &self.variables[self.variables.len() - 1] {
            if !variable.1.r#const
                && !variable.1.assigned
                && variable.1.name.chars().nth(0).unwrap() != '_'
            {
                ParseWarning::new(
                    ParseWarningType::VariableNeverReAssigned,
                    variable.1.declaration_token.clone(),
                )
                .print(self.args.disable_warnings);
            }
        }
    }
}

pub struct Environment {
    pub variables: Vec<IndexMap<String, InstructionResult>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            variables: vec![IndexMap::new()],
        }
    }

    pub fn add_scope(&mut self) {
        self.variables.push(IndexMap::new());
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
