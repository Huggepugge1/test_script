use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserError, ParserWarning, ParserWarningType},
    process::Process,
    r#type::Type,
    type_checker::TypeCheck,
};

use super::{Instruction, InstructionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Instruction>,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for statement in &self.statements {
            writeln!(f, "    {}", statement)?;
        }
        write!(f, "}}")
    }
}

impl TypeCheck for Block {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        _token: &crate::token::Token,
    ) -> Result<Type, ParserError> {
        let mut result = Type::None;
        let mut failed: Option<ParserError> = None;
        environment.add_scope();
        for (index, statement) in self.statements.iter().enumerate() {
            match statement.type_check(environment) {
                Ok(value) => {
                    if value != Type::None && index != self.statements.len() - 1 {
                        ParserWarning::new(ParserWarningType::UnusedValue, statement.token.clone())
                            .print(environment.args.disable_warnings);
                    }
                    result = value;
                }
                Err(err) => {
                    if let Some(e) = failed {
                        e.print();
                    }
                    failed = Some(err);
                }
            }
        }
        environment.remove_scope();
        match failed {
            Some(err) => Err(err),
            None => Ok(result),
        }
    }
}

impl Block {
    pub fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        environment.add_scope();
        let mut result = InstructionResult::None;
        for statement in &self.statements {
            result = match statement.interpret(environment, process) {
                Ok(value) => value,
                Err(err) => {
                    environment.remove_scope();
                    return Err(err);
                }
            };
        }
        environment.remove_scope();
        Ok(result)
    }
}
