use crate::{
    environment::{Environment, ParserEnvironment},
    error::{InterpreterError, ParserMessage, ParserWarning, ParserWarningType},
    interpreter::Interpret,
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
        token: &crate::token::Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        let mut result = Type::None;
        environment.add_scope();
        for (index, statement) in self.statements.iter().enumerate() {
            let r#type = statement.type_check(environment, token, messages);
            if r#type != Type::None && index != self.statements.len() - 1 {
                messages.push(ParserMessage::Warning(ParserWarning {
                    r#type: ParserWarningType::UnusedValue,
                    token: statement.token.clone(),
                }));
            }
            result = r#type;
        }
        environment.remove_scope();
        result
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
