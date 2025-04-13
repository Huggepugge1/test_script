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

impl StringLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::String(self.value.clone())
    }
}
