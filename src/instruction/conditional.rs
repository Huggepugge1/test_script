use crate::{error::InterpreterError, instruction::InstructionType};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    pub condition: Box<Instruction>,
    pub r#if: Box<Instruction>,
    pub r#else: Box<Instruction>,
}

impl std::fmt::Display for Conditional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} {}", self.condition, self.r#if)?;
        match self.r#else.r#type {
            InstructionType::None => Ok(()),
            _ => write!(f, " else {{\n{}\n}}", self.r#else),
        }
    }
}

impl Conditional {
    pub fn interpret(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let condition_result = self.condition.interpret(environment, process)?;
        match condition_result {
            InstructionResult::Bool(true) => self.r#if.interpret(environment, process),
            InstructionResult::Bool(false) => self.r#else.interpret(environment, process),
            _ => unreachable!(),
        }
    }
}
