use crate::error::InterpreterError;

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negation,
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOperator::Not => "!",
                UnaryOperator::Negation => "-",
            }
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub instruction: Box<Instruction>,
}

impl std::fmt::Display for UnaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.operator, self.instruction)
    }
}

impl UnaryOperation {
    pub fn interpret(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let result = self.instruction.interpret(environment, process)?;
        match self.operator {
            UnaryOperator::Not => self.not(result),
            UnaryOperator::Negation => self.negation(result),
        }
    }

    fn not(&self, result: InstructionResult) -> Result<InstructionResult, InterpreterError> {
        match result {
            InstructionResult::Bool(value) => Ok(InstructionResult::Bool(!value)),
            _ => unreachable!(),
        }
    }

    fn negation(&self, result: InstructionResult) -> Result<InstructionResult, InterpreterError> {
        match result {
            InstructionResult::Int(value) => Ok(InstructionResult::Int(-value)),
            InstructionResult::Float(value) => Ok(InstructionResult::Float(-value)),
            _ => unreachable!(),
        }
    }
}
