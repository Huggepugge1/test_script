use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserMessage},
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

use super::{variable::Variable, Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Variable>,
    pub body: Box<Instruction>,
    pub return_type: Type,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}(", self.name)?;
        for (i, parameter) in self.parameters.iter().enumerate() {
            write!(f, "{}", parameter)?;
            if i != self.parameters.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "): {} ", self.return_type)?;
        write!(f, "{}", self.body)
    }
}

impl TypeCheck for Function {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        environment.add_function(self.clone());
        environment.add_scope();
        for parameter in &self.parameters {
            environment.insert(parameter.clone());
        }
        let result = self.body.type_check(environment, token, messages);
        environment.remove_scope();
        result
    }
}

impl Function {
    pub fn interpret(
        &self,
        environment: &mut Environment,
    ) -> Result<InstructionResult, InterpreterError> {
        environment.add_function(self.clone());
        Ok(InstructionResult::None)
    }
}
