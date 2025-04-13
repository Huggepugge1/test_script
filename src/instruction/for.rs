use crate::{environment::Environment, error::InterpreterError, process::Process};

use super::{assignment::iterable_assignment::IterableAssignment, Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub assignment: IterableAssignment,
    pub body: Box<Instruction>,
}

impl std::fmt::Display for For {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "for {} {}", self.assignment, self.body)
    }
}

impl For {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        environment.add_scope();
        let mut result = InstructionResult::None;

        match self.assignment.body.interpret(environment, process)? {
            InstructionResult::Regex(regex) => {
                for item in regex {
                    environment.insert(self.assignment.variable.name.clone(), item);
                    match self.body.interpret(environment, process) {
                        Ok(value) => result = value,
                        Err(err) => {
                            environment.remove_scope();
                            return Err(err);
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        environment.remove_scope();
        Ok(result)
    }
}
