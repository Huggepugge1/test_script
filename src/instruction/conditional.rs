use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserMessage},
    instruction::InstructionType,
    interpreter::Interpret,
    process::Process,
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
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

impl TypeCheck for Conditional {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        let condition_type = self.condition.type_check(environment, token, messages);
        if condition_type != Type::Bool {
            messages.push(ParserMessage::error_mismatched_type(
                vec![Type::Bool],
                condition_type.clone(),
                token.clone(),
            ));
        }
        let if_type = self.r#if.type_check(environment, token, messages);
        let else_type = self.r#else.type_check(environment, token, messages);
        if if_type != else_type {
            messages.push(ParserMessage::error_mismatched_type(
                vec![if_type.clone()],
                else_type.clone(),
                token.clone(),
            ));
        }
        if_type
    }
}

impl Interpret for Conditional {
    fn interpret(
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
