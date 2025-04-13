pub mod iterable_assignment;

use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserError, ParserErrorType},
    instruction::{variable::Variable, Instruction, InstructionResult},
    process::Process,
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub variable: Variable,
    pub body: Box<Instruction>,
    pub token: Token,
    pub declaration: bool,
}

impl std::fmt::Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.variable, self.body)
    }
}

impl TypeCheck for Assignment {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
    ) -> Result<Type, ParserError> {
        let variable_type = self.variable.r#type.clone();
        let body_type = self.body.type_check(environment)?;

        if variable_type != Type::Any && variable_type != body_type {
            return Err(ParserError::new(
                ParserErrorType::MismatchedType {
                    expected: vec![variable_type],
                    actual: body_type,
                },
                token.clone(),
            ));
        }

        let mut variable = match environment.get(&self.variable.name) {
            Some(variable) => variable.clone(),
            None => self.variable.clone(),
        };

        variable.read = false;
        variable.last_assignment_token = token.clone();
        variable.assigned = true;

        environment.insert(variable);
        Ok(Type::None)
    }
}

impl Assignment {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let result = self.body.interpret(environment, process)?;
        environment.insert(self.variable.name.clone(), result.clone());
        Ok(result)
    }
}
