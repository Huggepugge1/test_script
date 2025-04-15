use std::collections::HashMap;

use crate::{instruction::InstructionResult, r#type::Type};

use super::{BinaryOperationTrait, BinaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub struct Multiplication;

impl std::fmt::Display for Multiplication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}

impl BinaryOperationTrait for Multiplication {
    fn valid_types(&self) -> Vec<(Type, Type)> {
        vec![
            (Type::Int, Type::Int),
            (Type::Float, Type::Float),
            (Type::String, Type::String),
        ]
    }
    fn resulting_types(&self) -> HashMap<(Type, Type), Type> {
        let mut map = HashMap::new();
        map.insert((Type::Int, Type::Int), Type::Int);
        map.insert((Type::Float, Type::Float), Type::Float);
        map.insert((Type::String, Type::String), Type::String);
        map
    }

    fn operate(&self, left: &InstructionResult, right: &InstructionResult) -> InstructionResult {
        match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Int(left * right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Float(left * right)
            }
            (InstructionResult::String(left), InstructionResult::Int(right)) => {
                InstructionResult::String(left.repeat(*right as usize))
            }
            _ => unreachable!(),
        }
    }

    fn to_u8(&self) -> u8 {
        BinaryOperator::Multiplication.to_u8()
    }

    fn value(&self) -> BinaryOperator {
        BinaryOperator::Multiplication
    }
}
