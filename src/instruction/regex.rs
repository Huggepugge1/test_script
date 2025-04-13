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

impl RegexLiteral {
    pub fn interpret(&self) -> InstructionResult {
        InstructionResult::Regex(self.value.clone())
    }
}
