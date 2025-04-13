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

impl From<FloatLiteral> for String {
    fn from(val: FloatLiteral) -> Self {
        val.value.to_string()
    }
}

impl FloatLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::Float(self.value)
    }
}
