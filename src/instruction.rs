use crate::environment::Environment;
use crate::error::InterpreterError;
use crate::process::Process;
use crate::r#type::Type;
use crate::token::{Token, TokenType};
use crate::variable::Variable;

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionResult {
    String(String),
    Regex(Vec<String>),
    Int(i64),
    Float(f64),
    Bool(bool),
    None,
}

impl std::fmt::Display for InstructionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionResult::String(s) => write!(f, "{}", s),
            InstructionResult::Regex(s) => write!(f, "{:?}", s),
            InstructionResult::Int(i) => write!(f, "{}", i),
            InstructionResult::Float(i) => write!(f, "{}", i),
            InstructionResult::Bool(b) => write!(f, "{}", b),
            InstructionResult::None => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub enum BinaryOperator {
    And,
    Or,

    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,

    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinaryOperator::And => "&&",
                BinaryOperator::Or => "||",

                BinaryOperator::Equal => "==",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::GreaterThan => ">",
                BinaryOperator::GreaterThanOrEqual => ">=",
                BinaryOperator::LessThan => "<",
                BinaryOperator::LessThanOrEqual => "<=",

                BinaryOperator::Addition => "+",
                BinaryOperator::Subtraction => "-",
                BinaryOperator::Multiplication => "*",
                BinaryOperator::Division => "/",
                BinaryOperator::Modulo => "%",
            }
        )
    }
}

impl BinaryOperator {
    pub fn value(&self) -> Self {
        match self {
            BinaryOperator::Addition => Self::Addition,
            BinaryOperator::Subtraction => Self::Addition,
            BinaryOperator::Multiplication => Self::Multiplication,
            BinaryOperator::Division => Self::Multiplication,
            BinaryOperator::Modulo => Self::Multiplication,

            BinaryOperator::Equal => Self::Equal,
            BinaryOperator::NotEqual => Self::Equal,
            BinaryOperator::GreaterThan => Self::Equal,
            BinaryOperator::GreaterThanOrEqual => Self::Equal,
            BinaryOperator::LessThan => Self::Equal,
            BinaryOperator::LessThanOrEqual => Self::Equal,
            BinaryOperator::And => Self::And,
            BinaryOperator::Or => Self::And,
        }
    }
}

impl std::cmp::Ord for BinaryOperator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().partial_cmp(&other.value()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negation,
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOperator::Not => "!",
                UnaryOperator::Negation => "-",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltIn {
    Input(Box<Instruction>),
    Output(Box<Instruction>),
    Print(Box<Instruction>),
    Println(Box<Instruction>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub r#type: InstructionType,
    pub token: Token,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.r#type {
                InstructionType::StringLiteral(ref value) => value.clone(),
                InstructionType::RegexLiteral(ref value) => format!("{:?}", value),
                InstructionType::IntegerLiteral(ref value) => value.to_string(),
                InstructionType::FloatLiteral(ref value) => value.to_string(),
                InstructionType::BooleanLiteral(ref value) => value.to_string(),

                InstructionType::BuiltIn(ref built_in) => match built_in {
                    BuiltIn::Input(ref instruction) => format!("input({})", instruction),
                    BuiltIn::Output(ref instruction) => format!("output({})", instruction),
                    BuiltIn::Print(ref instruction) => format!("print({})", instruction),
                    BuiltIn::Println(ref instruction) => format!("println({})", instruction),
                },

                InstructionType::Block(ref instructions) => {
                    let mut result = String::new();
                    for instruction in instructions {
                        result.push_str(&format!("{}\n", instruction));
                    }
                    result
                }
                InstructionType::Paren(ref instruction) => format!("({})", instruction),

                InstructionType::Test(ref left, ref operator, ref right) => {
                    format!("{} {} {}", left, operator, right)
                }
                InstructionType::For {
                    ref assignment,
                    ref instruction,
                } => format!("for {} in {}", assignment, instruction),
                InstructionType::Conditional {
                    ref condition,
                    ref instruction,
                    ref r#else,
                } => format!(
                    "if {} {{\n{}\n}} else {{\n{}\n}}",
                    condition, instruction, r#else
                ),

                InstructionType::Assignment {
                    ref variable,
                    ref instruction,
                    ..
                } => format!("{} = {}", variable, instruction),
                InstructionType::IterableAssignment {
                    ref variable,
                    ref instruction,
                    ..
                } => {
                    format!("{} in {}", variable, instruction)
                }
                InstructionType::Variable(ref variable) => variable.to_string(),

                InstructionType::UnaryOperation {
                    ref operator,
                    ref instruction,
                } => format!("{}{}", operator, instruction),
                InstructionType::BinaryOperation {
                    ref operator,
                    ref left,
                    ref right,
                } => format!("{} {} {}", left, operator, right),

                InstructionType::TypeCast {
                    ref instruction,
                    ref r#type,
                } => format!("{} as {}", instruction, r#type),

                InstructionType::None => String::new(),
            }
        )
    }
}

impl Instruction {
    pub const NONE: Instruction = Instruction {
        r#type: InstructionType::None,
        token: Token {
            r#type: TokenType::None,
            file: String::new(),
            row: 0,
            column: 0,

            line: String::new(),
            last_token: None,
        },
    };

    pub fn new(r#type: InstructionType, token: Token) -> Self {
        Self { r#type, token }
    }

    pub fn inner_most(&self) -> &Self {
        match &self.r#type {
            InstructionType::Block(ref instructions) => {
                if instructions.is_empty() {
                    self
                } else {
                    instructions.last().unwrap().inner_most()
                }
            }
            InstructionType::Paren(ref instruction) => instruction.inner_most(),
            _ => self,
        }
    }

    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        Ok(match &self.r#type {
            InstructionType::StringLiteral(value) => InstructionResult::String(value.to_string()),
            InstructionType::RegexLiteral(value) => InstructionResult::Regex(value.to_vec()),
            InstructionType::IntegerLiteral(value) => InstructionResult::Int(*value),
            InstructionType::FloatLiteral(value) => InstructionResult::Float(*value),
            InstructionType::BooleanLiteral(value) => InstructionResult::Bool(*value),

            InstructionType::BuiltIn(_) => self.interpret_builtin(environment, process)?,

            InstructionType::Block(_) => self.interpret_block(environment, process)?,
            InstructionType::Paren(instruction) => instruction.interpret(environment, process)?,

            InstructionType::For { .. } => self.interpret_for(environment, process)?,
            InstructionType::Conditional { .. } => {
                self.interpret_conditional(environment, process)?
            }

            InstructionType::Assignment { .. } => {
                self.interpret_assignment(environment, process)?
            }
            InstructionType::Variable(..) => self.interpret_variable(environment, process)?,

            InstructionType::None => InstructionResult::None,

            InstructionType::UnaryOperation { .. } => {
                self.interpret_unary_operation(environment, process)?
            }
            InstructionType::BinaryOperation { .. } => {
                self.interpret_binary_operation(environment, process)?
            }

            InstructionType::TypeCast { .. } => self.interpret_typecast(environment, process)?,
            _ => {
                unreachable!();
            }
        })
    }

    fn interpret_builtin(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let builtin = match &self.r#type {
            InstructionType::BuiltIn(built_in) => built_in,
            _ => unreachable!(),
        };

        let value = match builtin {
            BuiltIn::Input(instruction) => instruction.interpret(environment, process)?,
            BuiltIn::Output(instruction) => instruction.interpret(environment, process)?,
            BuiltIn::Print(instruction) => instruction.interpret(environment, process)?,
            BuiltIn::Println(instruction) => instruction.interpret(environment, process)?,
        };

        let value = match value {
            InstructionResult::String(value) => value,
            _ => unreachable!(),
        };

        match process {
            Some(ref mut process) => match builtin {
                BuiltIn::Input(_) => match process.send(&value) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(e);
                    }
                },
                BuiltIn::Output(_) => match process.read_line(value) {
                    Ok(()) => (),
                    Err(e) => {
                        return Err(e);
                    }
                },
                BuiltIn::Print(_) => print!("{}", value),
                BuiltIn::Println(_) => println!("{}", value),
            },
            None => {
                return Err(InterpreterError::TestFailed(
                    "No process to send input to".to_string(),
                ));
            }
        };

        Ok(InstructionResult::None)
    }

    fn interpret_block(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        environment.add_scope();
        let instructions = match &self.r#type {
            InstructionType::Block(instructions) => instructions,
            _ => unreachable!(),
        };

        let mut result = InstructionResult::None;
        for instruction in instructions {
            result = match instruction.interpret(environment, process) {
                Ok(value) => value,
                Err(e) => {
                    environment.remove_scope();
                    return Err(e);
                }
            };
        }
        environment.remove_scope();
        Ok(result)
    }

    fn interpret_for(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let mut result = InstructionResult::None;
        let (assignment, instruction) = match &self.r#type {
            InstructionType::For {
                assignment,
                instruction,
            } => (assignment, instruction),
            _ => {
                unreachable!()
            }
        };
        environment.add_scope();
        let (assignment_var, assignment_values) = match &assignment.r#type {
            InstructionType::IterableAssignment { variable, .. } => (
                variable,
                match assignment.interpret(environment, process) {
                    Ok(value) => value,
                    Err(e) => {
                        environment.remove_scope();
                        return Err(e);
                    }
                },
            ),
            _ => {
                unreachable!()
            }
        };
        match assignment_values {
            InstructionResult::Regex(values) => {
                for value in values {
                    environment.insert(
                        assignment_var.name.clone(),
                        InstructionResult::String(value),
                    );
                    result = match instruction.interpret(environment, process) {
                        Ok(value) => value,
                        Err(e) => {
                            environment.remove_scope();
                            return Err(e);
                        }
                    };
                }
            }
            _ => {
                unreachable!()
            }
        }
        environment.remove_scope();
        Ok(result)
    }

    fn interpret_conditional(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (condition, instruction, r#else) = match &self.r#type {
            InstructionType::Conditional {
                condition,
                instruction,
                r#else,
            } => (condition, instruction, r#else),
            _ => {
                unreachable!()
            }
        };

        let condition = condition.interpret(environment, process)?;
        let condition = match condition {
            InstructionResult::Bool(value) => value,
            _ => {
                unreachable!()
            }
        };

        let result = if condition {
            instruction.interpret(environment, process)?
        } else {
            r#else.interpret(environment, process)?
        };
        Ok(result)
    }

    fn interpret_assignment(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (variable, instruction) = match &self.r#type {
            InstructionType::Assignment {
                variable,
                instruction,
                ..
            } => (variable, instruction),
            _ => {
                unreachable!()
            }
        };

        let value = instruction.interpret(environment, process)?;
        environment.insert(variable.name.clone(), value);
        Ok(InstructionResult::None)
    }

    fn interpret_variable(
        &self,
        environment: &mut Environment,
        _process: &Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let variable = match &self.r#type {
            InstructionType::Variable(variable) => variable,
            _ => {
                unreachable!()
            }
        };

        let value = environment.get(&variable.name).unwrap();
        Ok(value.clone())
    }

    fn interpret_unary_operation(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (operator, instruction) = match &self.r#type {
            InstructionType::UnaryOperation {
                operator,
                instruction,
            } => (operator, instruction),
            _ => {
                unreachable!()
            }
        };

        let value = instruction.interpret(environment, process)?;
        let value = match value {
            InstructionResult::Bool(value) => value,
            _ => {
                unreachable!()
            }
        };

        let result = match operator {
            UnaryOperator::Not => InstructionResult::Bool(!value),
            UnaryOperator::Negation => {
                unreachable!()
            }
        };
        Ok(result)
    }

    fn interpret_binary_operation(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let operator = match &self.r#type {
            InstructionType::BinaryOperation { operator, .. } => operator,
            _ => {
                unreachable!()
            }
        };

        Ok(match operator {
            BinaryOperator::Addition => self.interpret_addition(environment, process)?,
            BinaryOperator::Subtraction => self.interpret_subtraction(environment, process)?,
            BinaryOperator::Multiplication => {
                self.interpret_multiplication(environment, process)?
            }
            BinaryOperator::Division => self.interpret_division(environment, process)?,
            BinaryOperator::Modulo => self.interpret_modulo(environment, process)?,

            BinaryOperator::Equal => self.interpret_equal(environment, process)?,
            BinaryOperator::NotEqual => self.interpret_not_equal(environment, process)?,
            BinaryOperator::GreaterThan => self.interpret_greater_than(environment, process)?,
            BinaryOperator::GreaterThanOrEqual => {
                self.interpret_greater_than_or_equal(environment, process)?
            }
            BinaryOperator::LessThan => self.interpret_less_than(environment, process)?,
            BinaryOperator::LessThanOrEqual => {
                self.interpret_less_than_or_equal(environment, process)?
            }

            BinaryOperator::And => self.interpret_and(environment, process)?,
            BinaryOperator::Or => self.interpret_or(environment, process)?,
        })
    }

    fn interpret_addition(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::String(left), InstructionResult::String(right)) => {
                InstructionResult::String(format!("{}{}", left, right))
            }
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Int(left + right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Float(left + right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_subtraction(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Int(left - right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Float(left - right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_multiplication(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Int(left * right)
            }
            (InstructionResult::String(left), InstructionResult::Int(right)) => {
                InstructionResult::String(left.repeat(right as usize))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Float(left * right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_division(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Int(left / right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Float(left / right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_modulo(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Int(left % right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_equal(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::String(left), InstructionResult::String(right)) => {
                InstructionResult::Bool(left == right)
            }
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Bool(left == right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Bool(left == right)
            }
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                InstructionResult::Bool(left == right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_not_equal(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::String(left), InstructionResult::String(right)) => {
                InstructionResult::Bool(left != right)
            }
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Bool(left != right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Bool(left != right)
            }
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                InstructionResult::Bool(left != right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_greater_than(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Bool(left > right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Bool(left > right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_greater_than_or_equal(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Bool(left >= right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Bool(left >= right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_less_than(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Bool(left < right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Bool(left < right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_less_than_or_equal(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        Ok(match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Bool(left <= right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Bool(left <= right)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn interpret_and(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        let (left, right) = match (left, right) {
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => (left, right),
            _ => {
                unreachable!()
            }
        };
        Ok(InstructionResult::Bool(left && right))
    }

    fn interpret_or(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (left, right) = match &self.r#type {
            InstructionType::BinaryOperation { left, right, .. } => (
                left.interpret(environment, process)?,
                right.interpret(environment, process)?,
            ),
            _ => {
                unreachable!()
            }
        };
        let (left, right) = match (left, right) {
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => (left, right),
            _ => {
                unreachable!()
            }
        };
        Ok(InstructionResult::Bool(left || right))
    }

    fn interpret_typecast(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let (instruction, r#type) = match &self.r#type {
            InstructionType::TypeCast {
                instruction,
                r#type,
            } => (instruction, r#type),
            _ => {
                unreachable!()
            }
        };

        let value = instruction.interpret(environment, process)?;
        Ok(match r#type {
            Type::String => match value {
                InstructionResult::Int(value) => InstructionResult::String(value.to_string()),
                InstructionResult::Float(value) => InstructionResult::String(value.to_string()),
                InstructionResult::Bool(value) => InstructionResult::String(value.to_string()),
                _ => {
                    unreachable!()
                }
            },
            Type::Int => match value {
                InstructionResult::String(ref string_value) => {
                    InstructionResult::Int(match string_value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            return Err(InterpreterError::TypeCast {
                                result: value,
                                from: *r#type,
                                to: Type::Int,
                            });
                        }
                    })
                }
                InstructionResult::Float(value) => InstructionResult::Int(value as i64),
                _ => {
                    unreachable!()
                }
            },
            Type::Float => match value {
                InstructionResult::String(ref string_value) => {
                    InstructionResult::Float(match string_value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            return Err(InterpreterError::TypeCast {
                                result: value,
                                from: *r#type,
                                to: Type::Float,
                            });
                        }
                    })
                }
                InstructionResult::Int(value) => InstructionResult::Float(value as f64),
                _ => {
                    unreachable!()
                }
            },
            Type::Bool => match value {
                InstructionResult::String(ref string_value) => {
                    InstructionResult::Bool(match string_value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            return Err(InterpreterError::TypeCast {
                                result: value,
                                from: *r#type,
                                to: Type::Bool,
                            });
                        }
                    })
                }
                _ => {
                    unreachable!()
                }
            },
            _ => {
                unreachable!()
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    StringLiteral(String),
    RegexLiteral(Vec<String>),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    BooleanLiteral(bool),

    BuiltIn(BuiltIn),

    Block(Vec<Instruction>),
    Paren(Box<Instruction>),

    Test(Box<Instruction>, String, String),
    For {
        assignment: Box<Instruction>,
        instruction: Box<Instruction>,
    },
    Conditional {
        condition: Box<Instruction>,
        instruction: Box<Instruction>,
        r#else: Box<Instruction>,
    },

    Assignment {
        variable: Variable,
        instruction: Box<Instruction>,
        token: Token,
        declaration: bool,
    },
    IterableAssignment {
        variable: Variable,
        instruction: Box<Instruction>,
        token: Token,
    },
    Variable(Variable),

    UnaryOperation {
        operator: UnaryOperator,
        instruction: Box<Instruction>,
    },
    BinaryOperation {
        operator: BinaryOperator,
        left: Box<Instruction>,
        right: Box<Instruction>,
    },

    TypeCast {
        instruction: Box<Instruction>,
        r#type: Type,
    },

    None,
}
