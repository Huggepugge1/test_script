use crate::{
    environment::ParserEnvironment, error::ParserMessage, r#type::Type, type_checker::TypeCheck,
};

use super::InstructionResult;

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerLiteral {
    pub value: i64,
}

impl std::fmt::Display for IntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TypeCheck for IntegerLiteral {
    fn type_check(
        &self,
        _environment: &mut ParserEnvironment,
        _token: &crate::token::Token,
        _messages: &mut Vec<ParserMessage>,
    ) -> Type {
        Type::Int
    }
}

impl IntegerLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::Int(self.value)
    }
}
