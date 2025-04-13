use crate::environment::{Environment, ParserEnvironment};
use crate::error::{InterpreterError, ParserError};
use crate::r#type::Type;
use crate::token::Token;
use crate::type_checker::TypeCheck;

use super::InstructionResult;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub r#type: Type,

    pub declaration_token: Token,
    pub identifier_token: Token,
    pub last_assignment_token: Token,
    pub type_token: Token,

    pub r#const: bool,
    pub read: bool,
    pub assigned: bool,
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}: {}",
            if self.r#const { "const " } else { "" },
            self.name,
            self.r#type
        )
    }
}

impl std::hash::Hash for Variable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub trait SnakeCase {
    fn is_snake_case(&self) -> bool;
    fn is_upper_snake_case(&self) -> bool;
    fn to_snake_case(&self) -> String;
    fn to_upper_snake_case(&self) -> String;
}

impl SnakeCase for std::string::String {
    fn is_snake_case(&self) -> bool {
        self.chars().all(|c| c.is_lowercase() || c == '_')
    }

    fn is_upper_snake_case(&self) -> bool {
        self.chars().all(|c| c.is_uppercase() || c == '_')
    }

    fn to_snake_case(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.chars().enumerate() {
            if i > 0
                && c.is_uppercase()
                && (i < self.len() - 1 && self.chars().nth(i + 1).unwrap().is_lowercase())
            {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }

    fn to_upper_snake_case(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.chars().enumerate() {
            if i > 0
                && c.is_uppercase()
                && (i < self.len() - 1 && self.chars().nth(i + 1).unwrap().is_lowercase())
            {
                result.push('_');
            }
            result.push(c.to_uppercase().next().unwrap());
        }
        result
    }
}

impl TypeCheck for Variable {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        _token: &Token,
    ) -> Result<Type, ParserError> {
        Ok(match environment.get(&self.name) {
            Some(variable) => variable.clone(),
            None => self.clone(),
        }
        .r#type)
    }
}

impl Variable {
    pub fn interpret(
        &self,
        environment: &mut Environment,
    ) -> Result<InstructionResult, InterpreterError> {
        Ok(environment.get(&self.name).unwrap().clone())
    }
}
