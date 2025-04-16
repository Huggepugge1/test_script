use std::collections::HashMap;

use crate::{instruction::InstructionResult, r#type::Type};

use super::{BinaryOperationTrait, BinaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub struct Division;

impl std::fmt::Display for Division {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.operator())
    }
}

impl BinaryOperationTrait for Division {
    fn operator(&self) -> BinaryOperator {
        BinaryOperator::Division
    }

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
                InstructionResult::Int(left / right)
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                InstructionResult::Float(left / right)
            }
            _ => unreachable!(),
        }
    }

    fn priority(&self) -> u8 {
        BinaryOperator::Multiplication.to_u8()
    }
}
