use crate::cli::Args;
use crate::environment::ParseEnvironment;
use crate::error::{ParseError, ParseErrorType};
use crate::instruction::{BinaryOperator, BuiltIn, Instruction, InstructionType};
use crate::r#type::Type;
use crate::regex;
use crate::token::{Token, TokenCollection, TokenType};
use crate::variable::Variable;

pub struct Parser {
    tokens: TokenCollection,
    environment: ParseEnvironment,
    args: Args,
    success: bool,
}

impl Parser {
    pub fn new(tokens: TokenCollection, args: Args) -> Self {
        return Self {
            tokens,
            environment: ParseEnvironment::new(),
            args,
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
                        ParseErrorType::MismatchedTokenType(TokenType::Identifier, r#type),
                        token,
                        "Only test names are allowed in the main scope",
                    ))
                }
            };

            match instruction {
                Ok(instruction) => program.push(instruction),
                Err(e) => match e.r#type {
                    ParseErrorType::TestError => (),
                    _ => e.print(),
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
            TokenType::IntegerLiteral => self.parse_integer_literal()?,
            TokenType::Keyword => self.parse_keyword()?,
            TokenType::BuiltIn => self.parse_builtin()?,
            TokenType::Identifier => self.parse_identifier()?,
            TokenType::OpenBlock => self.parse_block()?,
            TokenType::Semicolon => Instruction::new(InstructionType::None, token.clone()),
            _ => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::NotImplemented,
                    token,
                    "See discord for more information about comming features",
                ));
            }
        };

        token = self.peek_next_token()?;
        while token.binary_operator() {
            instruction = self.parse_operator(instruction)?;
            token = self.peek_next_token()?;
        }

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
                ParseErrorType::MismatchedTokenType(TokenType::StringLiteral, token.clone().r#type),
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

    fn parse_operator(&mut self, last_instruction: Instruction) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        match token.value.as_str() {
            "+" => Ok(Instruction::new(
                InstructionType::BinaryOperation {
                    operator: BinaryOperator::Addition,
                    left: Box::new(last_instruction),
                    right: Box::new(self.parse_expression()?),
                },
                token,
            )),
            "as" => self.parse_type_cast(last_instruction),
            "=" => Ok(Instruction::new(
                InstructionType::Assignment(
                    Variable::new(
                        last_instruction.get_variable_name().unwrap(),
                        last_instruction.get_variable_type().unwrap(),
                    ),
                    Box::new(self.parse_expression()?),
                ),
                token,
            )),
            _ => unreachable!(),
        }
    }

    fn parse_type_cast(
        &mut self,
        last_instruction: Instruction,
    ) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let r#type = match token {
            Token {
                r#type: TokenType::Type,
                ref value,
                ..
            } => Type::from(&value),
            _ => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::MismatchedTokenType(TokenType::Type, token.clone().r#type),
                    token.clone(),
                    "The \"as\" keyword should always be followed by a type",
                ));
            }
        };
        Ok(Instruction::new(
            InstructionType::TypeCast {
                instruction: Box::new(last_instruction),
                r#type,
            },
            token,
        ))
    }

    fn parse_regex_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;

        if token.r#type != TokenType::RegexLiteral {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedTokenType(TokenType::RegexLiteral, token.clone().r#type),
                token.clone(),
                format!("Token {:?} is not a regex literal", token.value),
            ))
        } else {
            Ok(Instruction::new(
                InstructionType::RegexLiteral(regex::parse(&token, self.args.max_size)?),
                token,
            ))
        }
    }

    fn parse_integer_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        if token.r#type != TokenType::IntegerLiteral {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedTokenType(
                    TokenType::IntegerLiteral,
                    token.clone().r#type,
                ),
                token.clone(),
                format!("Token {:?} is not an integer literal", token.value),
            ))
        } else {
            Ok(Instruction::new(
                InstructionType::IntegerLiteral(token.value.parse().unwrap()),
                token,
            ))
        }
    }

    fn parse_keyword(&mut self) -> Result<Instruction, ParseError> {
        let token = self.peek_next_token()?;
        match token.value.as_str() {
            "let" => self.parse_assignment(),
            "for" => self.parse_for(),
            "in" => {
                self.tokens.advance_to_next_instruction();
                Err(ParseError::new(
                    ParseErrorType::UnexpectedToken,
                    token.clone(),
                    "\"in\" is not allowed outside of a for loop",
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_assignment(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let identifier = self.get_next_token()?;
        let identifier_name = identifier.value.clone();
        if identifier.r#type != TokenType::Identifier {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::MismatchedTokenType(
                    TokenType::Identifier,
                    identifier.r#type.clone(),
                ),
                identifier,
                "A \"for\" or \"let\" keyword should always be followed by an identifier",
            ));
        }

        self.expect_token(TokenType::Colon)?;

        let r#type = self.get_next_token()?;
        if r#type.r#type != TokenType::Type {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::MismatchedTokenType(TokenType::Type, r#type.r#type.clone()),
                r#type,
                "A colon should always be followed by a type in an assignment",
            ));
        }
        let r#type = Type::from(&r#type.value.clone());

        let assignment = self.get_next_token()?;
        if assignment.r#type != TokenType::AssignmentOperator {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::MismatchedTokenType(
                    TokenType::AssignmentOperator,
                    assignment.r#type.clone(),
                ),
                assignment,
                "A type should be followed by an assignment operator in an assingnment",
            ));
        }
        let instruction = self.parse_expression()?;
        match assignment.value.as_str() {
            "=" => {
                self.environment
                    .insert(Variable::new(identifier_name.clone(), r#type.clone()));
                Ok(Instruction::new(
                    InstructionType::Assignment(
                        Variable::new(identifier_name, r#type),
                        Box::new(instruction),
                    ),
                    token,
                ))
            }
            "in" => {
                self.environment
                    .insert(Variable::new(identifier_name.clone(), r#type.clone()));
                Ok(Instruction::new(
                    InstructionType::IterableAssignment(
                        Variable::new(identifier_name, r#type),
                        Box::new(instruction),
                    ),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_identifier(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        if self.environment.get(&token.value).is_none() {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::VariableNotDefined,
                token.clone(),
                format!("Variable \"{}\" is not defined", token.value),
            ))
        } else {
            Ok(Instruction::new(
                InstructionType::Variable(Variable::new(
                    token.value.clone(),
                    self.environment.get(&token.value).unwrap().clone(),
                )),
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
        let token = self.get_next_token()?;
        let mut block = Vec::new();
        self.environment.add_scope();

        let mut next_token = self.peek_next_token()?;
        while next_token.r#type != TokenType::CloseBlock {
            match self.parse_statement() {
                Ok(instruction) => block.push(instruction),
                Err(e) => {
                    e.print();
                    self.success = false;
                }
            }
            next_token = self.peek_next_token()?;
        }
        self.environment.remove_scope();
        Ok(Instruction::new(InstructionType::Block(block), token))
    }

    fn parse_for(&mut self) -> Result<Instruction, ParseError> {
        let token = self.peek_next_token()?;

        self.environment.add_scope();

        let assignment = self.parse_assignment();
        match assignment {
            Ok(ref assignment) => {
                let name = assignment.get_variable_name().unwrap();
                let r#type = assignment.get_variable_type().unwrap();
                self.environment.insert(Variable::new(name, r#type));
            }

            Err(ref e) => {
                e.print();
                self.success = false;
            }
        }

        let statement = self.parse_statement();
        self.environment.remove_scope();
        let statement = match statement {
            Ok(statement) => statement,
            Err(e) => {
                return Err(e);
            }
        };

        self.tokens.back();

        match assignment {
            Ok(assignment) => Ok(Instruction::new(
                InstructionType::For(Box::new(assignment), Box::new(statement)),
                token,
            )),
            Err(_) => Err(ParseError::none()),
        }
    }

    fn expect_token(&mut self, expected: TokenType) -> Result<(), ParseError> {
        let token = self.get_next_token()?;
        if token.r#type != expected {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedTokenType(expected, token.clone().r#type),
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
                    ParseErrorType::MismatchedTokenType(TokenType::Semicolon, token.clone().r#type),
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
