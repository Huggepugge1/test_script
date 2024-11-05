use crate::error::{ParseError, ParseErrorType};
use crate::lexer::{Token, TokenCollection, TokenType};
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
    For(Vec<Instruction>, Vec<String>),
    None,
}

fn parse_literal(tokens: &mut TokenCollection, max_size: u32) -> Result<Instruction, ParseError> {
    let token = tokens.current();
    let token = token.unwrap().clone();
    if token.r#type != TokenType::Literal {
        while let Some(token) = tokens.next() {
            if token.r#type == TokenType::SemiColon {
                break;
            }
        }
        Err(ParseError::new(
            ParseErrorType::Error,
            token.clone(),
            "Not a literal",
            None::<String>,
        ))
    } else {
        Ok(Instruction::new(
            InstructionType::Literal(regex::parse(&token.value, max_size)),
            token.line,
            token.column,
        ))
    }
}

fn parse_identifier(
    tokens: &mut TokenCollection,
    max_size: u32,
) -> Result<Instruction, ParseError> {
    let token = tokens.current().unwrap().clone();

    let mut next_token = tokens.next();
    if next_token.is_some() {
        let token = next_token.unwrap().clone();
        if token.r#type != TokenType::OpenParen {
            while let Some(token) = tokens.next() {
                if token.r#type == TokenType::SemiColon {
                    break;
                }
            }
            return Err(ParseError::new(
                ParseErrorType::Error,
                token,
                "Unexpected token",
                Some("Expected \"(\""),
            ));
        }
    }
    next_token = tokens.next();
    if next_token.is_some() {
        let next_token = next_token.unwrap().clone();
        match next_token.clone().r#type {
            TokenType::Literal => {
                let literal = Box::new(parse_literal(tokens, max_size)?);
                let next_token = tokens.peek();
                if next_token.is_some() {
                    let next_token = next_token.unwrap().clone();
                    if next_token.r#type != TokenType::CloseParen {
                        let _ = tokens.next();
                        while let Some(next_token) = tokens.next() {
                            if next_token.r#type == TokenType::SemiColon {
                                break;
                            }
                        }
                        return Err(ParseError::new(
                            ParseErrorType::Error,
                            next_token,
                            "Unexpected token",
                            Some("Expected \")\""),
                        ));
                    } else {
                        let _ = tokens.next();
                    }
                }
                match token.value.as_str() {
                    "input" => Ok(Instruction::new(
                        InstructionType::BuiltIn(BuiltIn::Input(literal)),
                        token.line,
                        token.column,
                    )),
                    "output" => Ok(Instruction::new(
                        InstructionType::BuiltIn(BuiltIn::Output(literal)),
                        token.line,
                        token.column,
                    )),
                    _ => {
                        while let Some(token) = tokens.next() {
                            if token.r#type == TokenType::SemiColon {
                                break;
                            }
                        }
                        Err(ParseError::new(
                            ParseErrorType::Error,
                            token,
                            "Unexpected identifier",
                            None::<String>,
                        ))
                    }
                }
            }
            _ => {
                while let Some(token) = tokens.next() {
                    if token.r#type == TokenType::SemiColon {
                        break;
                    }
                }
                Err(ParseError::new(
                    ParseErrorType::Error,
                    next_token.clone(),
                    "Unexpected token",
                    None::<String>,
                ))
            }
        }
    } else {
        while let Some(token) = tokens.next() {
            if token.r#type == TokenType::SemiColon {
                break;
            }
        }
        Err(ParseError::new(
            ParseErrorType::Error,
            token.clone(),
            "Unexpected end of input",
            None::<String>,
        ))
    }
}

fn parse_keyword(tokens: &mut TokenCollection, max_size: u32) -> Result<Instruction, ParseError> {
    let token = tokens.current().unwrap().clone();
    match token.value.as_str() {
        "for" => {
            let mut current = Instruction::new(InstructionType::None, 0, 0);
            let mut block = Vec::new();
            let mut failed = false;
            let mut variables = Vec::new();
            let token = tokens.current().unwrap().clone();
            if let Some(token) = tokens.next() {
                if token.r#type != TokenType::OpenParen {
                    while let Some(token) = tokens.next() {
                        if token.r#type == TokenType::SemiColon {
                            break;
                        }
                    }
                    return Err(ParseError::new(
                        ParseErrorType::Error,
                        token.clone(),
                        "Unexpected token",
                        Some("Expected \"(\""),
                    ));
                }
            } else {
                return Err(ParseError::new(
                    ParseErrorType::Error,
                    token.clone(),
                    "Unexpected end of input",
                    None::<String>,
                ));
            }

            while let Some(token) = tokens.next() {
                match token.r#type {
                    TokenType::Literal => {
                        variables.push(token.value.clone());
                    }
                    TokenType::Identifier => {
                        if current.r#type == InstructionType::None {
                            match parse_identifier(tokens, max_size) {
                                Ok(instruction) => current = instruction,
                                Err(message) => {
                                    message.print();
                                    failed = true;
                                }
                            }
                        } else {
                            ParseError::new(
                                ParseErrorType::Error,
                                token.clone(),
                                "Unexpected token",
                                Some("Did you forget a semicolon?"),
                            )
                            .print();
                            failed = true;
                            let token = token.clone();
                            tokens.insert(Token::new(
                                TokenType::SemiColon,
                                &";".to_string(),
                                token.line,
                                token.column,
                            ));
                        }
                    }
                    TokenType::Keyword => match parse_keyword(tokens, max_size) {
                        Ok(instruction) => {
                            if current.r#type == InstructionType::None {
                                current = instruction;
                            } else {
                                ParseError::new(
                                    ParseErrorType::Error,
                                    token.clone(),
                                    "Unexpected token",
                                    Some("Did you forget a semicolon?"),
                                )
                                .print();
                                failed = true;
                            }
                        }
                        Err(message) => {
                            message.print();
                            failed = true;
                        }
                    },
                    TokenType::OpenBlock => ParseError::new(
                        ParseErrorType::Error,
                        token.clone(),
                        "Blocks not supported",
                        None::<String>,
                    ),
                }
            }
        }
    }
}

fn parse_test(tokens: &mut TokenCollection, max_size: u32) -> Result<Instruction, ParseError> {
    let mut current = Instruction::new(InstructionType::None, 0, 0);
    let mut block = Vec::new();
    let mut failed = false;
    let token = tokens.current().unwrap().clone();
    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::OpenParen {}
    } else {
        return Err(ParseError::new(
            ParseErrorType::Error,
            token,
            "Unexpected end of input",
            None::<String>,
        ));
    }

    let file = if let Some(token) = tokens.next() {
        if token.r#type != TokenType::Literal {
            return Err(ParseError::new(
                ParseErrorType::Error,
                token.clone(),
                "Unexpected token",
                Some("Expected file path as a literal"),
            ));
        }

        PathBuf::from("./".to_string() + &token.value)
    } else {
        return Err(ParseError::new(
            ParseErrorType::Error,
            token.clone(),
            "Unexpected end of input",
            Some("Expected file path as a literal"),
        ));
    };

    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::CloseParen {
            return Err(ParseError::new(
                ParseErrorType::Error,
                token.clone(),
                "Unexpected Token",
                Some("Expected \")\""),
            ));
        }
    } else {
        return Err(ParseError::new(
            ParseErrorType::Error,
            token.clone(),
            "Unexpected end of input",
            None::<String>,
        ));
    }

    if let Some(token) = tokens.next() {
        if token.r#type != TokenType::OpenBlock {
            return Err(ParseError::new(
                ParseErrorType::Error,
                token.clone(),
                "Unexpected Token",
                Some("Expected \"{\""),
            ));
        }
    } else {
        return Err(ParseError::new(
            ParseErrorType::Error,
            token.clone(),
            "Unexpected end of input",
            Some("Expected \"{\""),
        ));
    }

    while let Some(token) = tokens.next() {
        match token.r#type {
            TokenType::Literal => {
                ParseError::new(
                    ParseErrorType::Warning,
                    token.clone(),
                    "Ignoring Literal",
                    Some("Remove the literal"),
                )
                .print();
            }
            TokenType::Identifier => {
                if current.r#type == InstructionType::None {
                    match parse_identifier(tokens, max_size) {
                        Ok(instruction) => current = instruction,
                        Err(message) => {
                            message.print();
                            failed = true;
                        }
                    }
                } else {
                    ParseError::new(
                        ParseErrorType::Error,
                        token.clone(),
                        "Unexpected token",
                        Some("Did you forget a semicolon?"),
                    )
                    .print();
                    failed = true;
                    let token = token.clone();
                    tokens.insert(Token::new(
                        TokenType::SemiColon,
                        &";".to_string(),
                        token.line,
                        token.column,
                    ));
                }
            }
            TokenType::Keyword => {
                if current.r#type == InstructionType::None {
                    match parse_keyword(tokens, max_size) {
                        Ok(instruction) => current = instruction,
                        Err(message) => {
                            message.print();
                            failed = true;
                        }
                    }
                } else {
                    ParseError::new(
                        ParseErrorType::Error,
                        token.clone(),
                        "Unexpected token",
                        Some("Did you forget a semicolon?"),
                    )
                    .print();
                    failed = true;
                    let token = token.clone();
                    tokens.insert(Token::new(
                        TokenType::SemiColon,
                        &";".to_string(),
                        token.line,
                        token.column,
                    ));
                }
            }
            TokenType::OpenBlock => ParseError::new(
                ParseErrorType::Error,
                token.clone(),
                "Blocks not supported",
                None::<String>,
            )
            .print(),

            TokenType::CloseBlock => {
                if current.r#type != InstructionType::None {
                    ParseError::new(
                        ParseErrorType::Error,
                        token.clone(),
                        "Unexpected token",
                        Some("Did you forget a semicolon?"),
                    )
                    .print();
                    failed = true;
                }
                break;
            }
            TokenType::OpenParen => ParseError::new(
                ParseErrorType::Error,
                token.clone(),
                "Parens not supported",
                None::<String>,
            )
            .print(),

            TokenType::CloseParen => ParseError::new(
                ParseErrorType::Error,
                token.clone(),
                "Parens not supported",
                None::<String>,
            )
            .print(),
            TokenType::SemiColon => {
                if current.r#type == InstructionType::None {
                    ParseError::new(
                        ParseErrorType::Warning,
                        token.clone(),
                        "Extra semicolon",
                        Some("Remove the semicolon"),
                    )
                    .print();
                }
                block.push(current.clone());
                current = Instruction::NONE;
            }
        }
    }
    if failed {
        Err(ParseError::new(
            ParseErrorType::Error,
            token.clone(),
            "Failed to parse test",
            None::<String>,
        ))
    } else {
        Ok(Instruction::new(
            InstructionType::Test(block, token.value, file),
            token.line,
            token.column,
        ))
    }
}

pub fn parse(tokens: &mut TokenCollection, max_size: u32) -> Result<Vec<Instruction>, ()> {
    let mut program = Vec::new();
    let mut failed = false;
    while let Some(token) = tokens.next() {
        match token.r#type {
            TokenType::Identifier => {
                let instruction = parse_test(tokens, max_size);
                match instruction {
                    Ok(instruction) => program.push(instruction),
                    Err(message) => {
                        message.print();
                        failed = true;
                    }
                }
            }
            _ => {
                ParseError::new(
                    ParseErrorType::Error,
                    token.clone(),
                    "Unexpected token",
                    Some("Expected a test block"),
                )
                .print();
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
