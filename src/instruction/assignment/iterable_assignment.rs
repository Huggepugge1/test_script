use crate::{
    environment::ParserEnvironment,
    error::ParserMessage,
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
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        let variable_type = self.variable.r#type.clone();
        let body_type = self.body.type_check(environment, token, messages);

        if body_type.is_iterable() && body_type.get_iterable_inner_type() == variable_type {
            environment.insert(self.variable.clone());
            if let Some(variable) = environment.get(&self.variable.name) {
                variable.assigned = true;
            }
            Type::None
        } else {
            messages.push(ParserMessage::error_mismatched_type(
                vec![Type::Iterable(Box::new(variable_type.clone()))],
                variable_type,
                token.clone(),
            ));
            Type::None
        }
    }
}
