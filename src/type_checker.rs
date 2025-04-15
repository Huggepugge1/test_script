use crate::cli::Args;
use crate::environment::ParserEnvironment;
use crate::error::ParserMessage;
use crate::instruction::Instruction;
use crate::r#type::Type;
use crate::token::Token;

pub trait TypeCheck {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type;
}

pub struct TypeChecker {
    program: Vec<Instruction>,
    environment: ParserEnvironment,
}

impl TypeChecker {
    pub fn new(program: Vec<Instruction>, args: Args) -> Self {
        Self {
            program,
            environment: ParserEnvironment::new(args.clone()),
        }
    }

    pub fn check(&mut self) -> Result<(), Vec<ParserMessage>> {
        let mut result: Vec<ParserMessage> = Vec::new();
        for instruction in self.program.clone() {
            instruction.type_check(&mut self.environment, &Token::none(), &mut result);
        }
        match result.is_empty() {
            true => Ok(()),
            false => Err(result),
        }
    }
}
