use crate::{environment::Environment, error::InterpreterError, process::Process};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Instruction>,
}

impl std::fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.arguments
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl FunctionCall {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let function = environment.get_function(&self.name).unwrap().clone();
        environment.add_frame();
        let arguments = self
            .arguments
            .iter()
            .map(|arg| arg.interpret(environment, process))
            .collect::<Result<Vec<_>, _>>()?;

        for (parameter, argument) in function.parameters.iter().zip(arguments) {
            environment.insert(parameter.name.clone(), argument);
        }

        environment.add_scope();
        let result = function.body.interpret(environment, process);
        environment.remove_scope();
        environment.remove_frame();
        result
    }
}
