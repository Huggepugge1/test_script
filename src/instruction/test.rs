use crate::{
    cli::Args,
    environment::{Environment, ParserEnvironment},
    error::ParserError,
    interpreter::Test,
    process::Process,
    r#type::Type,
    type_checker::TypeCheck,
};

use super::Instruction;

#[derive(Debug, Clone, PartialEq)]
pub struct TestInstruction {
    pub name: String,
    pub command: String,
    pub body: Box<Instruction>,
}

impl std::fmt::Display for TestInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "test {} {{ {} }}", self.name, self.body)
    }
}

impl TypeCheck for TestInstruction {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        _token: &crate::token::Token,
    ) -> Result<Type, ParserError> {
        environment.add_scope();
        let result = self.body.type_check(environment)?;
        environment.remove_scope();
        Ok(result)
    }
}

impl TestInstruction {
    pub fn interpret(&self, environment: &mut Environment, args: Args) {
        let process = Process::new(&self.command, args.debug);
        let mut test = Test {
            name: self.name.clone(),
            body: *self.body.clone(),
            process,
            passed: true,
        };

        test.run(environment);
    }
}
