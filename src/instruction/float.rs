use crate::{
    environment::ParserEnvironment, error::ParserError, r#type::Type, type_checker::TypeCheck,
};

use super::InstructionResult;

#[derive(Debug, Clone, PartialEq)]
pub struct FloatLiteral {
    pub value: f64,
}

impl std::fmt::Display for FloatLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TypeCheck for FloatLiteral {
    fn type_check(
        &self,
        _environment: &mut ParserEnvironment,
        _token: &crate::token::Token,
    ) -> Result<Type, ParserError> {
        Ok(Type::Float)
    }
}

impl FloatLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::Float(self.value)
    }
}
