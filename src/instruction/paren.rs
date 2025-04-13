use crate::{environment::Environment, error::InterpreterError, process::Process};

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

impl Paren {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        self.expression.interpret(environment, process)
    }
}
