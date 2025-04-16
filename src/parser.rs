use crate::{
    cli::Args,
    environment::ParserEnvironment,
    error::{ParserError, ParserErrorType, ParserWarning, ParserWarningType},
    instruction::{
        assignment::{iterable_assignment::IterableAssignment, Assignment},
        binary::{
            addition::Addition, and::And, division::Division, equal::Equal,
            greater_than::GreaterThan, greater_than_or_equal::GreaterThanOrEquality,
            less_than::LessThan, less_than_or_equal::LessThanOrEqual, modulo::Modulo,
            multiplication::Multiplication, not_equal::NotEqual, or::Or, subtraction::Subtraction,
            Binary, BinaryOperationTrait,
        },
        block::Block,
        boolean::BooleanLiteral,
        builtin::BuiltIn,
        conditional::Conditional,
        float::FloatLiteral,
        function::Function,
        function_call::FunctionCall,
        integer::IntegerLiteral,
        paren::Paren,
        r#for::For,
        regex::RegexLiteral,
        string::StringLiteral,
        test::TestInstruction,
        type_cast::TypeCast,
        unary::{Unary, UnaryOperator},
        variable::{SnakeCase, Variable},
        Instruction, InstructionType,
    },
    r#type::Type,
    regex,
    token::{Token, TokenCollection, TokenType},
    white_listed_constants,
};

pub struct Parser {
    tokens: TokenCollection,
    environment: ParserEnvironment,
    args: Args,
    in_constant_declaration: bool,
    success: bool,
}

impl Parser {
    pub fn new(tokens: TokenCollection, args: Args) -> Self {
        Self {
            tokens,
            environment: ParserEnvironment::new(args.clone()),
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
                        Err(ParserError::new(
                            ParserErrorType::GlobalScope(token.clone().r#type),
                            token,
                        ))
                    }
                },
                TokenType::OpenBlock | TokenType::CloseBlock => {
                    self.tokens.next();
                    Err(ParserError::new(
                        ParserErrorType::GlobalScope(token.clone().r#type),
                        token.clone(),
                    ))
                }

                r#type => {
                    self.tokens.advance_to_next_instruction();
                    Err(ParserError::new(
                        ParserErrorType::GlobalScope(r#type),
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

    fn parse_statement(&mut self) -> Result<Instruction, ParserError> {
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
    ) -> Result<Instruction, ParserError> {
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

            TokenType::UnaryOperator { .. } => self.parse_unary_operator()?,
            TokenType::BinaryOperator { value } => match value.as_str() {
                "-" => self.parse_unary_operator()?,
                _ => {
                    self.tokens.advance_to_next_instruction();
                    Err(ParserError::new(
                        ParserErrorType::UnexpectedToken(token.r#type.clone()),
                        token.clone(),
                    ))?
                }
            },

            TokenType::Semicolon => Instruction::new(InstructionType::None, token.clone()),
            token_type => {
                self.tokens.advance_to_next_instruction();
                return Err(ParserError::new(
                    ParserErrorType::UnexpectedToken(token_type.clone()),
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
                TokenType::AssignmentOperator => self.parse_assignment(&instruction)?,
                _ => unreachable!(),
            };
            token = self.peek_next_token()?;
        }

        Ok(instruction)
    }

    fn parse_test(&mut self) -> Result<Instruction, ParserError> {
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
            InstructionType::Test(TestInstruction {
                name: name.to_string(),
                command: path.to_string(),
                body: Box::new(instruction),
            }),
            token,
        ))
    }

    fn parse_function(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        let name = self.get_next_token()?;
        let name = match &name.r#type {
            TokenType::Identifier { value } => value,
            r#type => Err(ParserError::new(
                ParserErrorType::MismatchedTokenType {
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

        let return_type_token = self.get_next_token()?;
        let return_type = match &return_type_token {
            Token {
                r#type: TokenType::Type { value },
                ..
            } => value,
            return_type => {
                self.tokens.advance_to_next_instruction();
                return Err(ParserError::new(
                    ParserErrorType::MismatchedTokenType {
                        expected: TokenType::Type { value: Type::Any },
                        actual: return_type.r#type.clone(),
                    },
                    return_type.clone(),
                ));
            }
        };
        let mut function = Function {
            name: name.to_string(),
            parameters: parameters.clone(),
            body: Box::new(Instruction::NONE),
            return_type: return_type.clone(),
        };
        self.environment.add_function(function.clone());
        self.environment.add_scope();
        for parameter in parameters.iter() {
            self.environment.insert(parameter.clone());
        }
        let body = self.parse_statement()?;
        self.environment.remove_scope();
        function.body = Box::new(body);
        self.environment.add_function(function.clone());
        Ok(Instruction::new(InstructionType::Function(function), token))
    }

    fn parse_parameters(&mut self) -> Result<Vec<Variable>, ParserError> {
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
                        return Err(ParserError::new(
                            ParserErrorType::MismatchedTokenType {
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
                            return Err(ParserError::new(
                                ParserErrorType::MismatchedTokenType {
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
                    return Err(ParserError::new(
                        ParserErrorType::UnexpectedToken(token.r#type.clone()),
                        token.clone(),
                    ));
                }
            }
        }
        Ok(arguments)
    }

    fn parse_parameter(&mut self, r#const: bool) -> Result<Variable, ParserError> {
        let token = self.get_next_token()?;
        let name = match &token.r#type {
            TokenType::Identifier { value } => value,
            _ => Err(ParserError::new(
                ParserErrorType::MismatchedTokenType {
                    expected: TokenType::Identifier {
                        value: String::new(),
                    },
                    actual: token.r#type.clone(),
                },
                token.clone(),
            ))?,
        };

        self.expect_token(TokenType::Colon)?;

        let type_token = self.get_next_token()?;
        let r#type = match &type_token {
            Token {
                r#type: TokenType::Type { value },
                ..
            } => value.clone(),
            token => {
                self.tokens.advance_to_next_instruction();
                return Err(ParserError::new(
                    ParserErrorType::MismatchedTokenType {
                        expected: TokenType::Type { value: Type::Any },
                        actual: token.r#type.clone(),
                    },
                    token.clone(),
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
            type_token: type_token.clone(),

            read: true,
            assigned: true,
        })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Instruction>, ParserError> {
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
                            return Err(ParserError::new(
                                ParserErrorType::MismatchedTokenType {
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

    fn parse_string_literal(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;

        match &token.r#type {
            TokenType::StringLiteral { value } => {
                let value = value[1..value.len() - 1].to_string();

                Ok(Instruction::new(
                    InstructionType::StringLiteral(StringLiteral { value }),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_unary_operator(&mut self) -> Result<Instruction, ParserError> {
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
            InstructionType::UnaryOperation(Unary {
                operator,
                body: Box::new(instruction),
            }),
            token,
        ))
    }

    fn parse_binary_operator(
        &mut self,
        instruction: Instruction,
    ) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        let new_operator: Box<dyn BinaryOperationTrait> = match &token.r#type {
            TokenType::BinaryOperator { value } => match value.as_str() {
                "&&" => Box::new(And) as Box<dyn BinaryOperationTrait>,
                "||" => Box::new(Or) as Box<dyn BinaryOperationTrait>,

                "==" => Box::new(Equal) as Box<dyn BinaryOperationTrait>,
                "!=" => Box::new(NotEqual) as Box<dyn BinaryOperationTrait>,
                ">" => Box::new(GreaterThan) as Box<dyn BinaryOperationTrait>,
                ">=" => Box::new(GreaterThanOrEquality) as Box<dyn BinaryOperationTrait>,
                "<" => Box::new(LessThan) as Box<dyn BinaryOperationTrait>,
                "<=" => Box::new(LessThanOrEqual) as Box<dyn BinaryOperationTrait>,

                "+" => Box::new(Addition) as Box<dyn BinaryOperationTrait>,
                "-" => Box::new(Subtraction) as Box<dyn BinaryOperationTrait>,
                "*" => Box::new(Multiplication) as Box<dyn BinaryOperationTrait>,
                "/" => Box::new(Division) as Box<dyn BinaryOperationTrait>,
                "%" => Box::new(Modulo) as Box<dyn BinaryOperationTrait>,
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
            return Err(ParserError::new(
                ParserErrorType::UnexpectedToken(TokenType::Semicolon),
                token.clone(),
            ));
        }

        match instruction.r#type {
            InstructionType::BinaryOperation(Binary {
                ref operator,
                ref left,
                ref right,
            }) => Ok(Instruction::new(
                if &new_operator <= operator {
                    InstructionType::BinaryOperation(Binary {
                        operator: new_operator,
                        left: Box::new(instruction.clone()),
                        right: Box::new(new_right),
                    })
                } else {
                    InstructionType::BinaryOperation(Binary {
                        operator: operator.clone(),
                        left: left.clone(),
                        right: Box::new(Instruction::new(
                            InstructionType::BinaryOperation(Binary {
                                operator: new_operator,
                                left: right.clone(),
                                right: Box::new(new_right),
                            }),
                            token.clone(),
                        )),
                    })
                },
                token,
            )),
            _ => Ok(Instruction::new(
                InstructionType::BinaryOperation(Binary {
                    operator: new_operator,
                    left: Box::new(instruction.clone()),
                    right: Box::new(new_right),
                }),
                token,
            )),
        }
    }

    fn parse_type_cast(&mut self, instruction: &Instruction) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        let to = match self.get_next_token()? {
            Token {
                r#type: TokenType::Type { value },
                ..
            } => value,
            _ => {
                self.tokens.advance_to_next_instruction();
                return Err(ParserError::new(
                    ParserErrorType::MismatchedTokenType {
                        expected: TokenType::Type { value: Type::Any },
                        actual: token.clone().r#type,
                    },
                    token.clone(),
                ));
            }
        };
        Ok(Instruction::new(
            InstructionType::TypeCast(TypeCast {
                from: Box::new(instruction.clone()),
                to,
            }),
            token,
        ))
    }

    fn parse_regex_literal(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        match &token.r#type {
            TokenType::RegexLiteral { .. } => Ok(Instruction::new(
                InstructionType::RegexLiteral(RegexLiteral {
                    value: regex::parse(&token, self.args.max_size)?,
                }),
                token,
            )),
            _ => unreachable!(),
        }
    }

    fn parse_integer_literal(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        match token.r#type {
            TokenType::IntegerLiteral { value } => {
                if !self.args.disable_magic_warnings
                    && !self.in_constant_declaration
                    && !white_listed_constants::INTEGERS.contains(&value)
                    && !self.args.disable_style_warnings
                {
                    ParserWarning::new(ParserWarningType::MagicLiteral(Type::Int), token.clone())
                        .print(&self.args)
                }
                Ok(Instruction::new(
                    InstructionType::IntegerLiteral(IntegerLiteral { value }),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_float_literal(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        match token.r#type {
            TokenType::FloatLiteral { value } => {
                if !self.args.disable_magic_warnings
                    && !self.in_constant_declaration
                    && !white_listed_constants::FLOATS.contains(&value)
                    && !self.args.disable_style_warnings
                {
                    ParserWarning::new(ParserWarningType::MagicLiteral(Type::Float), token.clone())
                        .print(&self.args)
                }
                Ok(Instruction::new(
                    InstructionType::FloatLiteral(FloatLiteral { value }),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_boolean_literal(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        match token.r#type {
            TokenType::BooleanLiteral { value } => {
                if !self.args.disable_magic_warnings
                    && !self.in_constant_declaration
                    && !white_listed_constants::BOOLS.contains(&value)
                    && !self.args.disable_style_warnings
                {
                    ParserWarning::new(ParserWarningType::MagicLiteral(Type::Bool), token.clone())
                        .print(&self.args)
                }
                Ok(Instruction::new(
                    InstructionType::BooleanLiteral(BooleanLiteral { value }),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_keyword(&mut self) -> Result<Instruction, ParserError> {
        let token = self.peek_next_token()?;
        match &token.r#type {
            TokenType::Keyword { value } => match value.as_str() {
                "let" => self.parse_declaration(),
                "const" => self.parse_declaration(),
                "for" => self.parse_for(),
                "if" => self.parse_conditional(),
                _ => {
                    self.tokens.advance_to_next_instruction();
                    Err(ParserError::new(
                        ParserErrorType::UnexpectedToken(token.r#type.clone()),
                        token.clone(),
                    ))
                }
            },
            _ => unreachable!(),
        }
    }

    fn parse_declaration(&mut self) -> Result<Instruction, ParserError> {
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
                            ParserWarning::new(
                                ParserWarningType::VariableNotSnakeCase(value.to_string()),
                                identifier.clone(),
                            )
                            .print(&self.args)
                        }
                    }
                    true => {
                        self.in_constant_declaration = true;
                        if !self.args.disable_style_warnings && !value.is_upper_snake_case() {
                            ParserWarning::new(
                                ParserWarningType::ConstantNotUpperCase(value.to_string()),
                                identifier.clone(),
                            )
                            .print(&self.args)
                        }
                    }
                }
                value.clone()
            }
            _ => {
                self.tokens.advance_to_next_instruction();
                self.in_constant_declaration = false;
                return Err(ParserError::new(
                    ParserErrorType::MismatchedTokenType {
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
                    r#type: Type::Any,

                    declaration_token: token.clone(),
                    identifier_token: identifier.clone(),
                    last_assignment_token: token.clone(),
                    type_token: token.clone(),

                    r#const,
                    read: true,
                    assigned: true,
                };

                self.environment.insert(variable.clone());

                self.in_constant_declaration = false;
                return Err(ParserError::new(
                    ParserErrorType::VaribleTypeAnnotation,
                    identifier,
                ));
            }
        }

        let type_token = self.get_next_token()?;
        let r#type = match &type_token {
            Token {
                r#type: TokenType::Type { value },
                ..
            } => value.clone(),

            r#type => {
                self.tokens.advance_to_next_instruction();
                self.in_constant_declaration = false;
                return Err(ParserError::new(
                    ParserErrorType::MismatchedTokenType {
                        expected: TokenType::Type { value: Type::Any },
                        actual: r#type.r#type.clone(),
                    },
                    r#type.clone(),
                ));
            }
        };

        let assignment = self.get_next_token()?;
        match &assignment.r#type {
            TokenType::AssignmentOperator | TokenType::IterableAssignmentOperator => (),
            _ => {
                self.tokens.advance_to_next_instruction();
                self.in_constant_declaration = false;
                return Err(ParserError::new(
                    ParserErrorType::MismatchedTokenType {
                        expected: TokenType::AssignmentOperator,
                        actual: assignment.r#type.clone(),
                    },
                    assignment,
                ));
            }
        }

        let variable = Variable {
            name: identifier_name.clone(),
            r#type,

            declaration_token: token.clone(),
            identifier_token: identifier.clone(),
            last_assignment_token: assignment.clone(),
            type_token: type_token.clone(),

            r#const,
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
        match &assignment.r#type {
            TokenType::AssignmentOperator => {
                self.environment.insert(variable.clone());
                Ok(Instruction::new(
                    InstructionType::Assignment(Assignment {
                        variable,
                        body: Box::new(instruction),
                        token: identifier,
                        declaration: true,
                    }),
                    token,
                ))
            }
            TokenType::IterableAssignmentOperator => {
                self.environment.insert(variable.clone());
                Ok(Instruction::new(
                    InstructionType::IterableAssignment(IterableAssignment {
                        variable,
                        body: Box::new(instruction),
                    }),
                    token,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_assignment(&mut self, instruction: &Instruction) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        let variable = match &instruction.r#type {
            InstructionType::Variable(variable) => variable,
            _ => {
                self.tokens.advance_to_next_instruction();
                return Err(ParserError::new(
                    ParserErrorType::UnexpectedToken(token.r#type.clone()),
                    token.clone(),
                ));
            }
        };

        if variable.r#const {
            self.tokens.advance_to_next_instruction();
            return Err(ParserError::new(
                ParserErrorType::ConstantReassignment(variable.clone()),
                instruction.token.clone(),
            ));
        }

        if token.r#type != TokenType::AssignmentOperator {
            self.tokens.advance_to_next_instruction();
            return Err(ParserError::new(
                ParserErrorType::MismatchedTokenType {
                    expected: TokenType::AssignmentOperator,
                    actual: token.clone().r#type,
                },
                token,
            ));
        }

        let instruction = self.parse_expression(true, true)?;
        if self.environment.get(&variable.name).is_none() {
            self.tokens.advance_to_next_instruction();
            return Err(ParserError::new(
                ParserErrorType::IdentifierNotDefined(variable.name.clone()),
                token.clone(),
            ));
        }

        if let InstructionType::Variable(ref instruction_variable) = instruction.r#type {
            if variable.name == instruction_variable.name {
                ParserWarning::new(ParserWarningType::SelfAssignment, instruction.token.clone())
                    .print(&self.args);
            }
        }

        Ok(Instruction::new(
            InstructionType::Assignment(Assignment {
                variable: variable.clone(),
                body: Box::new(instruction),
                token: token.clone(),
                declaration: false,
            }),
            token,
        ))
    }

    fn parse_identifier(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        match &token.r#type {
            TokenType::Identifier { value } => {
                let variable = self.environment.get(value).cloned();
                let function = self.environment.get_function(value);
                if variable.is_none() && function.is_none() {
                    self.tokens.advance_to_next_instruction();
                    Err(ParserError::new(
                        ParserErrorType::IdentifierNotDefined(value.clone()),
                        token.clone(),
                    ))
                } else if function.is_some() {
                    self.expect_token(TokenType::OpenParen)?;
                    let arguments = self.parse_arguments()?;
                    self.expect_token(TokenType::CloseParen)?;
                    Ok(Instruction::new(
                        InstructionType::FunctionCall(FunctionCall {
                            name: value.to_string(),
                            arguments,
                        }),
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

    fn parse_builtin(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        self.expect_token(TokenType::OpenParen)?;
        let arguments = self.parse_arguments()?;
        self.expect_token(TokenType::CloseParen)?;

        match &token.r#type {
            TokenType::BuiltIn { name } => Ok(Instruction::new(
                InstructionType::BuiltIn(BuiltIn {
                    name: name.clone(),
                    arguments,
                }),
                token,
            )),
            _ => unreachable!(),
        }
    }

    fn parse_block(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        let mut statements = Vec::new();
        self.environment.add_scope();
        self.in_constant_declaration = false;

        let mut next_token = self.peek_next_token()?;
        while next_token.r#type != TokenType::CloseBlock {
            match self.parse_statement() {
                Ok(instruction) => statements.push(instruction),
                Err(e) => {
                    e.print();
                    self.success = false;
                }
            }
            next_token = match self.peek_next_token() {
                Ok(token) => token,
                Err(_) => {
                    self.tokens.advance_to_next_instruction();
                    return Err(ParserError::new(
                        ParserErrorType::UnclosedDelimiter(TokenType::OpenBlock),
                        token,
                    ));
                }
            }
        }

        self.environment.remove_scope();
        if statements.is_empty() {
            ParserWarning::new(ParserWarningType::EmptyBlock, token.clone()).print(&self.args)
        }
        Ok(Instruction::new(
            InstructionType::Block(Block { statements }),
            token,
        ))
    }

    fn parse_conditional(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        let condition = self.parse_expression(true, true)?;
        let r#if = self.parse_statement()?;
        match r#if.r#type {
            InstructionType::Block(_) => (),
            InstructionType::None => {
                self.tokens.advance_to_next_instruction();
                return Err(ParserError::new(
                    ParserErrorType::UnexpectedToken(self.tokens.current().unwrap().r#type),
                    self.tokens.current().unwrap(),
                ));
            }
            _ => ParserWarning::new(
                ParserWarningType::NoBlock(Box::new(self.tokens.current().unwrap())),
                r#if.token.clone(),
            )
            .print(&self.args),
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
            _ => ParserWarning::new(
                ParserWarningType::NoBlock(Box::new(self.tokens.peek().unwrap())),
                r#else.token.clone(),
            )
            .print(&self.args),
        }

        Ok(Instruction::new(
            InstructionType::Conditional(Conditional {
                condition: Box::new(condition),
                r#if: Box::new(r#if),
                r#else: Box::new(r#else),
            }),
            token,
        ))
    }

    fn parse_for(&mut self) -> Result<Instruction, ParserError> {
        let token = self.peek_next_token()?;

        self.environment.add_scope();

        let assignment = match self.parse_declaration() {
            Ok(instruction) => match instruction.r#type {
                InstructionType::IterableAssignment(iterable_assignment) => {
                    Some(iterable_assignment)
                }
                _ => {
                    self.tokens.advance_to_next_instruction();
                    return Err(ParserError::new(
                        ParserErrorType::MismatchedInstruction {
                            expected: vec![Instruction {
                                r#type: InstructionType::IterableAssignment(IterableAssignment {
                                    variable: Variable {
                                        name: String::new(),
                                        r#type: Type::Any,

                                        declaration_token: Token::none(),
                                        identifier_token: Token::none(),
                                        last_assignment_token: Token::none(),
                                        type_token: Token::none(),

                                        r#const: false,
                                        read: false,
                                        assigned: false,
                                    },
                                    body: Box::new(Instruction::NONE),
                                }),
                                token: token.clone(),
                            }],
                            actual: instruction,
                        },
                        token.clone(),
                    ));
                }
            },
            Err(e) => {
                e.print();
                self.success = false;
                None
            }
        };

        let statement = self.parse_statement();

        self.environment.remove_scope();

        let body = statement?;

        match body.r#type {
            InstructionType::Block(_) => (),
            InstructionType::None => {
                self.tokens.advance_to_next_instruction();
                return Err(ParserError::new(
                    ParserErrorType::UnexpectedToken(self.tokens.current().unwrap().r#type),
                    self.tokens.current().unwrap(),
                ));
            }
            _ => ParserWarning::new(
                ParserWarningType::NoBlock(Box::new(self.tokens.current().unwrap())),
                body.token.clone(),
            )
            .print(&self.args),
        }

        self.tokens.back();
        match assignment {
            Some(assignment) => Ok(Instruction::new(
                InstructionType::For(For {
                    assignment,
                    body: Box::new(body),
                }),
                token,
            )),
            None => Ok(Instruction::NONE),
        }
    }

    fn parse_parentheses(&mut self) -> Result<Instruction, ParserError> {
        let token = self.get_next_token()?;
        let instruction = self.parse_expression(true, true)?;
        self.expect_token(TokenType::CloseParen)?;
        Ok(Instruction::new(
            InstructionType::Paren(Paren {
                expression: Box::new(instruction),
            }),
            token,
        ))
    }

    fn expect_token(&mut self, expected: TokenType) -> Result<(), ParserError> {
        let token = self.get_next_token()?;
        if token.r#type != expected {
            self.tokens.advance_to_next_instruction();
            Err(ParserError::new(
                ParserErrorType::MismatchedTokenType {
                    expected,
                    actual: token.clone().r#type,
                },
                token.clone(),
            ))
        } else {
            Ok(())
        }
    }

    fn end_statement(&mut self) -> Result<(), ParserError> {
        let token = self.get_next_token()?;
        match token.r#type {
            TokenType::Semicolon | TokenType::CloseBlock => Ok(()),
            _ => {
                self.tokens.back();
                Err(ParserError::new(
                    ParserErrorType::MismatchedTokenType {
                        expected: TokenType::Semicolon,
                        actual: token.clone().r#type,
                    },
                    token,
                ))
            }
        }
    }

    fn get_next_token(&mut self) -> Result<Token, ParserError> {
        if let Some(token) = self.tokens.next() {
            Ok(token)
        } else {
            Err(ParserError::new(
                ParserErrorType::UnexpectedEndOfFile,
                self.tokens.current().unwrap(),
            ))
        }
    }

    fn peek_next_token(&mut self) -> Result<Token, ParserError> {
        if let Some(token) = self.tokens.peek() {
            Ok(token)
        } else {
            Err(ParserError::new(
                ParserErrorType::UnexpectedEndOfFile,
                self.tokens.current().unwrap(),
            ))
        }
    }
}
