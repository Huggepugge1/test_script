use crate::error::{ParseError, ParseErrorType};
use crate::token::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltIn {
    Input(Box<Instruction>),
    Output(Box<Instruction>),
    Print(Box<Instruction>),
    Println(Box<Instruction>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub r#type: InstructionType,
    pub token: Token,
}

impl Instruction {
    pub const NONE: Instruction = Instruction {
        r#type: InstructionType::None,
        token: Token {
            r#type: TokenType::None,
            value: String::new(),
            line: 0,
            column: 0,
        },
    };

    pub fn new(r#type: InstructionType, token: Token) -> Self {
        Self { r#type, token }
    }

    pub fn get_variable_name(&self) -> Result<String, ParseError> {
        match &self.r#type {
            InstructionType::IterableAssignment(var, _) => Ok(var.to_string()),
            InstructionType::Variable(var) => Ok(var.to_string()),
            _ => Err(ParseError::new(
                ParseErrorType::VariableNotDefined,
                self.token.clone(),
                "Expected a variable",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    StringLiteral(String),
    RegexLiteral(Vec<String>),
    BuiltIn(BuiltIn),
    Block(Vec<Instruction>),
    Test(Box<Instruction>, String, String),
    For(Box<Instruction>, Box<Instruction>),

    IterableAssignment(String, Box<Instruction>),
    Variable(String),

    Addition(Box<Instruction>, Box<Instruction>),

    None,
}
