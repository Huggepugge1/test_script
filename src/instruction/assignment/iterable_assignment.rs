use crate::{
    environment::ParserEnvironment,
    error::{ParserError, ParserErrorType},
    instruction::{variable::Variable, Instruction},
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

#[derive(Debug, Clone, PartialEq)]
pub struct IterableAssignment {
    pub variable: Variable,
    pub body: Box<Instruction>,
}

impl std::fmt::Display for IterableAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in {}", self.variable, self.body)
    }
}

impl TypeCheck for IterableAssignment {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        _token: &Token,
    ) -> Result<Type, ParserError> {
        let variable_type = self.variable.r#type.clone();
        let body_type = self.body.type_check(environment)?;
        if body_type.is_iterable() && body_type.get_iterable_inner_type() == variable_type {
            environment.insert(self.variable.clone());
            if let Some(variable) = environment.get(&self.variable.name) {
                variable.assigned = true;
            }
            Ok(Type::None)
        } else {
            Err(ParserError::new(
                ParserErrorType::MismatchedType {
                    expected: vec![Type::Iterable(Box::new(variable_type.clone()))],
                    actual: variable_type,
                },
                _token.clone(),
            ))
        }
    }
}
