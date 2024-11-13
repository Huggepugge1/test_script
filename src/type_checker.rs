use crate::cli::Args;
use crate::environment::ParseEnvironment;
use crate::error::{ParseError, ParseErrorType, ParseWarning, ParseWarningType};
use crate::instruction::{BinaryOperator, BuiltIn, Instruction, InstructionType, UnaryOperator};
use crate::r#type::Type;
use crate::token::Token;
use crate::variable::Variable;

pub struct TypeChecker {
    program: Vec<Instruction>,
    environment: ParseEnvironment,
    success: bool,
    args: Args,
}

impl TypeChecker {
    pub fn new(program: Vec<Instruction>, args: Args) -> Self {
        Self {
            program,
            environment: ParseEnvironment::new(),
            success: true,
            args,
        }
    }

    pub fn check(&mut self) -> Result<(), ParseError> {
        for instruction in self.program.clone() {
            match instruction.r#type {
                InstructionType::Test(instruction, _name, _command) => {
                    match self.check_instruction(&instruction) {
                        Ok(_) => (),
                        Err(e) => {
                            e.print();
                            self.success = false;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        match self.success {
            true => Ok(()),
            false => Err(ParseError::none()),
        }
    }

    fn check_instruction(&mut self, instruction: &Instruction) -> Result<Type, ParseError> {
        match &instruction.r#type {
            InstructionType::StringLiteral(_) => Ok(Type::String),
            InstructionType::RegexLiteral(_) => Ok(Type::Regex),
            InstructionType::IntegerLiteral(_) => Ok(Type::Int),
            InstructionType::BooleanLiteral(_) => Ok(Type::Bool),

            InstructionType::BuiltIn(instruction) => self.check_builtin(instruction),

            InstructionType::Block(instructions) => {
                let mut result = Type::None;
                self.environment.add_scope();
                for instruction in instructions {
                    result = self.check_instruction(&instruction)?;
                }
                self.environment.remove_scope();
                Ok(result)
            }

            InstructionType::Paren(instruction) => self.check_instruction(instruction),

            InstructionType::Conditional {
                condition,
                instruction,
                r#else,
            } => self.check_conditional(condition, instruction, r#else),

            InstructionType::For(assignment, statement) => {
                self.environment.add_scope();
                self.check_instruction(&assignment)?;
                let result = self.check_instruction(&statement)?;
                self.environment.remove_scope();
                Ok(result)
            }

            InstructionType::Variable(variable) => {
                self.environment.insert(variable.clone());
                Ok(variable.r#type)
            }

            InstructionType::Assignment {
                variable,
                instruction,
            } => self.check_assignment(&variable, &instruction),

            InstructionType::IterableAssignment(variable, instruction) => {
                self.check_iterable_assignment(&variable, &instruction)
            }

            InstructionType::UnaryOperation {
                operator,
                instruction,
            } => self.check_unary(operator, &instruction),
            InstructionType::BinaryOperation {
                operator,
                left,
                right,
            } => self.check_binary(operator, left, right, &instruction.token),

            InstructionType::TypeCast {
                instruction,
                r#type,
            } => self.check_type_cast(instruction, r#type),

            InstructionType::None => {
                ParseWarning::new(
                    ParseWarningType::TrailingSemicolon,
                    instruction.token.clone(),
                    "Remove the trailing semicolon",
                )
                .print(self.args.disable_warnings);
                Ok(Type::None)
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn check_builtin(&mut self, built_in: &BuiltIn) -> Result<Type, ParseError> {
        match built_in {
            BuiltIn::Input(instruction) => {
                let r#type = self.check_instruction(&instruction)?;
                if r#type == Type::String {
                    Ok(Type::None)
                } else {
                    Err(ParseError::new(
                        ParseErrorType::MismatchedType {
                            expected: Type::String,
                            actual: r#type,
                        },
                        instruction.token.clone(),
                        "Expected a string",
                    ))
                }
            }
            BuiltIn::Output(instruction) => {
                let r#type = self.check_instruction(&instruction)?;
                if r#type == Type::String {
                    Ok(Type::None)
                } else {
                    Err(ParseError::new(
                        ParseErrorType::MismatchedType {
                            expected: Type::String,
                            actual: r#type,
                        },
                        instruction.token.clone(),
                        "Expected a None",
                    ))
                }
            }
            BuiltIn::Print(instruction) => {
                let r#type = self.check_instruction(&instruction)?;
                if r#type == Type::String {
                    Ok(Type::None)
                } else {
                    Err(ParseError::new(
                        ParseErrorType::MismatchedType {
                            expected: Type::String,
                            actual: r#type,
                        },
                        instruction.token.clone(),
                        "Expected a string",
                    ))
                }
            }
            BuiltIn::Println(instruction) => {
                let r#type = self.check_instruction(&instruction)?;
                if r#type == Type::String {
                    Ok(Type::None)
                } else {
                    Err(ParseError::new(
                        ParseErrorType::MismatchedType {
                            expected: Type::String,
                            actual: r#type,
                        },
                        instruction.token.clone(),
                        "Expected a string",
                    ))
                }
            }
        }
    }

    fn check_assignment(
        &mut self,
        variable: &Variable,
        instruction: &Instruction,
    ) -> Result<Type, ParseError> {
        let variable_name = &variable.name;
        let variable_type = variable.r#type;

        let instruction_type = self.check_instruction(&instruction.clone())?;

        if variable_type != instruction_type {
            return Err(ParseError::new(
                ParseErrorType::MismatchedType{expected: variable_type, actual: instruction_type},
                instruction.token.clone(),
                format!(
                    "Expected expression of type {:?} because of \"{variable_name}\" type but found {:?}",
                    variable_type, instruction_type
                ),
            ));
        }

        self.environment.insert(variable.clone());
        Ok(variable_type)
    }

    fn check_iterable_assignment(
        &mut self,
        variable: &Variable,
        instruction: &Instruction,
    ) -> Result<Type, ParseError> {
        let variable_type = variable.r#type;
        match self.check_instruction(&instruction) {
            Ok(Type::Regex) => match variable_type {
                Type::String => {
                    self.environment.insert(variable.clone());
                    Ok(variable_type)
                }
                _ => Err(ParseError::new(
                ParseErrorType::MismatchedType{expected: Type::Regex, actual: variable_type},
                    instruction.token.clone(),
                    format!(
                        "Expected expression of type {:?} because of the variable type but found {:?}",
                        variable_type, instruction.r#type
                    ),
                )),
            },
            Ok(t) => Err(ParseError::new(
                ParseErrorType::MismatchedType{expected: Type::Iterable, actual: t},
                instruction.token.clone(),
                format!("Expected an iterable type but found a {t:?}"),
            )),
            Err(e) => Err(e),
        }
    }

    fn check_unary(
        &mut self,
        operator: &UnaryOperator,
        instruction: &Instruction,
    ) -> Result<Type, ParseError> {
        let instruction_type = self.check_instruction(instruction)?;
        match operator {
            UnaryOperator::Not => match instruction_type {
                Type::Bool => Ok(Type::Bool),
                t => Err(ParseError::new(
                    ParseErrorType::MismatchedType {
                        expected: Type::Bool,
                        actual: t,
                    },
                    instruction.token.clone(),
                    format!("Expected a boolean but found a {t:?}"),
                )),
            },
            UnaryOperator::Negation => match instruction_type {
                Type::Int => Ok(Type::Int),
                t => Err(ParseError::new(
                    ParseErrorType::MismatchedType {
                        expected: Type::Int,
                        actual: t,
                    },
                    instruction.token.clone(),
                    format!("Expected an integer but found a {t:?}"),
                )),
            },
        }
    }

    fn check_binary(
        &mut self,
        operator: &BinaryOperator,
        left: &Instruction,
        right: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        match operator {
            BinaryOperator::Addition => self.check_addition(left, right, token),
            BinaryOperator::Subtraction => self.check_subtraction(left, right, token),
            BinaryOperator::Multiplication => self.check_multiplication(left, right, token),
            BinaryOperator::Division => self.check_division(left, right, token),

            BinaryOperator::Equal => self.check_comparison(left, right, token),
            BinaryOperator::NotEqual => self.check_comparison(left, right, token),
            BinaryOperator::GreaterThan => self.check_comparison(left, right, token),
            BinaryOperator::GreaterThanOrEqual => self.check_comparison(left, right, token),
            BinaryOperator::LessThan => self.check_comparison(left, right, token),
            BinaryOperator::LessThanOrEqual => self.check_comparison(left, right, token),

            BinaryOperator::And => self.check_logical(left, right, token),
            BinaryOperator::Or => self.check_logical(left, right, token),
        }
    }

    fn check_addition(
        &mut self,
        left: &Instruction,
        right: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        let left = self.check_instruction(left)?;
        let right = self.check_instruction(right)?;

        match (left, right) {
            (Type::String, Type::String) => Ok(Type::String),
            (Type::Int, Type::Int) => Ok(Type::Int),
            (t1, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedTypeBinary {
                    expected_left: t1,
                    actual_left: t1,
                    expected_right: t1,
                    actual_right: t2,
                },
                token.clone(),
                format!("Addition is not supported between `{}` and `{}`", t1, t2),
            )),
        }
    }

    fn check_subtraction(
        &mut self,
        left: &Instruction,
        right: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        let left = self.check_instruction(left)?;
        let right = self.check_instruction(right)?;

        match (left, right) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (t1, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedTypeBinary {
                    expected_left: Type::Int,
                    actual_left: t1,
                    expected_right: Type::Int,
                    actual_right: t2,
                },
                token.clone(),
                format!("Subtraction is not supported between `{}` and `{}`", t1, t2),
            )),
        }
    }

    fn check_multiplication(
        &mut self,
        left: &Instruction,
        right: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        let left = self.check_instruction(left)?;
        let right = self.check_instruction(right)?;

        match (left, right) {
            (Type::String, Type::Int) => Ok(Type::String),
            (Type::Int, Type::Int) => Ok(Type::Int),

            (t1, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedTypeBinary {
                    expected_left: Type::Int,
                    actual_left: t1,
                    expected_right: Type::Int,
                    actual_right: t2,
                },
                token.clone(),
                format!(
                    "Multiplication is not supported between `{}` and `{}`",
                    t1, t2
                ),
            )),
        }
    }

    fn check_division(
        &mut self,
        left: &Instruction,
        right: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        let left = self.check_instruction(left)?;
        let right = self.check_instruction(right)?;

        match (left, right) {
            (Type::Int, Type::Int) => Ok(Type::Int),

            (t1, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedTypeBinary {
                    expected_left: Type::Int,
                    actual_left: t1,
                    expected_right: Type::Int,
                    actual_right: t2,
                },
                token.clone(),
                format!(
                    "Multiplication is not supported between `{}` and `{}`",
                    t1, t2
                ),
            )),
        }
    }

    fn check_comparison(
        &mut self,
        left: &Instruction,
        right: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        let left = self.check_instruction(left)?;
        let right = self.check_instruction(right)?;

        match (left, right) {
            (Type::Int, Type::Int) => Ok(Type::Bool),
            (Type::String, Type::String) | (Type::Bool, Type::Bool) => match token.value.as_str() {
                "==" | "!=" => Ok(Type::Bool),
                _ => Err(ParseError::new(
                    ParseErrorType::MismatchedTypeBinary {
                        expected_left: Type::Int,
                        actual_left: Type::Int,
                        expected_right: Type::Int,
                        actual_right: Type::Int,
                    },
                    token.clone(),
                    format!(
                        "Comparison is not supported between `{}` and `{}`",
                        left, right
                    ),
                )),
            },

            (t1, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedTypeBinary {
                    expected_left: t1,
                    actual_left: t1,
                    expected_right: t1,
                    actual_right: t2,
                },
                token.clone(),
                format!("Comparison is not supported between `{}` and `{}`", t1, t2),
            )),
        }
    }

    fn check_logical(
        &mut self,
        left: &Instruction,
        right: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        let left = self.check_instruction(left)?;
        let right = self.check_instruction(right)?;

        match (left, right) {
            (Type::Bool, Type::Bool) => Ok(Type::Bool),

            (t1, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedTypeBinary {
                    expected_left: Type::Bool,
                    actual_left: t1,
                    expected_right: Type::Bool,
                    actual_right: t2,
                },
                token.clone(),
                format!(
                    "Logical operation is not supported between `{}` and `{}`",
                    t1, t2
                ),
            )),
        }
    }

    fn check_type_cast(
        &mut self,
        instruction: &Instruction,
        r#type: &Type,
    ) -> Result<Type, ParseError> {
        let instruction_type = self.check_instruction(instruction)?;
        match (instruction_type, r#type) {
            (Type::String, Type::Int) => Ok(Type::Int),
            (Type::Int, Type::String) => Ok(Type::String),

            (Type::String, Type::Bool) => Ok(Type::Bool),
            (Type::Bool, Type::String) => Ok(Type::String),
            (Type::String, Type::Regex) => Ok(Type::Regex),
            _ => Err(ParseError::new(
                ParseErrorType::TypeCast {
                    from: instruction_type,
                    to: *r#type,
                },
                instruction.token.clone(),
                format!("Cannot cast {instruction_type} to {}", r#type),
            )),
        }
    }

    fn check_conditional(
        &mut self,
        condition: &Instruction,
        instruction: &Instruction,
        r#else: &Instruction,
    ) -> Result<Type, ParseError> {
        let condition_type = self.check_instruction(&condition)?;
        if condition_type != Type::Bool {
            return Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: Type::Bool,
                    actual: condition_type,
                },
                condition.token.clone(),
                "Only booleans are allowed in conditions",
            ));
        }
        let result = self.check_instruction(&instruction)?;
        let result_else = self.check_instruction(&r#else)?;
        if result == result_else {
            Ok(result)
        } else {
            Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: result,
                    actual: result_else,
                },
                instruction.token.clone(),
                "Conditional return two types that don't match",
            ))
        }
    }
}
