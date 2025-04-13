use crate::cli::Args;
use crate::error::{ParseWarning, ParseWarningType};
use crate::instruction::function::Function;
use crate::instruction::variable::Variable;
use crate::instruction::InstructionResult;

use indexmap::IndexMap;

#[derive(Debug)]
pub struct ParseEnvironment {
    pub variables: Vec<IndexMap<String, Variable>>,
    pub functions: IndexMap<String, Function>,
    pub args: Args,
}

impl ParseEnvironment {
    pub fn new(args: Args) -> ParseEnvironment {
        ParseEnvironment {
            variables: vec![IndexMap::new()],
            functions: IndexMap::new(),
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
        for scope in self.variables.iter_mut().rev() {
            if scope.contains_key(&variable.name) {
                scope.insert(variable.name.clone(), variable);
                return;
            }
        }
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

    pub fn add_function(&mut self, function: Function) {
        self.functions.insert(function.name.clone(), function);
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }
}

pub struct Environment {
    pub frames: Vec<Frame>,
    pub global_constants: IndexMap<String, InstructionResult>,
    pub functions: IndexMap<String, Function>,
}

impl Environment {
    pub fn add_frame(&mut self) {
        self.frames.push(Frame {
            variables: vec![IndexMap::new()],
        });
    }

    pub fn remove_frame(&mut self) {
        self.frames.pop();
    }

    pub fn add_scope(&mut self) {
        let len = self.frames.len();
        self.frames[len - 1].variables.push(IndexMap::new());
    }

    pub fn remove_scope(&mut self) {
        let len = self.frames.len();
        self.frames[len - 1].variables.pop();
    }

    pub fn insert(&mut self, name: String, value: InstructionResult) {
        let len = self.frames.len();
        if len == 0 {
            self.global_constants.insert(name, value);
            return;
        }
        for scope in self.frames[len - 1].variables.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return;
            }
        }
        self.frames
            .last_mut()
            .unwrap()
            .variables
            .last_mut()
            .unwrap()
            .insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&InstructionResult> {
        let len = self.frames.len();
        if len == 0 {
            return self.global_constants.get(name);
        }
        for scope in self.frames[len - 1].variables.iter().rev() {
            if let Some(r#type) = scope.get(name) {
                return Some(r#type);
            }
        }

        self.global_constants.get(name)
    }

    pub fn add_function(&mut self, function: Function) {
        self.functions.insert(function.name.clone(), function);
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            frames: vec![],
            global_constants: IndexMap::new(),
            functions: IndexMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub variables: Vec<IndexMap<String, InstructionResult>>,
}
