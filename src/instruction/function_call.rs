use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserMessage},
    interpreter::Interpret,
    process::Process,
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

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

impl TypeCheck for FunctionCall {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        let function = environment.get_function(&self.name).unwrap().clone();
        if function.parameters.len() != self.arguments.len() {
            messages.push(ParserMessage::error_mismatched_number_of_arguments(
                function.parameters.len(),
                self.arguments.len(),
                token.clone(),
            ));
        }

        for (parameter, argument) in function.parameters.iter().zip(&self.arguments) {
            let argument_type = argument.type_check(environment, token, messages);
            if parameter.r#type != argument_type {
                messages.push(ParserMessage::error_mismatched_type(
                    vec![parameter.r#type.clone()],
                    argument_type,
                    token.clone(),
                ));
            }
        }

        function.return_type
    }
}

impl Interpret for FunctionCall {
    fn interpret(
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
