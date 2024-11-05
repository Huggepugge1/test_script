use crate::error::{ParseError, ParseErrorType, ParseWarning, ParseWarningType};
use crate::token::{TokenCollection, TokenType};
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
    Literal(String),
    BuiltIn(BuiltIn),
    Test(Vec<Instruction>, String, PathBuf),
    For(Vec<Instruction>, Vec<String>),
    None,
}

fn parse_literal(tokens: &mut TokenCollection, max_size: u32) -> Result<Instruction, ParseError> {
    let token = tokens.current().unwrap();
    if token.r#type != TokenType::Literal {
        tokens.advance_to_next_instruction();
        Err(ParseError::new(
            ParseErrorType::MismatchedType(TokenType::Literal, token.clone().r#type),
            token.clone(),
            format!("Token {:?} is not a literal string", token.value),
        ))
    } else {
        Ok(Instruction::new(
            InstructionType::Literal(token.value),
            token.line,
            token.column,
        ))
    }
}

fn expect_token(tokens: &mut TokenCollection, expected: TokenType) -> Result<(), ParseError> {
    if let Some(token) = tokens.next() {
        if token.r#type != expected {
            tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedType(expected, token.clone().r#type),
                token.clone(),
                format!("Token {:?} is not of the right type", token.value),
            ))
        } else {
            Ok(())
        }
    } else {
        Err(ParseError::new(
            ParseErrorType::UnexpectedEndOfFile,
            tokens.current().unwrap(),
            "The file ended in the middle of an instruction",
        ))
    }
}

fn parse_builtin(tokens: &mut TokenCollection, max_size: u32) -> Result<Instruction, ParseError> {
    let token = tokens.current().unwrap();
    expect_token(tokens, TokenType::OpenParen)?;
    Ok(Instruction::NONE)
}

fn parse_identifier(
    tokens: &mut TokenCollection,
    max_size: u32,
) -> Result<Instruction, ParseError> {
    let token = tokens.current();
    Ok(Instruction::NONE)
}

fn parse_keyword(tokens: &mut TokenCollection, _max_size: u32) -> Result<Instruction, ParseError> {
    Err(ParseError::new(
        ParseErrorType::NotImplemented,
        tokens.current().unwrap().clone(),
        "See discord for more information about comming features",
    ))
}

fn end_statement(tokens: &mut TokenCollection) -> Result<(), ParseError> {
    if let Some(token) = tokens.current() {
        if token.r#type == TokenType::SemiColon {
            tokens.next();
            Ok(())
        } else {
            Err(ParseError::new(
                ParseErrorType::MissingSemicolon,
                token.clone(),
                "Did you forget a semicolon?",
            ))
        }
    } else {
        Err(ParseError::new(
            ParseErrorType::UnexpectedEndOfFile,
            tokens.current().unwrap(),
            "The file ended in the middle of an instruction",
        ))
    }
}

fn parse_test(tokens: &mut TokenCollection, max_size: u32) -> Result<Instruction, ParseError> {
    let token = tokens.current().unwrap();
    expect_token(tokens, TokenType::OpenParen)?;
    let path = parse_literal(tokens, max_size)?;
    let path = match path.r#type {
        InstructionType::Literal(path) => path,
        _ => unreachable!(),
    };
    expect_token(tokens, TokenType::CloseParen)?;
    expect_token(tokens, TokenType::OpenBlock)?;

    let mut block = Vec::new();
    while let Some(token) = tokens.current() {
        let instruction = match token.r#type {
            TokenType::Literal => {
                ParseWarning::new(
                    ParseWarningType::UnusedLiteral,
                    token,
                    "See discord for more information about comming features",
                )
                .print();
                parse_literal(tokens, max_size)
            }
            TokenType::Keyword => parse_keyword(tokens, max_size),
            TokenType::BuiltIn => parse_builtin(tokens, max_size),
            TokenType::Identifier => parse_identifier(tokens, max_size),
            TokenType::CloseBlock => {
                break;
            }
            _ => {
                tokens.advance_to_next_instruction();
                Err(ParseError::new(
                    ParseErrorType::NotImplemented,
                    token,
                    "See discord for more information about comming features",
                ))
            }
        };

        match instruction {
            Ok(instruction) => block.push(instruction),
            Err(error) => error.print(),
        }
        end_statement(tokens)?;
    }

    Ok(Instruction::new(
        InstructionType::Test(block, token.value, path.into()),
        token.line,
        token.column,
    ))
}

pub fn parse(tokens: &mut TokenCollection, max_size: u32) -> Result<Vec<Instruction>, ()> {
    let mut program = Vec::new();
    let mut failed = false;

    while let Some(token) = tokens.current() {
        let instruction = match token.clone().r#type {
            TokenType::Identifier => parse_test(tokens, max_size),
            r#type => {
                tokens.advance_to_next_instruction();
                Err(ParseError::new(
                    ParseErrorType::MismatchedType(TokenType::Identifier, r#type),
                    token,
                    "Only test names are allowed in the main scope",
                ))
            }
        };

        match instruction {
            Ok(instruction) => program.push(instruction),
            Err(error) => {
                error.print();
                failed = true;
            }
        }
    }

    if failed {
        Err(())
    } else {
        Ok(program)
    }
}
