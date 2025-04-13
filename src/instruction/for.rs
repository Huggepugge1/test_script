use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserError},
    process::Process,
    r#type::Type,
    type_checker::TypeCheck,
};

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

impl TypeCheck for For {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &crate::token::Token,
    ) -> Result<Type, ParserError> {
        environment.add_scope();
        self.assignment.type_check(environment, token)?;
        let result = self.body.type_check(environment);
        environment.remove_scope();
        result
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
