pub mod iterable_assignment;

use crate::{
    environment::Environment,
    error::InterpreterError,
    instruction::{variable::Variable, Instruction, InstructionResult},
    process::Process,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub variable: Variable,
    pub body: Box<Instruction>,
    pub token: Token,
    pub declaration: bool,
}

impl std::fmt::Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.variable, self.body)
    }
}

impl Assignment {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let result = self.body.interpret(environment, process)?;
        environment.insert(self.variable.name.clone(), result.clone());
        Ok(result)
    }
}
