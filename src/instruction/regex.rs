use crate::{
    environment::ParserEnvironment, error::ParserMessage, r#type::Type, type_checker::TypeCheck,
};

use super::InstructionResult;

#[derive(Debug, Clone, PartialEq)]
pub struct RegexLiteral {
    pub value: Vec<InstructionResult>,
}

impl std::fmt::Display for RegexLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl TypeCheck for RegexLiteral {
    fn type_check(
        &self,
        _environment: &mut ParserEnvironment,
        _token: &crate::token::Token,
        _messages: &mut Vec<ParserMessage>,
    ) -> Type {
        Type::Regex
    }
}

impl RegexLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::Regex(self.value.clone())
    }
}
