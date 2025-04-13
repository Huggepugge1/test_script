use crate::{environment::Environment, error::InterpreterError, process::Process, r#type::Type};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeCast {
    pub from: Box<Instruction>,
    pub to: Type,
}

impl std::fmt::Display for TypeCast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} as {}", self.from, self.to)
    }
}

impl TypeCast {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        match self.to {
            Type::String => self.cast_to_string(environment, process),
            Type::Int => self.cast_to_int(environment, process),
            Type::Float => self.cast_to_float(environment, process),
            Type::Bool => self.cast_to_bool(environment, process),
            _ => unreachable!(),
        }
    }

    fn cast_to_string(
        &self,
        environtment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let from = self.from.interpret(environtment, process)?;
        Ok(match from {
            InstructionResult::Int(i) => InstructionResult::String(i.to_string()),
            InstructionResult::Float(f) => InstructionResult::String(f.to_string()),
            InstructionResult::Bool(b) => InstructionResult::String(b.to_string()),
            _ => unreachable!(),
        })
    }

    fn cast_to_int(
        &self,
        environtment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let from = self.from.interpret(environtment, process)?;
        Ok(match from {
            InstructionResult::String(ref s) => InstructionResult::Int(match s.parse::<i64>() {
                Ok(i) => i,
                Err(_) => {
                    return Err(InterpreterError::TypeCast {
                        result: from,
                        from: Type::String,
                        to: Type::Int,
                    })
                }
            }),
            InstructionResult::Float(f) => InstructionResult::Int(f as i64),
            InstructionResult::Bool(b) => InstructionResult::Int(b as i64),
            _ => unreachable!(),
        })
    }

    fn cast_to_float(
        &self,
        environtment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let from = self.from.interpret(environtment, process)?;
        Ok(match from {
            InstructionResult::String(ref s) => InstructionResult::Float(match s.parse::<f64>() {
                Ok(f) => f,
                Err(_) => {
                    return Err(InterpreterError::TypeCast {
                        result: from,
                        from: Type::String,
                        to: Type::Float,
                    })
                }
            }),
            InstructionResult::Int(i) => InstructionResult::Float(i as f64),
            InstructionResult::Bool(b) => InstructionResult::Float(b as i64 as f64),
            _ => unreachable!(),
        })
    }

    fn cast_to_bool(
        &self,
        environtment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let from = self.from.interpret(environtment, process)?;
        Ok(match from {
            InstructionResult::String(ref s) => InstructionResult::Bool(match s.as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    return Err(InterpreterError::TypeCast {
                        result: from,
                        from: Type::String,
                        to: Type::Bool,
                    })
                }
            }),
            InstructionResult::Int(i) => InstructionResult::Bool(i != 0),
            InstructionResult::Float(f) => InstructionResult::Bool(f != 0.0),
            _ => unreachable!(),
        })
    }
}
