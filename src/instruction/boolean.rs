use crate::{
    environment::ParserEnvironment, error::ParserError, r#type::Type, type_checker::TypeCheck,
};

use super::InstructionResult;

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanLiteral {
    pub value: bool,
}

impl std::fmt::Display for BooleanLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TypeCheck for BooleanLiteral {
    fn type_check(
        &self,
        _environment: &mut ParserEnvironment,
        _token: &crate::token::Token,
    ) -> Result<Type, ParserError> {
        Ok(Type::Bool)
    }
}

impl BooleanLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::Bool(self.value)
    }
}
