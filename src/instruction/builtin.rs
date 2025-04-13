use crate::error::InterpreterError;

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

impl BuiltIn {
    pub fn interpret(
        &self,
        environment: &mut crate::environment::Environment,
        process: &mut Option<&mut crate::process::Process>,
    ) -> Result<crate::instruction::InstructionResult, crate::error::InterpreterError> {
        match self.name.as_str() {
            "print" => self.interpret_print(environment, process),
            "println" => self.interpret_println(environment, process),
            "input" => self.interpret_input(environment, process),
            "output" => self.interpret_output(environment, process),
            _ => unreachable!(),
        }
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
