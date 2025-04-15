use std::collections::HashMap;

use crate::{instruction::InstructionResult, r#type::Type};

use super::{BinaryOperationTrait, BinaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub struct Subtraction;

impl std::fmt::Display for Subtraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "-")
    }
}

impl BinaryOperationTrait for Subtraction {
    fn valid_types(&self) -> Vec<(Type, Type)> {
        vec![(Type::Int, Type::Int), (Type::Float, Type::Float)]
    }

    fn resulting_types(&self) -> HashMap<(Type, Type), Type> {
        let mut map = HashMap::new();
        map.insert((Type::Int, Type::Int), Type::Int);
        map.insert((Type::Float, Type::Float), Type::Float);
        map
    }

    fn operate(&self, left: &InstructionResult, right: &InstructionResult) -> InstructionResult {
        match (left, right) {
            (InstructionResult::Int(left), InstructionResult::Int(right)) => {
                InstructionResult::Int(left - right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Float(left - right)
            }
            _ => unreachable!(),
        }
    }

    fn to_u8(&self) -> u8 {
        BinaryOperator::Subtraction.to_u8()
    }

    fn value(&self) -> BinaryOperator {
        BinaryOperator::Addition
    }
}
