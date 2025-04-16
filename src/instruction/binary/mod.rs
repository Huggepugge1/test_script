pub mod addition;
pub mod and;
pub mod division;
pub mod equal;
pub mod greater_than;
pub mod greater_than_or_equal;
pub mod less_than;
pub mod less_than_or_equal;
pub mod modulo;
pub mod multiplication;
pub mod not_equal;
pub mod or;
pub mod subtraction;

use std::collections::HashMap;

use addition::Addition;
use and::And;
use division::Division;
use equal::Equal;
use greater_than::GreaterThan;
use greater_than_or_equal::GreaterThanOrEquality;
use less_than::LessThan;
use less_than_or_equal::LessThanOrEqual;
use modulo::Modulo;
use multiplication::Multiplication;
use not_equal::NotEqual;
use or::Or;
use subtraction::Subtraction;

use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserMessage},
    interpreter::Interpret,
    process::Process,
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
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

    pub fn to_u8(&self) -> u8 {
        match self {
            BinaryOperator::And => 0,
            BinaryOperator::Or => 1,

            BinaryOperator::Equal => 2,
            BinaryOperator::NotEqual => 3,
            BinaryOperator::GreaterThan => 4,
            BinaryOperator::GreaterThanOrEqual => 5,
            BinaryOperator::LessThan => 6,
            BinaryOperator::LessThanOrEqual => 7,

            BinaryOperator::Addition => 8,
            BinaryOperator::Subtraction => 9,
            BinaryOperator::Multiplication => 10,
            BinaryOperator::Division => 11,
            BinaryOperator::Modulo => 12,
        }
    }
}

impl std::cmp::PartialOrd for BinaryOperator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (self.value().to_u8())
                .partial_cmp(&(other.value().to_u8()))
                .unwrap(),
        )
    }
}

pub trait BinaryOperationTrait: std::fmt::Debug + std::fmt::Display {
    fn operator(&self) -> BinaryOperator;

    fn valid_types(&self) -> Vec<(Type, Type)>;
    fn resulting_types(&self) -> HashMap<(Type, Type), Type>;

    fn operate(&self, left: &InstructionResult, right: &InstructionResult) -> InstructionResult;

    fn to_u8(&self) -> u8 {
        self.operator().to_u8()
    }

    fn priority(&self) -> u8;
}

impl PartialEq for Box<dyn BinaryOperationTrait> {
    fn eq(&self, other: &Self) -> bool {
        self.priority() == other.priority()
    }
}

impl PartialOrd for Box<dyn BinaryOperationTrait> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (self.to_u8())
                .partial_cmp(&(other.as_ref().to_u8()))
                .unwrap(),
        )
    }
}

impl Clone for Box<dyn BinaryOperationTrait> {
    fn clone(&self) -> Self {
        match self.as_ref() {
            op if op.to_string() == "&&" => Box::new(And),
            op if op.to_string() == "||" => Box::new(Or),

            op if op.to_string() == "==" => Box::new(Equal),
            op if op.to_string() == "!=" => Box::new(NotEqual),
            op if op.to_string() == ">" => Box::new(GreaterThan),
            op if op.to_string() == ">=" => Box::new(GreaterThanOrEquality),
            op if op.to_string() == "<" => Box::new(LessThan),
            op if op.to_string() == "<=" => Box::new(LessThanOrEqual),

            op if op.to_string() == "+" => Box::new(Addition),
            op if op.to_string() == "-" => Box::new(Subtraction),
            op if op.to_string() == "*" => Box::new(Multiplication),
            op if op.to_string() == "/" => Box::new(Division),
            op if op.to_string() == "%" => Box::new(Modulo),

            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub operator: Box<dyn BinaryOperationTrait>,

    pub left: Box<Instruction>,
    pub right: Box<Instruction>,
}

impl PartialEq for Binary {
    fn eq(&self, other: &Self) -> bool {
        self.operator == other.operator.clone()
            && self.left == other.left
            && self.right == other.right
    }
}

impl std::fmt::Display for Binary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.left, self.operator, self.right)
    }
}

impl TypeCheck for Binary {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        let left_type = self.left.type_check(environment, token, messages);
        let right_type = self.right.type_check(environment, token, messages);

        let valid_types = self.operator.valid_types();

        let valid_operation = valid_types
            .iter()
            .any(|(left, right)| left == &left_type && right == &right_type);

        let valid_left_types = valid_types
            .iter()
            .map(|(left, _right)| left.clone())
            .collect::<Vec<_>>();

        let valid_right_types = valid_types
            .iter()
            .filter(|(left, _right)| left == &left_type)
            .map(|(_left, right)| right.clone())
            .collect::<Vec<_>>();

        if valid_operation {
            self.operator
                .resulting_types()
                .get(&(left_type.clone(), right_type.clone()))
                .cloned()
                .unwrap()
        } else {
            if valid_left_types.contains(&left_type) {
                messages.push(ParserMessage::error_mismatched_type(
                    valid_right_types,
                    right_type.clone(),
                    self.right.token.clone(),
                ));
            } else {
                messages.push(ParserMessage::error_mismatched_type(
                    valid_left_types,
                    left_type.clone(),
                    self.left.token.clone(),
                ));
            }
            Type::None
        }
    }
}

impl Interpret for Binary {
    fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.left.interpret(environment, process)?;
        let right = self.right.interpret(environment, process)?;

        Ok(self.operator.operate(&left, &right))
    }
}
