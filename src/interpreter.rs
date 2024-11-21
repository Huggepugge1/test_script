use crate::cli::Args;
use crate::environment::Environment;
use crate::error::InterpreterError;
use crate::instruction::{Instruction, InstructionType};
use crate::process::Process;

struct Test {
    name: String,
    instruction: Instruction,
    process: Process,
    passed: bool,
}

impl Test {
    fn new(name: String, command: String, instruction: Instruction, args: Args) -> Self {
        let process = Process::new(&command, args.debug);

        Self {
            name,

            instruction,
            process,
            passed: true,
        }
    }

    fn run(&mut self, environment: &mut Environment) {
        environment.add_frame();
        let instruction = self.instruction.clone();
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
        let environment = Environment::new();
        Self {
            program,
            args,
            environment,
        }
    }

    fn interpret_test(&mut self, instruction: Instruction) {
        match instruction.r#type {
            InstructionType::Test(instruction, name, file) => {
                let mut test = Test::new(name, file, *instruction, self.args.clone());
                test.run(&mut self.environment);
            }
            _ => {
                unreachable!()
            }
        }
    }

    pub fn interpret(&mut self) {
        for instruction in self.program.clone().into_iter() {
            match instruction.r#type {
                InstructionType::Test(_, _, _) => self.interpret_test(instruction),
                InstructionType::Function { .. } => {
                    let _ = instruction.interpret(&mut self.environment, &mut None);
                }

                InstructionType::Assignment {
                    variable,
                    instruction,
                    ..
                } => {
                    let result = match instruction.interpret(&mut self.environment, &mut None) {
                        Ok(value) => value,
                        Err(e) => {
                            e.print();
                            return;
                        }
                    };
                    self.environment.insert(variable.name, result);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}
