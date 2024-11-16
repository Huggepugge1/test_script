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
            environment: ParseEnvironment::new(args.clone()),
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
            InstructionType::FloatLiteral(_) => Ok(Type::Float),
            InstructionType::BooleanLiteral(_) => Ok(Type::Bool),

            InstructionType::BuiltIn(instruction) => self.check_builtin(instruction),

            InstructionType::Block(instructions) => self.check_block(instructions),

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
                let mut variable = match self.environment.get(&variable.name) {
                    Some(v) => v.clone(),
                    None => variable.clone(),
                };
                variable.read = true;
                self.environment.insert(variable.clone());
                Ok(variable.r#type)
            }

            InstructionType::Assignment {
                variable,
                instruction,
                token,
            } => self.check_assignment(&variable, &instruction, token),

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
            } => self.check_binary(operator, left, right),

            InstructionType::TypeCast {
                instruction: left_instruction,
                r#type,
            } => self.check_type_cast(left_instruction, instruction, r#type),

            InstructionType::None => {
                ParseWarning::new(
                    ParseWarningType::TrailingSemicolon,
                    instruction.token.clone(),
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
                            expected: vec![Type::String],
                            actual: r#type,
                        },
                        instruction.token.clone(),
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
                            expected: vec![Type::String],
                            actual: r#type,
                        },
                        instruction.token.clone(),
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
                            expected: vec![Type::String],
                            actual: r#type,
                        },
                        instruction.token.clone(),
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
                            expected: vec![Type::String],
                            actual: r#type,
                        },
                        instruction.token.clone(),
                    ))
                }
            }
        }
    }

    fn check_block(&mut self, instructions: &Vec<Instruction>) -> Result<Type, ParseError> {
        self.environment.add_scope();
        if (instructions.len()) == 0 {
            return Ok(Type::None);
        }
        for instruction in &instructions[..instructions.len() - 1] {
            match self.check_instruction(&instruction) {
                Ok(t) => match t {
                    Type::None => (),
                    _ => {
                        ParseWarning::new(
                            ParseWarningType::UnusedValue,
                            instruction.inner_most().token.clone(),
                        )
                        .print(self.args.disable_warnings);
                    }
                },
                Err(e) => {
                    e.print();
                    self.success = false;
                }
            }
        }
        let result = self.check_instruction(&instructions[instructions.len() - 1])?;
        self.environment.remove_scope();
        Ok(result)
    }

    fn check_assignment(
        &mut self,
        variable: &Variable,
        instruction: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        let variable_type = variable.r#type;

        let instruction_type = self.check_instruction(&instruction.clone())?;

        if variable_type != Type::Any && variable_type != instruction_type {
            return Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![variable_type],
                    actual: instruction_type,
                },
                token.clone(),
            ));
        }

        let mut variable = match self.environment.get(&variable.name) {
            Some(v) => v.clone(),
            None => variable.clone(),
        };
        variable.read = false;
        variable.last_assignment_token = token.clone();

        self.environment.insert(variable);
        Ok(Type::None)
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
                    ParseErrorType::MismatchedType {
                        expected: vec![Type::Regex],
                        actual: variable_type,
                    },
                    instruction.token.clone(),
                )),
            },
            Ok(t) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Iterable],
                    actual: t,
                },
                instruction.token.clone(),
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
                        expected: vec![Type::Bool],
                        actual: t,
                    },
                    instruction.token.clone(),
                )),
            },
            UnaryOperator::Negation => match instruction_type {
                Type::Int => Ok(Type::Int),
                t => Err(ParseError::new(
                    ParseErrorType::MismatchedType {
                        expected: vec![Type::Int],
                        actual: t,
                    },
                    instruction.token.clone(),
                )),
            },
        }
    }

    fn check_binary(
        &mut self,
        operator: &BinaryOperator,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        match operator {
            BinaryOperator::Addition => self.check_addition(left, right),
            BinaryOperator::Subtraction => self.check_subtraction(left, right),
            BinaryOperator::Multiplication => self.check_multiplication(left, right),
            BinaryOperator::Division => self.check_division(left, right),
            BinaryOperator::Modulo => self.check_modulo(left, right),

            BinaryOperator::Equal => self.check_comparison(operator, left, right),
            BinaryOperator::NotEqual => self.check_comparison(operator, left, right),
            BinaryOperator::GreaterThan => self.check_comparison(operator, left, right),
            BinaryOperator::GreaterThanOrEqual => self.check_comparison(operator, left, right),
            BinaryOperator::LessThan => self.check_comparison(operator, left, right),
            BinaryOperator::LessThanOrEqual => self.check_comparison(operator, left, right),

            BinaryOperator::And => self.check_logical(left, right),
            BinaryOperator::Or => self.check_logical(left, right),
        }
    }

    fn check_addition(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::String, Type::String) => Ok(Type::String),
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::String, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::String],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::String, Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_subtraction(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (t1, _) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_multiplication(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::String, Type::Int) => Ok(Type::String),
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::String, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (t1, _) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::String, Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_division(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::Float, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Float],
                    actual: t2,
                },
                right.token.clone(),
            )),

            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_modulo(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_comparison(
        &mut self,
        operator: &BinaryOperator,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Int, Type::Int) => Ok(Type::Bool),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::Float, Type::Float) => Ok(Type::Bool),
            (Type::Float, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Float],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::String, Type::String) | (Type::Bool, Type::Bool) => match operator {
                BinaryOperator::Equal | BinaryOperator::NotEqual => Ok(Type::Bool),
                _ => Err(ParseError::new(
                    ParseErrorType::MismatchedType {
                        expected: vec![Type::Int],
                        actual: Type::Int,
                    },
                    left.token.clone(),
                )),
            },

            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int, Type::Float],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_logical(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Bool, Type::Bool) => Ok(Type::Bool),

            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Bool],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_type_cast(
        &mut self,
        left_instruction: &Instruction,
        instruction: &Instruction,
        r#type: &Type,
    ) -> Result<Type, ParseError> {
        let instruction_type = self.check_instruction(left_instruction)?;
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
                    expected: vec![Type::Bool],
                    actual: condition_type,
                },
                condition.token.clone(),
            ));
        }
        let result = self.check_instruction(&instruction)?;
        let result_else = if *r#else != Instruction::NONE {
            self.check_instruction(&r#else)?
        } else {
            Type::None
        };

        if result == Type::None || result == result_else {
            Ok(result)
        } else {
            Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![result],
                    actual: result_else,
                },
                r#else.inner_most().token.clone(),
            ))
        }
    }
}
