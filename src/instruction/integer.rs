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

impl IntegerLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::Int(self.value)
    }
}
