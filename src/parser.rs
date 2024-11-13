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
        let instruction = self.parse_expression(true)?;
        self.end_statement()?;
        Ok(instruction)
    }

    fn parse_expression(&mut self, parse_binary: bool) -> Result<Instruction, ParseError> {
        let mut token = self.peek_next_token()?;
        let mut instruction = match token.r#type {
            TokenType::StringLiteral => self.parse_string_literal()?,
            TokenType::RegexLiteral => self.parse_regex_literal()?,
            TokenType::IntegerLiteral => self.parse_integer_literal()?,
            TokenType::BooleanLiteral => self.parse_boolean_literal()?,

            TokenType::Keyword => self.parse_keyword()?,
            TokenType::BuiltIn => self.parse_builtin()?,

            TokenType::Identifier => self.parse_identifier()?,

            TokenType::OpenBlock => self.parse_block()?,
            TokenType::OpenParen => self.parse_parentheses()?,

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
            instruction = match token.r#type {
                TokenType::BinaryOperator => match parse_binary {
                    true => self.parse_operator(instruction)?,
                    false => break,
                },
                TokenType::TypeCast => self.parse_type_cast(instruction)?,
                TokenType::AssignmentOperator => self.parse_assignment(instruction)?,
                _ => unreachable!(),
            };
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
        let new_operator = match token.value.as_str() {
            "+" => BinaryOperator::Addition,
            "-" => BinaryOperator::Subtraction,
            "*" => BinaryOperator::Multiplication,
            "/" => BinaryOperator::Division,
            _ => unreachable!(),
        };

        let new_right = self.parse_expression(false)?;
        match last_instruction.r#type {
            InstructionType::BinaryOperation {
                ref operator,
                ref left,
                ref right,
            } => Ok(Instruction::new(
                if new_operator.cmp(&operator) != std::cmp::Ordering::Greater {
                    InstructionType::BinaryOperation {
                        operator: new_operator,
                        left: Box::new(last_instruction),
                        right: Box::new(new_right),
                    }
                } else {
                    InstructionType::BinaryOperation {
                        operator: operator.clone(),
                        left: left.clone(),
                        right: Box::new(Instruction::new(
                            InstructionType::BinaryOperation {
                                operator: new_operator,
                                left: right.clone(),
                                right: Box::new(new_right),
                            },
                            token.clone(),
                        )),
                    }
                },
                token,
            )),
            _ => Ok(Instruction::new(
                InstructionType::BinaryOperation {
                    operator: new_operator,
                    left: Box::new(last_instruction),
                    right: Box::new(new_right),
                },
                token,
            )),
        }
    }

    fn parse_type_cast(
        &mut self,
        last_instruction: Instruction,
    ) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let r#type = match self.get_next_token()? {
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

    fn parse_boolean_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        if token.r#type != TokenType::BooleanLiteral {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedTokenType(
                    TokenType::BooleanLiteral,
                    token.clone().r#type,
                ),
                token.clone(),
                format!("Token {:?} is not a boolean literal", token.value),
            ))
        } else {
            Ok(Instruction::new(
                InstructionType::BooleanLiteral(token.value.parse().unwrap()),
                token,
            ))
        }
    }

    fn parse_keyword(&mut self) -> Result<Instruction, ParseError> {
        let token = self.peek_next_token()?;
        match token.value.as_str() {
            "let" => self.parse_declaration(),
            "const" => self.parse_declaration(),
            "for" => self.parse_for(),
            "in" => {
                self.tokens.advance_to_next_instruction();
                Err(ParseError::new(
                    ParseErrorType::UnexpectedToken,
                    token.clone(),
                    "\"in\" is not allowed outside of a for loop",
                ))
            }
            "if" => self.parse_conditional(),
            "else" => {
                self.tokens.advance_to_next_instruction();
                Err(ParseError::new(
                    ParseErrorType::UnexpectedToken,
                    token.clone(),
                    "\"else\" is not allowed outside of an if statement",
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_declaration(&mut self) -> Result<Instruction, ParseError> {
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
        let instruction = self.parse_expression(true)?;
        let variable = Variable::new(
            identifier_name.clone(),
            token.value == "const",
            r#type.clone(),
        );
        match assignment.value.as_str() {
            "=" => {
                self.environment.insert(variable.clone());
                Ok(Instruction::new(
                    InstructionType::Assignment {
                        variable,
                        instruction: Box::new(instruction),
                    },
                    token,
                ))
            }
            "in" => {
                self.environment.insert(variable.clone());
                Ok(Instruction::new(
                    InstructionType::IterableAssignment(variable, Box::new(instruction)),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_assignment(&mut self, instruction: Instruction) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let variable = match instruction.r#type {
            InstructionType::Variable(variable) => variable,
            _ => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::UnexpectedToken,
                    token.clone(),
                    "An asignment operator should always be preceded by a variable",
                ));
            }
        };

        if variable.r#const {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::VariableIsConstant,
                instruction.token.clone(),
                format!(
                    "Variable \"{}\" is a constant and cannot be reassigned",
                    variable.name
                ),
            ));
        }

        if token.r#type != TokenType::AssignmentOperator {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::MismatchedTokenType(
                    TokenType::AssignmentOperator,
                    token.r#type.clone(),
                ),
                token,
                "An identifier should always be followed by an assignment operator in an assignment",
            ));
        }

        let instruction = self.parse_expression(true)?;
        if self.environment.get(&variable.name).is_none() {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::VariableNotDefined,
                token.clone(),
                format!("Variable \"{}\" is not defined", variable.name),
            ));
        }

        Ok(Instruction::new(
            InstructionType::Assignment {
                variable,
                instruction: Box::new(instruction),
            },
            token,
        ))
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
                InstructionType::Variable(self.environment.get(&token.value).unwrap().clone()),
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
                self.parse_expression(true)
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

    fn parse_conditional(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let condition = self.parse_expression(true)?;
        let statement = self.parse_statement()?;
        let r#else = match self.peek_next_token()?.value.as_str() {
            "else" => {
                self.get_next_token()?;
                self.parse_statement()?
            }
            _ => Instruction::NONE,
        };

        self.tokens.back();

        Ok(Instruction::new(
            InstructionType::Conditional {
                condition: Box::new(condition),
                instruction: Box::new(statement),
                r#else: Box::new(r#else),
            },
            token,
        ))
    }

    fn parse_for(&mut self) -> Result<Instruction, ParseError> {
        let token = self.peek_next_token()?;

        self.environment.add_scope();

        let assignment = self.parse_declaration();

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

    fn parse_parentheses(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let instruction = self.parse_expression(true)?;
        self.expect_token(TokenType::CloseParen)?;
        Ok(Instruction::new(
            InstructionType::Paren(Box::new(instruction)),
            token,
        ))
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
