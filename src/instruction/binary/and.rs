use std::collections::HashMap;

use crate::{instruction::InstructionResult, r#type::Type};

use super::{BinaryOperationTrait, BinaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub struct And;

impl std::fmt::Display for And {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.operator())
    }
}

impl BinaryOperationTrait for And {
    fn operator(&self) -> BinaryOperator {
        BinaryOperator::And
    }

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
                InstructionResult::Bool(*left && *right)
            }
            _ => unreachable!(),
        }
    }

    fn priority(&self) -> u8 {
        BinaryOperator::And.to_u8()
    }
}
