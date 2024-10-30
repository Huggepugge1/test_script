use crate::lexer::{Token, TokenType};
use crate::regex;
use std::path::PathBuf;

// #[derive(Debug, PartialEq)]
// pub enum TokenType {
//     Literal,
//     Identifier,
//     OpenBlock,
//     CloseBlock,
//     OpenParen,
//     CloseParen,
//     SemiColon,
// }

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltIn {
    Input(Box<Instruction>),
    Output(Box<Instruction>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Literal(Vec<String>),
    BuiltIn(BuiltIn),
    Test(Vec<Instruction>, String, PathBuf),
    None,
}

fn parse_literal(
    _tokens: &mut impl Iterator<Item = Token>,
    value: String,
    max_size: u32,
) -> Instruction {
    if value == "" {
        Instruction::None
    } else {
        Instruction::Literal(regex::parse(&value, max_size))
    }
}

fn parse_identifier(
    tokens: &mut impl Iterator<Item = Token>,
    value: String,
    max_size: u32,
) -> Instruction {
    if value == "input" || value == "output" {
        if let Some(token) = tokens.next() {
            if token.r#type != TokenType::OpenParen {
                panic!("Expected open paren {:?}", token);
            }
        }
        if let Some(token) = tokens.next() {
            match token.r#type {
                TokenType::Literal => {
                    if let Some(token) = tokens.next() {
                        if token.r#type != TokenType::CloseParen {
                            panic!("Expected close paren {:?}", token);
                        }
                    }
                    match value.as_str() {
                        "input" => Instruction::BuiltIn(BuiltIn::Input(Box::new(parse_literal(
                            tokens,
                            token.value,
                            max_size,
                        )))),
                        "output" => Instruction::BuiltIn(BuiltIn::Output(Box::new(parse_literal(
                            tokens,
                            token.value,
                            max_size,
                        )))),
                        _ => panic!("Unexpected identifier {:?}", value),
                    }
                }
                _ => panic!("Unexpected token {:?}", token),
            }
        } else {
            panic!("Unexpected end of input")
        }
    } else {
        panic!("Unexpected identifier {:?}", value)
    }
}

fn parse_test(
    tokens: &mut impl Iterator<Item = Token>,
    name: String,
    max_size: u32,
) -> Instruction {
    let mut current = Instruction::None;
    let mut block = Vec::new();
    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::OpenParen {
            panic!("Expected ( {:?}", token);
        }
    } else {
        panic!("Unexpected end of input")
    }

    let file = if let Some(token) = tokens.next() {
        if token.r#type != TokenType::Identifier {
            panic!("Expected filename {:?}", token);
        }

        PathBuf::from("./".to_string() + &token.value)
    } else {
        panic!("Unexpected end of input")
    };

    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::CloseParen {
            panic!("Expected ) {:?}", token);
        }
    } else {
        panic!("Unexpected end of input");
    }

    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::OpenBlock {
            panic!("Expected {{ {:?}", token);
        }
    } else {
        panic!("Unexpected end of input");
    }

    while let Some(token) = tokens.next() {
        match token.r#type {
            TokenType::Literal => eprintln!("Warning: Ignoring literal {:?}", token.value),
            TokenType::Identifier => {
                if current == Instruction::None {
                    current = parse_identifier(tokens, token.value, max_size);
                } else {
                    panic!("Expected semicolon, found {:?}", token.value);
                }
            }
            TokenType::OpenBlock => panic!("Blocks not supported {:?}", token),
            TokenType::CloseBlock => break,
            TokenType::OpenParen => panic!("Parens not supported {:?}", token),
            TokenType::CloseParen => panic!("Parens not supported {:?}", token),
            TokenType::SemiColon => {
                if current == Instruction::None {
                    panic!("Unexpected semicolon")
                }
                block.push(current.clone());
                current = Instruction::None;
            }
        }
    }
    Instruction::Test(block, name, file)
}

pub fn parse(tokens: Vec<Token>, max_size: u32) -> Vec<Instruction> {
    let mut tokens = tokens.into_iter();
    let mut program = Vec::new();
    while let Some(token) = tokens.next() {
        let test_name = token.value.to_string();
        match token.r#type {
            TokenType::Identifier => {
                program.push(parse_test(&mut tokens, test_name, max_size));
            }
            other => panic!("Unexpected token {:?}", other),
        }
    }

    program
}
