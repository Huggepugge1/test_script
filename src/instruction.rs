use crate::error::{ParseError, ParseErrorType};
use crate::r#type::Type;
use crate::token::{Token, TokenType};
use crate::variable::Variable;

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
            InstructionType::IterableAssignment(var, _instruction) => Ok(var.name.clone()),
            InstructionType::Assignment(var, _instruction) => Ok(var.name.clone()),
            InstructionType::Variable(var) => Ok(var.name.clone()),
            _ => Err(ParseError::new(
                ParseErrorType::VariableNotDefined,
                self.token.clone(),
                "Expected a variable",
            )),
        }
    }

    pub fn get_variable_type(&self) -> Result<Type, ParseError> {
        match &self.r#type {
            InstructionType::IterableAssignment(var, _instruction) => Ok(var.r#type),
            InstructionType::Assignment(var, _instruction) => Ok(var.r#type),
            InstructionType::Variable(var) => Ok(var.r#type),
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

    Assignment(Variable, Box<Instruction>),
    IterableAssignment(Variable, Box<Instruction>),
    Variable(Variable),

    Addition {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },

    None,
}
