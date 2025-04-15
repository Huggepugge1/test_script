use crate::{
    environment::ParserEnvironment,
    error::{InterpreterError, ParserMessage},
    interpreter::Interpret,
    r#type::Type,
    token::Token,
    type_checker::TypeCheck,
};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltIn {
    pub name: String,
    pub arguments: Vec<Instruction>,
}

impl std::fmt::Display for BuiltIn {
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

impl TypeCheck for BuiltIn {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        match self.name.as_str() {
            "print" | "println" => self.type_check_print(environment, token, messages),
            "input" | "output" => self.type_check_input_output(environment, token, messages),
            _ => unreachable!(),
        }
    }
}

impl Interpret for BuiltIn {
    fn interpret(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        match self.name.as_str() {
            "print" => self.interpret_print(environment, process),
            "println" => self.interpret_println(environment, process),
            "input" => self.interpret_input(environment, process),
            "output" => self.interpret_output(environment, process),
            _ => unreachable!(),
        }
    }
}

impl BuiltIn {
    fn type_check_print(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        if self.arguments.is_empty() {
            messages.push(ParserMessage::error_mismatched_number_of_arguments(
                1,
                self.arguments.len(),
                token.clone(),
            ));
        }
        for arg in &self.arguments {
            let r#type = arg.type_check(environment, token, messages);
            if r#type != Type::String {
                messages.push(ParserMessage::error_mismatched_type(
                    vec![Type::String],
                    r#type,
                    token.clone(),
                ));
            }
        }
        Type::None
    }

    fn type_check_input_output(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        if self.arguments.len() != 1 {
            messages.push(ParserMessage::error_mismatched_number_of_arguments(
                1,
                self.arguments.len(),
                token.clone(),
            ));
        }
        let r#type = self.arguments[0].type_check(environment, token, messages);
        if r#type != Type::String {
            messages.push(ParserMessage::error_mismatched_type(
                vec![Type::String],
                r#type,
                token.clone(),
            ));
        }
        Type::None
    }

    fn args_to_string(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<String, crate::error::InterpreterError> {
        let mut result = String::new();
        for arg in &self.arguments {
            let value = arg.interpret(environment, process)?;
            result.push_str(&value.to_string());
            result.push(' ');
        }
        Ok(result)
    }

    fn interpret_print(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<crate::instruction::InstructionResult, crate::error::InterpreterError> {
        print!("{}", self.args_to_string(environment, process)?);
        Ok(crate::instruction::InstructionResult::None)
    }

    fn interpret_println(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<crate::instruction::InstructionResult, crate::error::InterpreterError> {
        println!("{}", self.args_to_string(environment, process)?);
        Ok(crate::instruction::InstructionResult::None)
    }

    fn interpret_input(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let argument = self.arguments[0].interpret(environment, process)?;
        match argument {
            InstructionResult::String(argument) => match process {
                Some(process) => process.send(&argument)?,
                None => {
                    return Err(InterpreterError::TestFailed(
                        "No process to send input to".to_string(),
                    ));
                }
            },
            _ => unreachable!(),
        }

        Ok(InstructionResult::None)
    }

    fn interpret_output(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        let argument = self.arguments[0].interpret(environment, process)?;
        match argument {
            InstructionResult::String(argument) => match process {
                Some(process) => process.read_line(&argument)?,
                None => {
                    return Err(InterpreterError::TestFailed(
                        "No process to receive output from".to_string(),
                    ));
                }
            },
            _ => unreachable!(),
        }

        Ok(InstructionResult::None)
    }
}
