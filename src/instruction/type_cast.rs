use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserError, ParserErrorType, ParserMessage},
    interpreter::Interpret,
    process::Process,
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

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

impl TypeCheck for TypeCast {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        let from_type = self.from.type_check(environment, token, messages);
        if !from_type.can_cast_to(&self.to) {
            messages.push(ParserMessage::Error(ParserError::new(
                ParserErrorType::TypeCast {
                    from: from_type,
                    to: self.to.clone(),
                },
                token.clone(),
            )));
            Type::None
        } else {
            self.to.clone()
        }
    }
}

impl Interpret for TypeCast {
    fn interpret(
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
}

impl TypeCast {
    fn cast_to_string(
        &self,
        environtment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        Ok(InstructionResult::String(
            self.from.interpret(environtment, process)?.to_string(),
        ))
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
