use crate::{
    environment::{self, ParserEnvironment},
    error::{InterpreterError, ParserMessage},
    interpreter::Interpret,
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

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
pub struct Unary {
    pub operator: UnaryOperator,
    pub body: Box<Instruction>,
}

impl std::fmt::Display for Unary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.operator, self.body)
    }
}

impl TypeCheck for Unary {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        let body_type = self.body.type_check(environment, token, messages);
        match self.operator {
            UnaryOperator::Not => {
                if body_type != Type::Bool {
                    messages.push(ParserMessage::error_mismatched_type(
                        vec![Type::Bool],
                        body_type.clone(),
                        token.clone(),
                    ));
                }
            }
            UnaryOperator::Negation => {
                if !matches!(body_type, Type::Int | Type::Float) {
                    messages.push(ParserMessage::error_mismatched_type(
                        vec![Type::Int, Type::Float],
                        body_type.clone(),
                        token.clone(),
                    ));
                }
            }
        }
        body_type
    }
}

impl Interpret for Unary {
    fn interpret(
        &self,
        environment: &mut environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let result = self.body.interpret(environment, process)?;
        match self.operator {
            UnaryOperator::Not => self.not(result),
            UnaryOperator::Negation => self.negation(result),
        }
    }
}

impl Unary {
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
