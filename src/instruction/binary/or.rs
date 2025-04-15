use std::collections::HashMap;

use crate::{instruction::InstructionResult, r#type::Type};

use super::{BinaryOperationTrait, BinaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub struct Or;

impl std::fmt::Display for Or {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "||")
    }
}

impl BinaryOperationTrait for Or {
    fn valid_types(&self) -> Vec<(Type, Type)> {
        vec![(Type::Bool, Type::Bool)]
    }

    fn resulting_types(&self) -> HashMap<(Type, Type), Type> {
        let mut map = HashMap::new();
        map.insert((Type::Bool, Type::Bool), Type::Bool);
        map
    }

    fn operate(&self, left: &InstructionResult, right: &InstructionResult) -> InstructionResult {
        match (left, right) {
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                InstructionResult::Bool(*left || *right)
            }
            _ => unreachable!(),
        }
    }

    fn to_u8(&self) -> u8 {
        BinaryOperator::Or.to_u8()
    }

    fn value(&self) -> BinaryOperator {
        BinaryOperator::And
    }
}
