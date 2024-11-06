use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltIn {
    Input(Box<Instruction>),
    Output(Box<Instruction>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub r#type: InstructionType,
    pub line: u32,
    pub column: u32,
}

impl Instruction {
    pub const NONE: Instruction = Instruction {
        r#type: InstructionType::None,
        line: 0,
        column: 0,
    };
    pub fn new(r#type: InstructionType, line: u32, column: u32) -> Self {
        Self {
            r#type,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    StringLiteral(String),
    RegexLiteral(Vec<String>),
    BuiltIn(BuiltIn),
    Test(Vec<Instruction>, String, PathBuf),
    For(Vec<Instruction>, Instruction),
    None,
}
