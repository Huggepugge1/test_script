use crate::cli::Args;
use crate::environment::ParseEnvironment;
use crate::error::{ParseError, ParseErrorType, ParseWarning, ParseWarningType};
use crate::instruction::{BinaryOperator, BuiltIn, Instruction, InstructionType, UnaryOperator};
use crate::r#type::Type;
use crate::regex;
use crate::token::{Token, TokenCollection, TokenType};
use crate::variable::{SnakeCase, Variable};
use crate::white_listed_constants;

pub struct Parser {
    tokens: TokenCollection,
    environment: ParseEnvironment,
    args: Args,
    in_constant_declaration: bool,
    success: bool,
}

impl Parser {
    pub fn new(tokens: TokenCollection, args: Args) -> Self {
        Self {
            tokens,
            environment: ParseEnvironment::new(args.clone()),
            args,
            in_constant_declaration: false,
            success: true,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Instruction>, Vec<Instruction>> {
        let mut program = Vec::new();

        while let Some(token) = self.tokens.peek() {
            let instruction = match token.clone().r#type {
                TokenType::Identifier { .. } => self.parse_test(),
                TokenType::Keyword { value } => match value.as_str() {
                    "const" => self.parse_statement(),
                    "fn" => self.parse_function(),
                    _ => {
                        self.tokens.advance_to_next_instruction();
                        Err(ParseError::new(
                            ParseErrorType::GlobalScope(token.clone().r#type),
                            token,
                        ))
                    }
                },
                TokenType::OpenBlock | TokenType::CloseBlock => {
                    self.tokens.next();
                    Err(ParseError::new(
                        ParseErrorType::GlobalScope(token.clone().r#type),
                        token.clone(),
                    ))
                }

                r#type => {
                    self.tokens.advance_to_next_instruction();
                    Err(ParseError::new(
                        ParseErrorType::GlobalScope(r#type),
                        token.clone(),
                    ))
                }
            };

            match instruction {
                Ok(instruction) => program.push(instruction),
                Err(e) => e.print(),
            }
        }

        match self.success {
            true => Ok(program),
            false => Err(program),
        }
    }

    fn parse_statement(&mut self) -> Result<Instruction, ParseError> {
        let instruction = self.parse_expression(true, true)?;
        match self.end_statement() {
            Ok(_) => (),
            Err(e) => {
                e.print();
                self.success = false;
            }
        }

        Ok(instruction)
    }

    fn parse_expression(
        &mut self,
        parse_binary: bool,
        parse_type_cast: bool,
    ) -> Result<Instruction, ParseError> {
        let mut token = self.peek_next_token()?;
        let mut instruction = match &token.r#type {
            TokenType::StringLiteral { .. } => self.parse_string_literal()?,
            TokenType::RegexLiteral { .. } => self.parse_regex_literal()?,
            TokenType::IntegerLiteral { .. } => self.parse_integer_literal()?,
            TokenType::FloatLiteral { .. } => self.parse_float_literal()?,
            TokenType::BooleanLiteral { .. } => self.parse_boolean_literal()?,

            TokenType::Keyword { .. } => self.parse_keyword()?,
            TokenType::BuiltIn { .. } => self.parse_builtin()?,

            TokenType::Identifier { .. } => self.parse_identifier()?,

            TokenType::OpenBlock => self.parse_block()?,
            TokenType::OpenParen => self.parse_parentheses()?,
            TokenType::OpenBracket => self.parse_vector()?,

            TokenType::UnaryOperator { .. } => self.parse_unary_operator()?,
            TokenType::BinaryOperator { value } => match value.as_str() {
                "-" => self.parse_unary_operator()?,
                _ => {
                    self.tokens.advance_to_next_instruction();
                    Err(ParseError::new(
                        ParseErrorType::UnexpectedToken(token.r#type.clone()),
                        token.clone(),
                    ))?
                }
            },

            TokenType::Semicolon => Instruction::new(InstructionType::None, token.clone()),
            token_type => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::UnexpectedToken(token_type.clone()),
                    token.clone(),
                ));
            }
        };

        token = self.peek_next_token()?;
        while token.binary_operator() {
            instruction = match token.r#type {
                TokenType::BinaryOperator { .. } => match parse_binary {
                    true => self.parse_binary_operator(instruction)?,
                    false => break,
                },
                TokenType::TypeCast => match parse_type_cast {
                    true => self.parse_type_cast(&instruction)?,
                    false => break,
                },
                _ => unreachable!(),
            };
            token = self.peek_next_token()?;
        }

        Ok(instruction)
    }

    fn parse_test(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let name = match &token.r#type {
            TokenType::Identifier { value } => value,
            _ => unreachable!(),
        };
        self.expect_token(TokenType::OpenParen)?;
        self.in_constant_declaration = true;
        let path = self.parse_string_literal()?;
        let path = match path.r#type {
            InstructionType::StringLiteral(path) => path,
            _ => unreachable!(),
        };
        self.in_constant_declaration = false;
        self.expect_token(TokenType::CloseParen)?;
        let instruction = self.parse_statement()?;

        Ok(Instruction::new(
            InstructionType::Test(Box::new(instruction), name.to_string(), path),
            token,
        ))
    }

    fn parse_function(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let name = self.get_next_token()?;
        let name = match &name.r#type {
            TokenType::Identifier { value } => value,
            r#type => Err(ParseError::new(
                ParseErrorType::MismatchedTokenType {
                    expected: TokenType::Identifier {
                        value: String::new(),
                    },
                    actual: r#type.clone(),
                },
                name.clone(),
            ))?,
        };

        self.expect_token(TokenType::OpenParen)?;
        let parameters = self.parse_parameters()?;
        self.expect_token(TokenType::CloseParen)?;
        self.expect_token(TokenType::Colon)?;
        let return_type = match &self.get_next_token()? {
            Token {
                r#type: TokenType::Type { value },
                ..
            } => value.clone(),
            return_type => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::MismatchedTokenType {
                        expected: TokenType::Type { value: Type::Any },
                        actual: return_type.r#type.clone(),
                    },
                    return_type.clone(),
                ));
            }
        };
        let function = Instruction::new(
            InstructionType::Function {
                name: name.to_string(),
                parameters: parameters.clone(),
                instruction: Box::new(Instruction::NONE),
                return_type: return_type.clone(),
            },
            token.clone(),
        );
        self.environment.add_function(Box::new(function.clone()));
        self.environment.add_scope();
        for parameter in parameters.iter() {
            self.environment.insert(parameter.clone());
        }
        let instruction = self.parse_statement()?;
        self.environment.remove_scope();
        let function = Instruction::new(
            InstructionType::Function {
                name: name.to_string(),
                parameters,
                instruction: Box::new(instruction),
                return_type,
            },
            token.clone(),
        );
        self.environment.add_function(Box::new(function.clone()));
        Ok(function)
    }

    fn parse_parameters(&mut self) -> Result<Vec<Variable>, ParseError> {
        let mut arguments = Vec::new();
        let mut r#const = false;
        while let Some(token) = self.tokens.peek() {
            match token.r#type {
                TokenType::CloseParen => {
                    break;
                }
                TokenType::Keyword { ref value } => {
                    if value != "const" {
                        self.tokens.advance_to_next_instruction();
                        return Err(ParseError::new(
                            ParseErrorType::MismatchedTokenType {
                                expected: TokenType::Identifier {
                                    value: String::new(),
                                },
                                actual: token.r#type.clone(),
                            },
                            token.clone(),
                        ));
                    }
                    r#const = true;
                }
                TokenType::Identifier { .. } => {
                    arguments.push(self.parse_parameter(r#const)?);
                    match self.peek_next_token()?.r#type {
                        TokenType::Comma => {
                            self.get_next_token()?;
                            continue;
                        }
                        TokenType::CloseParen => {
                            break;
                        }
                        _ => {
                            self.tokens.advance_to_next_instruction();
                            return Err(ParseError::new(
                                ParseErrorType::MismatchedTokenType {
                                    expected: TokenType::Comma,
                                    actual: self.peek_next_token()?.r#type,
                                },
                                self.peek_next_token()?,
                            ));
                        }
                    }
                }
                _ => {
                    self.tokens.advance_to_next_instruction();
                    return Err(ParseError::new(
                        ParseErrorType::UnexpectedToken(token.r#type.clone()),
                        token.clone(),
                    ));
                }
            }
        }
        Ok(arguments)
    }

    fn parse_parameter(&mut self, r#const: bool) -> Result<Variable, ParseError> {
        let token = self.get_next_token()?;
        let name = match &token.r#type {
            TokenType::Identifier { value } => value,
            _ => Err(ParseError::new(
                ParseErrorType::MismatchedTokenType {
                    expected: TokenType::Identifier {
                        value: String::new(),
                    },
                    actual: token.r#type.clone(),
                },
                token.clone(),
            ))?,
        };

        self.expect_token(TokenType::Colon)?;

        let r#type = match &self.get_next_token()? {
            Token {
                r#type: TokenType::Type { value },
                ..
            } => value.clone(),
            r#type => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::MismatchedTokenType {
                        expected: TokenType::Type { value: Type::Any },
                        actual: r#type.r#type.clone(),
                    },
                    r#type.clone(),
                ));
            }
        };

        Ok(Variable {
            name: name.to_string(),
            r#const,
            r#type,
            declaration_token: token.clone(),
            identifier_token: token.clone(),
            last_assignment_token: token.clone(),
            read: true,
            assigned: true,
        })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Instruction>, ParseError> {
        let mut arguments = Vec::new();
        while let Some(token) = self.tokens.peek() {
            match token.r#type {
                TokenType::CloseParen => {
                    break;
                }
                _ => {
                    arguments.push(self.parse_expression(true, true)?);
                    match self.peek_next_token()?.r#type {
                        TokenType::Comma => {
                            self.get_next_token()?;
                            continue;
                        }
                        TokenType::CloseParen => {
                            break;
                        }
                        _ => {
                            self.tokens.advance_to_next_instruction();
                            return Err(ParseError::new(
                                ParseErrorType::MismatchedTokenType {
                                    expected: TokenType::Comma,
                                    actual: self.peek_next_token()?.r#type,
                                },
                                self.peek_next_token()?,
                            ));
                        }
                    }
                }
            }
        }
        Ok(arguments)
    }

    fn parse_string_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;

        match &token.r#type {
            TokenType::StringLiteral { value } => {
                let value = value[1..value.len() - 1].to_string();
                if !self.args.disable_magic_warnings
                    && !self.in_constant_declaration
                    && !white_listed_constants::STRINGS.contains(&value.as_str())
                    && !self.args.disable_style_warnings
                {
                    ParseWarning::new(ParseWarningType::MagicLiteral(Type::String), token.clone())
                        .print(self.args.disable_warnings)
                }

                Ok(Instruction::new(
                    InstructionType::StringLiteral(value),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_unary_operator(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let operator = match &token.r#type {
            TokenType::UnaryOperator { value } => match value.as_str() {
                "!" => UnaryOperator::Not,
                "-" => UnaryOperator::Negation,
                _ => unreachable!(),
            },
            TokenType::BinaryOperator { value } => match value.as_str() {
                "-" => UnaryOperator::Negation,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        let instruction = self.parse_expression(false, false)?;
        Ok(Instruction::new(
            InstructionType::UnaryOperation {
                operator,
                instruction: Box::new(instruction),
            },
            token,
        ))
    }

    fn parse_binary_operator(
        &mut self,
        instruction: Instruction,
    ) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let new_operator = match &token.r#type {
            TokenType::BinaryOperator { value } => match value.as_str() {
                "+" => BinaryOperator::Addition,
                "-" => BinaryOperator::Subtraction,
                "*" => BinaryOperator::Multiplication,
                "/" => BinaryOperator::Division,
                "%" => BinaryOperator::Modulo,
                "==" => BinaryOperator::Equal,
                "!=" => BinaryOperator::NotEqual,
                ">" => BinaryOperator::GreaterThan,
                ">=" => BinaryOperator::GreaterThanOrEqual,
                "<" => BinaryOperator::LessThan,
                "<=" => BinaryOperator::LessThanOrEqual,
                "&&" => BinaryOperator::And,
                "||" => BinaryOperator::Or,
                "in" => BinaryOperator::In,
                "=" => {
                    self.tokens.back();
                    return self.parse_assignment(&instruction);
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        let new_right = self.parse_expression(false, true)?;
        if let Instruction {
            r#type: InstructionType::None,
            ..
        } = new_right
        {
            return Err(ParseError::new(
                ParseErrorType::UnexpectedToken(TokenType::Semicolon),
                token.clone(),
            ));
        }
        match instruction.r#type {
            InstructionType::BinaryOperation {
                ref operator,
                ref left,
                ref right,
            } => Ok(Instruction::new(
                if new_operator.cmp(operator) != std::cmp::Ordering::Greater {
                    InstructionType::BinaryOperation {
                        operator: new_operator,
                        left: Box::new(instruction.clone()),
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
                    left: Box::new(instruction.clone()),
                    right: Box::new(new_right),
                },
                token,
            )),
        }
    }

    fn parse_type_cast(&mut self, instruction: &Instruction) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let r#type = match self.get_next_token()? {
            Token {
                r#type: TokenType::Type { value },
                ..
            } => value,
            _ => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::MismatchedTokenType {
                        expected: TokenType::Type { value: Type::Any },
                        actual: token.clone().r#type,
                    },
                    token.clone(),
                ));
            }
        };
        Ok(Instruction::new(
            InstructionType::TypeCast {
                instruction: Box::new(instruction.clone()),
                r#type,
            },
            token,
        ))
    }

    fn parse_regex_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        match &token.r#type {
            TokenType::RegexLiteral { value } => {
                if !self.args.disable_magic_warnings
                    && !self.in_constant_declaration
                    && !white_listed_constants::STRINGS.contains(&value.to_string().as_str())
                    && !self.args.disable_style_warnings
                {
                    ParseWarning::new(ParseWarningType::MagicLiteral(Type::Regex), token.clone())
                        .print(self.args.disable_warnings)
                }
                Ok(Instruction::new(
                    InstructionType::RegexLiteral(regex::parse(&token, self.args.max_size)?),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_integer_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        match token.r#type {
            TokenType::IntegerLiteral { value } => {
                if !self.args.disable_magic_warnings
                    && !self.in_constant_declaration
                    && !white_listed_constants::INTEGERS.contains(&value)
                    && !self.args.disable_style_warnings
                {
                    ParseWarning::new(ParseWarningType::MagicLiteral(Type::Int), token.clone())
                        .print(self.args.disable_warnings)
                }
                Ok(Instruction::new(
                    InstructionType::IntegerLiteral(value),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_float_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        match token.r#type {
            TokenType::FloatLiteral { value } => {
                if !self.args.disable_magic_warnings
                    && !self.in_constant_declaration
                    && !white_listed_constants::FLOATS.contains(&value)
                    && !self.args.disable_style_warnings
                {
                    ParseWarning::new(ParseWarningType::MagicLiteral(Type::Float), token.clone())
                        .print(self.args.disable_warnings)
                }
                Ok(Instruction::new(
                    InstructionType::FloatLiteral(value),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_boolean_literal(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        match token.r#type {
            TokenType::BooleanLiteral { value } => {
                if !self.args.disable_magic_warnings
                    && !self.in_constant_declaration
                    && !white_listed_constants::BOOLS.contains(&value)
                    && !self.args.disable_style_warnings
                {
                    ParseWarning::new(ParseWarningType::MagicLiteral(Type::Bool), token.clone())
                        .print(self.args.disable_warnings)
                }
                Ok(Instruction::new(
                    InstructionType::BooleanLiteral(value),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_keyword(&mut self) -> Result<Instruction, ParseError> {
        let token = self.peek_next_token()?;
        match &token.r#type {
            TokenType::Keyword { value } => match value.as_str() {
                "let" => self.parse_declaration(),
                "const" => self.parse_declaration(),
                "for" => self.parse_for(),
                "if" => self.parse_conditional(),
                _ => {
                    self.tokens.advance_to_next_instruction();
                    Err(ParseError::new(
                        ParseErrorType::UnexpectedToken(token.r#type.clone()),
                        token.clone(),
                    ))
                }
            },
            _ => unreachable!(),
        }
    }

    fn parse_declaration(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let r#const = match &token.r#type {
            TokenType::Keyword { value } => value == "const",
            _ => unreachable!(),
        };
        let identifier = self.get_next_token()?;

        let identifier_name = match &identifier.r#type {
            TokenType::Identifier { value } => {
                match r#const {
                    false => {
                        if !self.args.disable_style_warnings && !value.is_snake_case() {
                            ParseWarning::new(
                                ParseWarningType::VariableNotSnakeCase(value.to_string()),
                                identifier.clone(),
                            )
                            .print(self.args.disable_warnings)
                        }
                    }
                    true => {
                        self.in_constant_declaration = true;
                        if !self.args.disable_style_warnings && !value.is_upper_snake_case() {
                            ParseWarning::new(
                                ParseWarningType::ConstantNotUpperCase(value.to_string()),
                                identifier.clone(),
                            )
                            .print(self.args.disable_warnings)
                        }
                    }
                }
                value.clone()
            }
            _ => {
                self.tokens.advance_to_next_instruction();
                self.in_constant_declaration = false;
                return Err(ParseError::new(
                    ParseErrorType::MismatchedTokenType {
                        expected: TokenType::Identifier {
                            value: String::new(),
                        },
                        actual: identifier.r#type.clone(),
                    },
                    identifier,
                ));
            }
        };

        match self.expect_token(TokenType::Colon) {
            Ok(_) => (),
            Err(_) => {
                let variable = Variable {
                    name: identifier_name.clone(),
                    r#const,
                    r#type: Type::Any,
                    declaration_token: token.clone(),
                    identifier_token: identifier.clone(),
                    last_assignment_token: token.clone(),
                    read: true,
                    assigned: true,
                };

                self.environment.insert(variable.clone());

                self.in_constant_declaration = false;
                return Err(ParseError::new(
                    ParseErrorType::VaribleTypeAnnotation,
                    identifier,
                ));
            }
        }

        let r#type = match &self.get_next_token()? {
            Token {
                r#type: TokenType::Type { value },
                ..
            } => value.clone(),
            Token {
                r#type: TokenType::OpenBracket,
                ..
            } => {
                if let Token {
                    r#type: TokenType::Type { value },
                    ..
                } = self.get_next_token()?
                {
                    self.expect_token(TokenType::CloseBracket)?;
                    Type::from(&format!("[{}]", value))
                } else {
                    self.tokens.advance_to_next_instruction();
                    self.in_constant_declaration = false;
                    return Err(ParseError::new(
                        ParseErrorType::MismatchedTokenType {
                            expected: TokenType::Type { value: Type::Any },
                            actual: self.get_next_token()?.r#type.clone(),
                        },
                        self.get_next_token()?,
                    ));
                }
            }

            r#type => {
                self.tokens.advance_to_next_instruction();
                self.in_constant_declaration = false;
                return Err(ParseError::new(
                    ParseErrorType::MismatchedTokenType {
                        expected: TokenType::Type { value: Type::Any },
                        actual: r#type.r#type.clone(),
                    },
                    r#type.clone(),
                ));
            }
        };

        let assignment = self.get_next_token()?;
        match &assignment.r#type {
            TokenType::BinaryOperator { value } => {
                if value != "=" && value != "in" {
                    self.tokens.advance_to_next_instruction();
                    self.in_constant_declaration = false;
                    return Err(ParseError::new(
                        ParseErrorType::MismatchedTokenType {
                            expected: TokenType::BinaryOperator {
                                value: "=".to_string(),
                            },
                            actual: assignment.r#type.clone(),
                        },
                        assignment,
                    ));
                }
            }
            _ => {
                self.tokens.advance_to_next_instruction();
                self.in_constant_declaration = false;
                return Err(ParseError::new(
                    ParseErrorType::MismatchedTokenType {
                        expected: TokenType::BinaryOperator {
                            value: "=".to_string(),
                        },
                        actual: assignment.r#type.clone(),
                    },
                    assignment,
                ));
            }
        }

        let variable = Variable {
            name: identifier_name.clone(),
            r#const,
            r#type: r#type.clone(),
            declaration_token: token.clone(),
            identifier_token: identifier.clone(),
            last_assignment_token: assignment.clone(),
            read: true,
            assigned: true,
        };

        let instruction = match self.parse_expression(true, true) {
            Ok(instruction) => instruction,
            Err(e) => {
                self.environment.insert(variable.clone());
                self.in_constant_declaration = false;
                return Err(e);
            }
        };
        self.in_constant_declaration = false;
        if let TokenType::BinaryOperator { value } = assignment.r#type {
            match value.as_str() {
                "=" => {
                    self.environment.insert(variable.clone());
                    Ok(Instruction::new(
                        InstructionType::Assignment {
                            variable,
                            instruction: Box::new(instruction),
                            token: identifier,
                            declaration: true,
                        },
                        token,
                    ))
                }
                "in" => {
                    self.environment.insert(variable.clone());
                    Ok(Instruction::new(
                        InstructionType::IterableAssignment {
                            variable,
                            instruction: Box::new(instruction),
                            token: token.clone(),
                        },
                        token,
                    ))
                }
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        }
    }

    fn parse_assignment(&mut self, instruction: &Instruction) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let variable = match &instruction.r#type {
            InstructionType::Variable(variable) => variable,
            _ => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::UnexpectedToken(token.r#type.clone()),
                    token.clone(),
                ));
            }
        };

        if variable.r#const {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::ConstantReassignment(variable.clone()),
                instruction.token.clone(),
            ));
        }

        if token.r#type
            != (TokenType::BinaryOperator {
                value: "=".to_string(),
            })
        {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::MismatchedTokenType {
                    expected: TokenType::BinaryOperator {
                        value: "=".to_string(),
                    },
                    actual: token.clone().r#type,
                },
                token,
            ));
        }

        let instruction = self.parse_expression(true, true)?;
        if self.environment.get(&variable.name).is_none() {
            self.tokens.advance_to_next_instruction();
            return Err(ParseError::new(
                ParseErrorType::IdentifierNotDefined(variable.name.clone()),
                token.clone(),
            ));
        }

        if let InstructionType::Variable(ref instruction_variable) = instruction.r#type {
            if variable.name == instruction_variable.name {
                ParseWarning::new(ParseWarningType::SelfAssignment, instruction.token.clone())
                    .print(self.args.disable_warnings);
            }
        }

        Ok(Instruction::new(
            InstructionType::Assignment {
                variable: variable.clone(),
                instruction: Box::new(instruction),
                token: token.clone(),
                declaration: false,
            },
            token,
        ))
    }

    fn parse_identifier(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        match &token.r#type {
            TokenType::Identifier { value } => {
                let variable = self.environment.get(value).cloned();
                let function = self.environment.get_function(value);
                if variable.is_none() && function.is_none() {
                    self.tokens.advance_to_next_instruction();
                    Err(ParseError::new(
                        ParseErrorType::IdentifierNotDefined(value.clone()),
                        token.clone(),
                    ))
                } else if function.is_some() {
                    self.expect_token(TokenType::OpenParen)?;
                    let arguments = self.parse_arguments()?;
                    self.expect_token(TokenType::CloseParen)?;
                    Ok(Instruction::new(
                        InstructionType::FunctionCall {
                            name: value.to_string(),
                            arguments,
                        },
                        token,
                    ))
                } else {
                    Ok(Instruction::new(
                        InstructionType::Variable(self.environment.get(value).unwrap().clone()),
                        token,
                    ))
                }
            }
            _ => unreachable!(),
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
                self.parse_expression(true, true)
            }
        }?;

        self.expect_token(TokenType::CloseParen)?;

        match &token.r#type {
            TokenType::BuiltIn { value } => match value.as_str() {
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
            },
            _ => unreachable!(),
        }
    }

    fn parse_block(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let mut block = Vec::new();
        self.environment.add_scope();
        self.in_constant_declaration = false;

        let mut next_token = self.peek_next_token()?;
        while next_token.r#type != TokenType::CloseBlock {
            match self.parse_statement() {
                Ok(instruction) => block.push(instruction),
                Err(e) => {
                    e.print();
                    self.success = false;
                }
            }
            next_token = match self.peek_next_token() {
                Ok(token) => token,
                Err(_) => {
                    self.tokens.advance_to_next_instruction();
                    return Err(ParseError::new(
                        ParseErrorType::UnclosedDelimiter(TokenType::OpenBlock),
                        token,
                    ));
                }
            }
        }

        self.environment.remove_scope();
        if block.is_empty() {
            ParseWarning::new(ParseWarningType::EmptyBlock, token.clone())
                .print(self.args.disable_warnings)
        }
        Ok(Instruction::new(InstructionType::Block(block), token))
    }

    fn parse_conditional(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let condition = self.parse_expression(true, true)?;
        let statement = self.parse_statement()?;
        match statement.r#type {
            InstructionType::Block(_) => (),
            InstructionType::None => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::UnexpectedToken(self.tokens.current().unwrap().r#type),
                    self.tokens.current().unwrap(),
                ));
            }
            _ => ParseWarning::new(
                ParseWarningType::NoBlock(&self.tokens.current().unwrap()),
                statement.token.clone(),
            )
            .print(self.args.disable_warnings || self.args.disable_style_warnings),
        }
        let r#else = match &self.peek_next_token()?.r#type {
            TokenType::Keyword { value } => match value.as_str() {
                "else" => {
                    self.get_next_token()?;
                    let statement = self.parse_statement()?;
                    self.tokens.back();
                    statement
                }
                _ => {
                    self.tokens.back();
                    Instruction::NONE
                }
            },
            _ => {
                self.tokens.back();
                Instruction::NONE
            }
        };

        match r#else.r#type {
            InstructionType::Block(_) => (),
            InstructionType::None => (),
            _ => ParseWarning::new(
                ParseWarningType::NoBlock(&self.tokens.peek().unwrap()),
                r#else.token.clone(),
            )
            .print(self.args.disable_warnings || self.args.disable_style_warnings),
        }

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

        let assignment = match self.parse_declaration() {
            Ok(instruction) => instruction,
            Err(e) => {
                e.print();
                self.success = false;
                Instruction::NONE
            }
        };

        let statement = self.parse_statement();

        self.environment.remove_scope();

        let statement = statement?;

        match statement.r#type {
            InstructionType::Block(_) => (),
            InstructionType::None => {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::UnexpectedToken(self.tokens.current().unwrap().r#type),
                    self.tokens.current().unwrap(),
                ));
            }
            _ => ParseWarning::new(
                ParseWarningType::NoBlock(&self.tokens.current().unwrap()),
                statement.token.clone(),
            )
            .print(self.args.disable_warnings || self.args.disable_style_warnings),
        }

        self.tokens.back();
        Ok(Instruction::new(
            InstructionType::For {
                assignment: Box::new(assignment),
                instruction: Box::new(statement),
            },
            token,
        ))
    }

    fn parse_parentheses(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let instruction = self.parse_expression(true, true)?;
        self.expect_token(TokenType::CloseParen)?;
        Ok(Instruction::new(
            InstructionType::Paren(Box::new(instruction)),
            token,
        ))
    }

    fn parse_vector(&mut self) -> Result<Instruction, ParseError> {
        let token = self.get_next_token()?;
        let mut vector = Vec::new();
        while !self.tokens.is_empty() {
            match self.parse_expression(true, true) {
                Ok(instruction) => vector.push(instruction),
                Err(e) => {
                    e.print();
                    self.success = false;
                }
            }
            if let Ok(token) = self.get_next_token() {
                if token.r#type == TokenType::Comma {
                    continue;
                } else if token.r#type == TokenType::CloseBracket {
                    break;
                } else {
                    self.tokens.advance_to_next_instruction();
                    return Err(ParseError::new(
                        ParseErrorType::MismatchedTokenType {
                            expected: TokenType::Comma,
                            actual: token.r#type.clone(),
                        },
                        token.clone(),
                    ));
                }
            } else {
                self.tokens.advance_to_next_instruction();
                return Err(ParseError::new(
                    ParseErrorType::UnexpectedEndOfFile,
                    self.tokens.current().unwrap(),
                ));
            }
        }
        Ok(Instruction::new(
            InstructionType::VectorLiteral(vector),
            token,
        ))
    }

    fn expect_token(&mut self, expected: TokenType) -> Result<(), ParseError> {
        let token = self.get_next_token()?;
        if token.r#type != expected {
            self.tokens.advance_to_next_instruction();
            Err(ParseError::new(
                ParseErrorType::MismatchedTokenType {
                    expected,
                    actual: token.clone().r#type,
                },
                token.clone(),
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
                    ParseErrorType::MismatchedTokenType {
                        expected: TokenType::Semicolon,
                        actual: token.clone().r#type,
                    },
                    token,
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
            ))
        }
    }
}
