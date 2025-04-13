use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserError},
    process::Process,
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Paren {
    pub expression: Box<Instruction>,
}

impl std::fmt::Display for Paren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.expression)
    }
}

impl TypeCheck for Paren {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        _token: &Token,
    ) -> Result<Type, ParserError> {
        self.expression.type_check(environment)
    }
}

impl Paren {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        self.expression.interpret(environment, process)
    }
}
