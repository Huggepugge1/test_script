use crate::error::{ParseError, ParseErrorType};
use crate::instruction::{BuiltIn, Instruction, InstructionType};
use crate::regex;
use crate::token::{Token, TokenCollection, TokenType};

use std::collections::HashSet;

pub struct Parser {
    tokens: TokenCollection,
    variables: HashSet<String>,
    max_size: u32,
    success: bool,
}

impl Parser {
    pub fn new(tokens: TokenCollection, max_size: u32) -> Self {
        return Self {
            tokens,
            variables: HashSet::new(),
            max_size,
            success: true,
        };
    }

    pub fn parse(&mut self) -> Result<Vec<Instruction>, ParseError> {
        let mut program = Vec::new();

        while let Some(token) = self.tokens.peek() {
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
                Err(error) => match error.r#type {
                    ParseErrorType::TestError => (),
                    _ => error.print(),
                },
            }
        }

        match self.success {
            true => Ok(program),

            false => Err(ParseError::new(
                ParseErrorType::TestError,
                self.tokens.current().unwrap(),
                "",
            )),
        }
    }

    fn parse_statement(&mut self) -> Result<Instruction, ParseError> {
        let instruction = self.parse_expression()?;
        self.end_statement()?;
        Ok(instruction)
    }

    fn parse_expression(&mut self) -> Result<Instruction, ParseError> {
        let mut token = self.peek_next_token()?;
        let mut instruction = match token.r#type {
            TokenType::StringLiteral => self.parse_string_literal()?,
            TokenType::RegexLiteral => self.parse_regex_literal()?,
            TokenType::Keyword => self.parse_keyword()?,
            TokenType::BuiltIn => self.parse_builtin()?,
            TokenType::Identifier => self.parse_identifier()?,
            TokenType::OpenBlock => self.parse_block()?,
            TokenType::Semicolon => Instruction::NONE,
            _ => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::NotImplemented,
                    token,
                    "See discord for more information about comming features",
                ));
            }
        };

        token = self.get_next_token()?;
        while token.operator() {
            instruction = self.parse_operator()?;
            token = self.get_next_token()?;
        }

        self.tokens.back();
        Ok(instruction)
    }

    fn parse_test(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        self.expect_token(TokenType::OpenParen)?;
        let path = self.parse_string_literal()?;
        let path = match path.r#type {
            InstructionType::StringLiteral(path) => path,
            _ => unreachable!(),
        };
        self.expect_token(TokenType::CloseParen)?;
        let instruction = self.parse_statement()?;

        Ok(Instruction::new(
            InstructionType::Test(Box::new(instruction), token.value.clone(), path.into()),
            token,
        ))
    }

    fn parse_string_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;

        if token.r#type != TokenType::StringLiteral {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedType(TokenType::StringLiteral, token.clone().r#type),
                token.clone(),
                format!("Token {:?} is not a string literal", token.value),
            ))
        } else {
            Ok(Instruction::new(
                InstructionType::StringLiteral(token.value.clone()),
                token,
            ))
        }
    }

    fn parse_operator(&mut self) -> Result<Instruction, ParseError> {
        unreachable!();
    }

    fn parse_regex_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;

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
                token,
            ))
        }
    }

    fn parse_keyword(&mut self) -> Result<Instruction, ParseError> {
        let token = self.peek_next_token()?;
        match token.value.as_str() {
            "for" => self.parse_for(),
            "in" => Err(ParseError::new(
                ParseErrorType::UnexpectedToken,
                token.clone(),
                "\"in\" is not allowed outside of a for loop",
            )),
            _ => unreachable!(),
        }
    }

    fn parse_assignment(&mut self) -> Result<Instruction, ParseError> {
        let identifier = self.get_next_token()?;
        if identifier.r#type != TokenType::Identifier {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::MismatchedType(TokenType::Identifier, identifier.r#type.clone()),
                identifier,
                "A \"for\" or \"let\" keyword should always be followed by an identifier",
            ));
        }
        let assignment = self.get_next_token()?;
        if assignment.r#type != TokenType::AssignmentOperator {
            return Err(ParseError::new(
                ParseErrorType::MismatchedType(
                    TokenType::AssignmentOperator,
                    assignment.r#type.clone(),
                ),
                assignment,
                "An identifier should be followed by an assignment operator in an assignment",
            ));
        }
        let instruction = self.parse_expression()?;
        Ok(Instruction::new(
            InstructionType::IterableAssignment(identifier.value.clone(), Box::new(instruction)),
            identifier,
        ))
    }

    fn parse_identifier(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        if !self.variables.contains(&token.value) {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::VariableNotDefined,
                token.clone(),
                format!("Variable \"{}\" is not defined", token.value),
            ))
        } else {
            Ok(Instruction::new(
                InstructionType::Variable(token.value.clone()),
                token,
            ))
        }
    }

    fn parse_builtin(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        self.expect_token(TokenType::OpenParen)?;
        let close_paren = self.get_next_token()?;
        let instruction = match close_paren.r#type {
            TokenType::CloseParen => Ok(Instruction::NONE),
            _ => {
                self.tokens.back();
                self.parse_expression()
            }
        }?;

        self.expect_token(TokenType::CloseParen)?;
        match token.value.as_str() {
            "input" => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn::Input(Box::new(instruction))),
                token,
            )),
            "output" => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn::Output(Box::new(instruction))),
                token,
            )),
            "print" => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn::Print(Box::new(instruction))),
                token,
            )),
            "println" => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn::Println(Box::new(instruction))),
                token,
            )),
            _ => unreachable!(),
        }
    }

    fn parse_block(&mut self) -> Result<Instruction, ParseError> {
        let mut token = self.get_next_token()?;
        let mut block = Vec::new();
        while token.r#type != TokenType::CloseBlock {
            match self.parse_statement() {
                Ok(instruction) => block.push(instruction),
                Err(e) => {
                    e.print();
                    self.success = false;
                }
            }
            token = self.peek_next_token()?;
        }
        Ok(Instruction::new(InstructionType::Block(block), token))
    }

    fn parse_for(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;

        let assignment = self.parse_assignment()?;
        self.variables.insert(assignment.get_variable_name()?);

        let statement = self.parse_expression()?;

        Ok(Instruction::new(
            InstructionType::For(Box::new(statement), Box::new(assignment)),
            token,
        ))
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
        match token.r#type {
            TokenType::Semicolon | TokenType::CloseBlock => Ok(()),
            _ => {
                self.tokens.back();
                Err(ParseError::new(
                    ParseErrorType::MismatchedType(TokenType::Semicolon, token.clone().r#type),
                    token,
                    "Did you forget a semicolon?",
                ))
            }
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
