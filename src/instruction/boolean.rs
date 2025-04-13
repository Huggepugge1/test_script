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

impl BooleanLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::Bool(self.value)
    }
}
