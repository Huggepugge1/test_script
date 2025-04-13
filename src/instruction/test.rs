use crate::environment::Environment;
use crate::{cli::Args, interpreter::Test, process::Process};

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
