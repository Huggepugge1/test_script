use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserError, ParserErrorType},
    instruction::InstructionType,
    process::Process,
    r#type::Type,
    token::Token,
};

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
    pub fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
    ) -> Result<Type, ParserError> {
        let condition_type = self.condition.type_check(environment)?;
        if condition_type != Type::Bool {
            return Err(ParserError::new(
                ParserErrorType::MismatchedType {
                    expected: vec![Type::Bool],
                    actual: condition_type,
                },
                token.clone(),
            ));
        }
        let if_type = self.r#if.type_check(environment)?;
        let else_type = self.r#else.type_check(environment)?;
        if if_type != else_type {
            return Err(ParserError::new(
                ParserErrorType::MismatchedType {
                    expected: vec![if_type],
                    actual: condition_type,
                },
                token.clone(),
            ));
        }
        Ok(if_type)
    }

    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let condition_result = self.condition.interpret(environment, process)?;
        match condition_result {
            InstructionResult::Bool(true) => self.r#if.interpret(environment, process),
            InstructionResult::Bool(false) => self.r#else.interpret(environment, process),
            _ => unreachable!(),
        }
    }
}
