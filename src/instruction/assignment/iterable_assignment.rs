use crate::instruction::{variable::Variable, Instruction};

#[derive(Debug, Clone, PartialEq)]
pub struct IterableAssignment {
    pub variable: Variable,
    pub body: Box<Instruction>,
}

impl std::fmt::Display for IterableAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in {}", self.variable, self.body)
    }
}
