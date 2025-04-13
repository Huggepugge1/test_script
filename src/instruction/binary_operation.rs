use crate::{environment::Environment, error::InterpreterError, process::Process};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl std::cmp::PartialOrd for BinaryOperator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (self.value() as u8)
                .partial_cmp(&(other.value() as u8))
                .unwrap(),
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperation {
    pub operator: BinaryOperator,
    pub left: Box<Instruction>,
    pub right: Box<Instruction>,
}

impl std::fmt::Display for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.left, self.operator, self.right)
    }
}

impl BinaryOperation {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        match self.operator {
            BinaryOperator::Addition => self.interpret_addition(environment, process),
            BinaryOperator::Subtraction => self.interpret_subtraction(environment, process),
            BinaryOperator::Multiplication => self.interpret_multiplication(environment, process),
            BinaryOperator::Division => self.interpret_division(environment, process),
            BinaryOperator::Modulo => self.interpret_modulo(environment, process),

            BinaryOperator::Equal => self.interpret_equal(environment, process),
            BinaryOperator::NotEqual => self.interpret_not_equal(environment, process),
            BinaryOperator::LessThan => self.interpret_less_than(environment, process),
            BinaryOperator::LessThanOrEqual => {
                self.interpret_less_than_or_equal(environment, process)
            }
            BinaryOperator::GreaterThan => self.interpret_greater_than(environment, process),
            BinaryOperator::GreaterThanOrEqual => {
                self.interpret_greater_than_or_equal(environment, process)
            }

            BinaryOperator::And => self.interpret_and(environment, process),
            BinaryOperator::Or => self.interpret_or(environment, process),
        }
    }

    fn interpret_addition(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Int(left + right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left + right))
            }
            (InstructionResult::String(left), InstructionResult::String(right)) => {
                Ok(InstructionResult::String(format!("{}{}", left, right)))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_subtraction(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Int(left - right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left - right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_multiplication(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Int(left * right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left * right))
            }
            (InstructionResult::String(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::String(left.repeat(right as usize)))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_division(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Float(left as f64 / right as f64))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left / right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_modulo(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Int(left % right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left % right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_equal(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Bool(left == right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left == right))
            }
            (InstructionResult::String(left), InstructionResult::String(right)) => {
                Ok(InstructionResult::Bool(left == right))
            }
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                Ok(InstructionResult::Bool(left == right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_not_equal(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Bool(left != right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left != right))
            }
            (InstructionResult::String(left), InstructionResult::String(right)) => {
                Ok(InstructionResult::Bool(left != right))
            }
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                Ok(InstructionResult::Bool(left != right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_less_than(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Bool(left < right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left < right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_less_than_or_equal(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Bool(left <= right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left <= right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_greater_than(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Bool(left > right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left > right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_greater_than_or_equal(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                Ok(InstructionResult::Bool(left >= right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left >= right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_and(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                Ok(InstructionResult::Bool(left && right))
            }
            _ => unreachable!(),
        }
    }

    fn interpret_or(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left_result = self.left.interpret(environment, process)?;
        let right_result = self.right.interpret(environment, process)?;

        match (left_result, right_result) {
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                Ok(InstructionResult::Bool(left || right))
            }
            _ => unreachable!(),
        }
    }
}
