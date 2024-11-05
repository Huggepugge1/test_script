use crate::lexer::{Token, TokenType};
use crate::regex;
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
    Literal(Vec<String>),
    BuiltIn(BuiltIn),
    Test(Vec<Instruction>, String, PathBuf),
    None,
}

fn parse_literal(
    _tokens: &mut impl Iterator<Item = Token>,
    token: Token,
    max_size: u32,
) -> Instruction {
    if token.value == "" {
        Instruction::new(InstructionType::None, 0, 0)
    } else {
        Instruction::new(
            InstructionType::Literal(regex::parse(&token.value, max_size)),
            token.line,
            token.column,
        )
    }
}

fn parse_identifier(
    tokens: &mut impl Iterator<Item = Token>,
    token: Token,
    max_size: u32,
) -> Instruction {
    if token.value == "input" || token.value == "output" {
        if let Some(token) = tokens.next() {
            if token.r#type != TokenType::OpenParen {
                panic!(
                    "Expected {:?}, {}:{}",
                    token.value, token.line, token.column
                );
            }
        }
        if let Some(next_token) = tokens.next() {
            match next_token.r#type {
                TokenType::Literal => {
                    if let Some(token) = tokens.next() {
                        if token.r#type != TokenType::CloseParen {
                            panic!(
                                "Expected {:?}, {}:{}",
                                token.value, token.line, token.column
                            );
                        }
                    }
                    let literal = Box::new(parse_literal(tokens, next_token, max_size));
                    match token.value.as_str() {
                        "input" => Instruction::new(
                            InstructionType::BuiltIn(BuiltIn::Input(literal)),
                            token.line,
                            token.column,
                        ),
                        "output" => Instruction::new(
                            InstructionType::BuiltIn(BuiltIn::Output(literal)),
                            token.line,
                            token.column,
                        ),
                        _ => panic!(
                            "Unexpected token {:?}, {}:{}",
                            token.value, token.line, token.column
                        ),
                    }
                }
                _ => panic!(
                    "Unexpected token {:?}, {}:{}",
                    next_token.value, next_token.line, next_token.column
                ),
            }
        } else {
            panic!("Unexpected end of input")
        }
    } else {
        panic!(
            "Unexpected identifier {:?}, {}:{}",
            token.value, token.line, token.column
        );
    }
}

fn parse_test(
    tokens: &mut impl Iterator<Item = Token>,
    token: Token,
    max_size: u32,
) -> Instruction {
    let mut current = Instruction::new(InstructionType::None, 0, 0);
    let mut block = Vec::new();
    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::OpenParen {}
    } else {
        panic!("Unexpected end of input")
    }

    let file = if let Some(token) = tokens.next() {
        if token.r#type != TokenType::Literal {
            panic!(
                "Expected filename literal, found {:?}, {}:{}",
                token.value, token.line, token.column
            );
        }

        PathBuf::from("./".to_string() + &token.value)
    } else {
        panic!("Unexpected end of input")
    };

    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::CloseParen {
            panic!(
                "Expected ), found {:?}, {}:{}",
                token, token.line, token.column
            );
        }
    } else {
        panic!("Unexpected end of input");
    }

    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::OpenBlock {
            panic!(
                "Expected {{, found {:?}, {}:{}",
                token, token.line, token.column
            );
        }
    } else {
        panic!("Unexpected end of input");
    }

    while let Some(token) = tokens.next() {
        match token.r#type {
            TokenType::Literal => {
                eprintln!(
                    "Warning: Ignoring literal {:?}, {}:{}",
                    token.value, token.line, token.column
                );
                current = parse_literal(tokens, token, max_size);
            }
            TokenType::Identifier => {
                if current.r#type == InstructionType::None {
                    current = parse_identifier(tokens, token, max_size);
                } else {
                    panic!(
                        "Unexpected token {:?}, {}:{}",
                        token.value, token.line, token.column
                    );
                }
            }
            TokenType::OpenBlock => panic!(
                "Blocks not supported {:?}, {}:{}",
                token.value, token.line, token.column
            ),
            TokenType::CloseBlock => break,
            TokenType::OpenParen => panic!(
                "Parens not supported {:?}, {}:{}",
                token.value, token.line, token.column
            ),
            TokenType::CloseParen => panic!(
                "Parens not supported {:?}, {}:{}",
                token.value, token.line, token.column
            ),
            TokenType::SemiColon => {
                if current.r#type == InstructionType::None {
                    eprintln!(
                        "Warning: unecessary {:?}, {}:{}",
                        token.value, token.line, token.column
                    );
                }
                block.push(current.clone());
                current = Instruction::NONE;
            }
        }
    }
    Instruction::new(
        InstructionType::Test(block, token.value, file),
        token.line,
        token.column,
    )
}

pub fn parse(tokens: Vec<Token>, max_size: u32) -> Vec<Instruction> {
    let mut tokens = tokens.into_iter();
    let mut program = Vec::new();
    while let Some(token) = tokens.next() {
        match token.r#type {
            TokenType::Identifier => {
                program.push(parse_test(&mut tokens, token, max_size));
            }
            other => panic!(
                "Unexpected token {:?}, {}:{}",
                other, token.line, token.column
            ),
        }
    }

    program
}
