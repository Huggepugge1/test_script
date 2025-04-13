use crate::cli::Args;
use crate::environment::ParserEnvironment;
use crate::error::ParserError;
use crate::instruction::Instruction;
use crate::r#type::Type;
use crate::token::Token;

pub trait TypeCheck {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
    ) -> Result<Type, ParserError>;
}

pub struct TypeChecker {
    program: Vec<Instruction>,
    environment: ParserEnvironment,
    success: bool,
}

impl TypeChecker {
    pub fn new(program: Vec<Instruction>, args: Args) -> Self {
        Self {
            program,
            environment: ParserEnvironment::new(args.clone()),
            success: true,
        }
    }

    pub fn check(&mut self) -> Result<(), ParserError> {
        for instruction in self.program.clone() {
            instruction.type_check(&mut self.environment)?;
        }
        match self.success {
            true => Ok(()),
            false => Err(ParserError::none()),
        }
    }
}
