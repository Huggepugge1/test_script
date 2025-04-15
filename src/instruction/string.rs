use crate::{
    environment::ParserEnvironment, error::ParserMessage, r#type::Type, token::Token,
    type_checker::TypeCheck,
};

use super::InstructionResult;

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    pub value: String,
}

impl std::fmt::Display for StringLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TypeCheck for StringLiteral {
    fn type_check(
        &self,
        _environment: &mut ParserEnvironment,
        _token: &Token,
        _messages: &mut Vec<ParserMessage>,
    ) -> Type {
        Type::String
    }
}

impl StringLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::String(self.value.clone())
    }
}
