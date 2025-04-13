use crate::cli::Args;
use crate::environment::Environment;
use crate::error::InterpreterError;
use crate::instruction::{Instruction, InstructionType};
use crate::process::Process;

pub struct Test {
    pub name: String,
    pub body: Instruction,
    pub process: Process,
    pub passed: bool,
}

impl Test {
    pub fn run(&mut self, environment: &mut Environment) {
        environment.add_frame();
        let instruction = self.body.clone();
        match instruction.interpret(environment, &mut Some(&mut self.process)) {
            Ok(_) => (),
            Err(e) => {
                e.print();
                environment.remove_frame();
                return;
            }
        }
        environment.remove_frame();

        match self.process.terminate() {
            Ok(()) => (),
            Err(e) => {
                self.fail(e);
                return;
            }
        }

        match self.passed {
            false => (),
            true => self.pass(),
        }
    }

    fn pass(&self) {
        println!("Test passed: {}", self.name);
    }

    fn fail(&mut self, error: InterpreterError) {
        error.print();
        let _ = self.process.terminate();
    }
}

pub struct Interpreter {
    args: Args,
    program: Vec<Instruction>,
    environment: Environment,
}

impl Interpreter {
    pub fn new(program: Vec<Instruction>, args: Args) -> Self {
        let environment = Environment::default();
        Self {
            program,
            args,
            environment,
        }
    }

    pub fn interpret(&mut self) {
        for instruction in self.program.clone().into_iter() {
            match instruction.r#type {
                InstructionType::Test(test) => {
                    test.interpret(&mut self.environment, self.args.clone())
                }
                InstructionType::Function(function) => {
                    let _ = function.interpret(&mut self.environment);
                }

                InstructionType::Assignment(assignment) => {
                    let _ = assignment.interpret(&mut self.environment, &mut None);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}
