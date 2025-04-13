use crate::{
    environment::{self, ParserEnvironment},
    error::{InterpreterError, ParserError, ParserErrorType},
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
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub body: Box<Instruction>,
}

impl std::fmt::Display for UnaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.operator, self.body)
    }
}

impl TypeCheck for UnaryOperation {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
    ) -> Result<Type, ParserError> {
        let body_type = self.body.type_check(environment)?;
        match self.operator {
            UnaryOperator::Not => {
                if body_type != Type::Bool {
                    let expected = vec![Type::Bool];
                    return Err(ParserError::new(
                        ParserErrorType::MismatchedType {
                            expected,
                            actual: body_type,
                        },
                        token.clone(),
                    ));
                }
            }
            UnaryOperator::Negation => {
                if !matches!(body_type, Type::Int | Type::Float) {
                    return Err(ParserError::new(
                        ParserErrorType::MismatchedType {
                            expected: vec![Type::Int, Type::Float],
                            actual: body_type,
                        },
                        token.clone(),
                    ));
                }
            }
        }
        Ok(body_type)
    }
}

impl UnaryOperation {
    pub fn interpret(
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
