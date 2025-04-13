use crate::{environment::Environment, error::InterpreterError, process::Process};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Instruction>,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for statement in &self.statements {
            writeln!(f, "    {}", statement)?;
        }
        write!(f, "}}")
    }
}

impl Block {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        environment.add_scope();
        let mut result = InstructionResult::None;
        for statement in &self.statements {
            result = match statement.interpret(environment, process) {
                Ok(value) => value,
                Err(err) => {
                    environment.remove_scope();
                    return Err(err);
                }
            };
        }
        environment.remove_scope();
        Ok(result)
    }
}
