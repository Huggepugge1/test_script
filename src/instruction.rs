use crate::error::{ParseError, ParseErrorType};
use crate::r#type::Type;
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

    pub fn literal(&self) -> bool {
        matches!(
            self.r#type,
            InstructionType::StringLiteral(_) | InstructionType::RegexLiteral(_)
        )
    }

    pub fn get_variable_name(&self) -> Result<String, ParseError> {
        match &self.r#type {
            InstructionType::IterableAssignment(var, _type, _instruction) => Ok(var.to_string()),
            InstructionType::Variable(var) => Ok(var.to_string()),
            _ => Err(ParseError::new(
                ParseErrorType::VariableNotDefined,
                self.token.clone(),
                "Expected a variable",
            )),
        }
    }

    pub fn get_variable_type(&self) -> Result<Type, ParseError> {
        match &self.r#type {
            InstructionType::IterableAssignment(_var, r#type, _instruction) => Ok(r#type.clone()),
            InstructionType::Assignment(_var, r#type, _instruction) => Ok(r#type.clone()),
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

    Assignment(String, Type, Box<Instruction>),
    IterableAssignment(String, Type, Box<Instruction>),
    Variable(String),

    Addition(Box<Instruction>, Box<Instruction>),

    None,
}
