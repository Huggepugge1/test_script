use crate::error::{ParseError, ParseErrorType, ParseWarning, ParseWarningType};
use crate::instruction::{BuiltIn, Instruction, InstructionType};
use crate::regex;
use crate::token::{Token, TokenCollection, TokenType};

use std::collections::HashSet;

pub struct Parser {
    tokens: TokenCollection,
    variables: HashSet<String>,
    max_size: u32,
}

impl Parser {
    pub fn new(tokens: TokenCollection, max_size: u32) -> Self {
        return Self {
            tokens,
            variables: HashSet::new(),
            max_size,
        };
    }

    pub fn parse(&mut self) -> Result<Vec<Instruction>, ParseError> {
        let mut program = Vec::new();
        let mut failed = false;

        while let Some(token) = self.tokens.next() {
            let instruction = match token.clone().r#type {
                TokenType::Identifier => self.parse_test(),
                r#type => {
                    self.tokens.advance_to_next_instruction();
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
                    match error.r#type {
                        ParseErrorType::TestError => (),
                        _ => error.print(),
                    }
                    failed = true;
                }
            }
        }

        if failed {
            Err(ParseError::new(
                ParseErrorType::TestError,
                self.tokens.current().unwrap(),
                "",
            ))
        } else {
            Ok(program)
        }
    }

    fn parse_statement(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();
        let instruction = match token.r#type {
            TokenType::Semicolon => {
                ParseWarning::new(
                    ParseWarningType::ExtraSemicolon,
                    token,
                    "Remove the trailing semicolon",
                )
                .print();
                self.tokens.next();
                return Ok(Instruction::NONE);
            }
            TokenType::StringLiteral => {
                ParseWarning::new(
                    ParseWarningType::UnusedLiteral,
                    token,
                    "This literal is not being used in the program",
                );
                self.parse_string_literal()
            }
            _ => self.parse_expression(),
        }?;
        match instruction.r#type {
            InstructionType::For(_, _) => {
                self.tokens.next();
            }
            _ => self.end_statement()?,
        }
        Ok(instruction)
    }

    fn parse_expression(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();
        let instruction = match token.r#type {
            TokenType::StringLiteral => self.parse_string_literal(),
            TokenType::RegexLiteral => self.parse_regex_literal(),
            TokenType::Keyword => self.parse_keyword(),
            TokenType::BuiltIn => self.parse_builtin(),
            TokenType::Identifier => self.parse_identifier(),
            TokenType::Semicolon => Err(ParseError::new(
                ParseErrorType::UnexpectedToken,
                token,
                "Semicolon found in the middle of an expression",
            )),
            _ => {
                self.tokens.advance_to_next_instruction();
                Err(ParseError::new(
                    ParseErrorType::NotImplemented,
                    token,
                    "See discord for more information about comming features",
                ))
            }
        };

        instruction
    }

    fn parse_test(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();
        self.expect_token(TokenType::OpenParen)?;
        self.tokens.next();
        let path = self.parse_string_literal()?;
        let path = match path.r#type {
            InstructionType::StringLiteral(path) => path,
            _ => unreachable!(),
        };
        self.expect_token(TokenType::CloseParen)?;
        self.expect_token(TokenType::OpenBlock)?;
        self.tokens.next();

        let (block, failed) = self.parse_block();

        match failed {
            true => Err(ParseError::new(ParseErrorType::TestError, token, "")),
            false => Ok(Instruction::new(
                InstructionType::Test(block, token.value, path.into()),
                token.line,
                token.column,
            )),
        }
    }

    fn parse_string_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();

        if token.r#type != TokenType::StringLiteral {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedType(TokenType::StringLiteral, token.clone().r#type),
                token.clone(),
                format!("Token {:?} is not a string literal", token.value),
            ))
        } else {
            Ok(Instruction::new(
                InstructionType::StringLiteral(token.value),
                token.line,
                token.column,
            ))
        }
    }

    fn parse_regex_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();

        if token.r#type != TokenType::RegexLiteral {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedType(TokenType::RegexLiteral, token.clone().r#type),
                token.clone(),
                format!("Token {:?} is not a regex literal", token.value),
            ))
        } else {
            Ok(Instruction::new(
                InstructionType::RegexLiteral(regex::parse(&token, self.max_size)?),
                token.line,
                token.column,
            ))
        }
    }

    fn parse_keyword(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();
        match token.value.as_str() {
            "for" => self.parse_for(),
            "in" => Err(ParseError::new(
                ParseErrorType::UnexpectedToken,
                token.clone(),
                "'in' is not allowed outside of a for loop",
            )),
            _ => unreachable!(),
        }
    }

    fn parse_identifier(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();
        let assignment = self.peek_next_token()?;
        if assignment.assignment() == false {
            if !self.variables.contains(&token.value) {
                self.tokens.advance_to_next_instruction();
                Err(ParseError::new(
                    ParseErrorType::VariableNotDefined,
                    token.clone(),
                    format!("Variable \"{}\" is not defined", token.value),
                ))
            } else {
                Ok(Instruction::new(
                    InstructionType::Variable(token.value),
                    token.line,
                    token.column,
                ))
            }
        } else {
            self.tokens.next();
            let identifier = token.value.clone();
            if self.variables.contains(&identifier) {
                self.tokens.advance_to_next_instruction();
                Err(ParseError::new(
                    ParseErrorType::VariableAlreadyDefined,
                    token.clone(),
                    format!("Variable {} is already defined", identifier),
                ))
            } else {
                self.variables.insert(identifier.clone());
                self.tokens.next();
                let instruction = self.parse_expression()?;
                Ok(Instruction::new(
                    InstructionType::Assignment(identifier, Box::new(instruction)),
                    token.line,
                    token.column,
                ))
            }
        }
    }

    fn parse_builtin(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();
        self.expect_token(TokenType::OpenParen)?;
        let close_paren = self.get_next_token()?;
        let instruction = match close_paren.r#type {
            TokenType::CloseParen => Ok(Instruction::NONE),
            _ => self.parse_expression(),
        }?;

        self.expect_token(TokenType::CloseParen)?;
        match token.value.as_str() {
            "input" => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn::Input(Box::new(instruction))),
                token.line,
                token.column,
            )),
            "output" => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn::Output(Box::new(instruction))),
                token.line,
                token.column,
            )),
            "print" => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn::Print(Box::new(instruction))),
                token.line,
                token.column,
            )),
            "println" => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn::Println(Box::new(instruction))),
                token.line,
                token.column,
            )),
            _ => unreachable!(),
        }
    }

    fn parse_block(&mut self) -> (Vec<Instruction>, bool) {
        let mut block = Vec::new();
        let mut failed = false;
        while let Some(token) = self.tokens.current() {
            if token.r#type == TokenType::CloseBlock {
                break;
            }
            match self.parse_statement() {
                Ok(instruction) => block.push(instruction),
                Err(e) => {
                    e.print();
                    failed = true;
                }
            }
        }
        (block, failed)
    }

    fn parse_for(&mut self) -> Result<Instruction, ParseError> {
        let token = self.tokens.current().unwrap();
        self.expect_token(TokenType::Identifier)?;
        let assignment = self.parse_identifier()?;
        self.expect_token(TokenType::OpenBlock)?;
        self.tokens.next();

        let (block, failed) = self.parse_block();

        match failed {
            true => Err(ParseError::new(ParseErrorType::TestError, token, "")),
            false => Ok(Instruction::new(
                InstructionType::For(block, Box::new(assignment)),
                token.line,
                token.column,
            )),
        }
    }

    fn expect_token(&mut self, expected: TokenType) -> Result<(), ParseError> {
        let token = self.get_next_token()?;
        if token.r#type != expected {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedType(expected, token.clone().r#type),
                token.clone(),
                format!("Token {:?} is not of the right type", token.value),
            ))
        } else {
            Ok(())
        }
    }

    fn end_statement(&mut self) -> Result<(), ParseError> {
        let token = self.get_next_token()?;
        if token.r#type == TokenType::Semicolon || token.r#type == TokenType::CloseBlock {
            self.tokens.next();
            Ok(())
        } else {
            Err(ParseError::new(
                ParseErrorType::MismatchedType(TokenType::Semicolon, token.clone().r#type),
                token.clone(),
                "Did you forget a semicolon?",
            ))
        }
    }

    fn get_next_token(&mut self) -> Result<Token, ParseError> {
        if let Some(token) = self.tokens.next() {
            Ok(token)
        } else {
            Err(ParseError::new(
                ParseErrorType::UnexpectedEndOfFile,
                self.tokens.current().unwrap(),
                "",
            ))
        }
    }

    fn peek_next_token(&mut self) -> Result<Token, ParseError> {
        if let Some(token) = self.tokens.peek() {
            Ok(token)
        } else {
            Err(ParseError::new(
                ParseErrorType::UnexpectedEndOfFile,
                self.tokens.current().unwrap(),
                "",
            ))
        }
    }
}
