use crate::environment::Environment;
use crate::error::{InterpreterError, InterpreterErrorType};
use crate::instruction::{BuiltIn, Instruction, InstructionType};
use crate::r#type::Type;

use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
pub enum InstructionResult {
    String(String),
    Regex(Vec<String>),
    None,
}

impl std::fmt::Display for InstructionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionResult::String(_) => write!(f, "string"),
            InstructionResult::Regex(_) => write!(f, "regex"),
            InstructionResult::None => write!(f, "none"),
        }
    }
}

struct Test {
    name: String,
    command: String,
    instruction: Instruction,
    environment: Environment,
    input: Vec<String>,
    output: Vec<String>,
}

impl Test {
    fn new(name: String, command: String, instruction: Instruction) -> Self {
        Self {
            name,
            command,

            instruction,
            environment: Environment::new(),
            input: Vec::new(),
            output: Vec::new(),
        }
    }

    fn run_test(&mut self) -> Result<(), ()> {
        let mut process = Process::spawn(self.command.clone()).expect("Failed to run test");
        let mut buffer = String::new();
        for line in self.input.clone() {
            buffer.push_str(&line);
            buffer.push('\n');
        }
        process.send(&buffer).expect("Failed to send input");

        let lines = process.get_lines().expect("Failed to get output");

        if lines.len() != self.output.len() {
            self.fail(&format!(
                "Expected output length: {}, got: {}",
                self.output.len(),
                lines.len()
            ));
            return Err(());
        }

        for (i, line) in lines.iter().enumerate() {
            if self.output[i] != *line {
                self.fail(&format!(
                    "Expected output: {:?}, got: {:?}",
                    self.output[i], line
                ));
                return Err(());
            }
        }

        process.kill().expect("Failed to kill process");
        Ok(())
    }

    fn run(&mut self) {
        let instruction = self.instruction.clone();
        match self.interpret_instruction(instruction) {
            Ok(_) => (),
            Err(e) => {
                e.print();
                return;
            }
        }

        match self.run_test() {
            Ok(_) => self.pass(),
            Err(_) => (),
        }
    }

    fn pass(&self) {
        println!("Test passed: {}", self.name);
    }

    fn fail(&self, message: &str) {
        eprintln!("{} failed: {}", self.name, message);
    }

    fn interpret_addition(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::String(left), InstructionResult::String(right)) => {
                Ok(InstructionResult::String(format!("{}{}", left, right)))
            }
            _ => {
                return Err(InterpreterError::new(
                    InterpreterErrorType::IncompatibleTypesBinary(
                        InstructionResult::String(String::new()),
                        left,
                        InstructionResult::String(String::new()),
                        right,
                    ),
                    "Expected two strings",
                ));
            }
        }
    }

    fn interpret_builtin(
        &mut self,
        builtin: BuiltIn,
    ) -> Result<InstructionResult, InterpreterError> {
        let value = match builtin.clone() {
            BuiltIn::Input(value) => self.interpret_instruction(*value)?,
            BuiltIn::Output(value) => self.interpret_instruction(*value)?,
            BuiltIn::Print(value) => self.interpret_instruction(*value)?,
            BuiltIn::Println(value) => self.interpret_instruction(*value)?,
        };
        let value = match value {
            InstructionResult::String(value) => value,
            _ => {
                return Err(InterpreterError::new(
                    InterpreterErrorType::IncompatibleTypes(
                        InstructionResult::String(String::new()),
                        value,
                    ),
                    "Expected a string",
                ));
            }
        };
        match builtin {
            BuiltIn::Input(_) => self.input.push(value.clone()),
            BuiltIn::Output(_) => self.output.push(value.clone()),
            BuiltIn::Print(_) => print!("{}", value),
            BuiltIn::Println(_) => println!("{}", value),
        }
        Ok(InstructionResult::None)
    }

    fn interpret_assignment(
        &mut self,
        var: String,
        _type: Type,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let value = self.interpret_instruction(instruction)?;
        self.environment.insert(var.clone(), value.clone());
        Ok(value)
    }

    fn interpret_block(
        &mut self,
        instructions: Vec<Instruction>,
    ) -> Result<InstructionResult, InterpreterError> {
        let mut result = InstructionResult::None;
        self.environment.add_scope();
        for instruction in instructions {
            result = self.interpret_instruction(instruction)?;
        }
        self.environment.remove_scope();
        Ok(result)
    }

    fn interpret_for(
        &mut self,
        assignment: Instruction,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let mut result = InstructionResult::None;
        let (assignment_var, assignment_values) = match assignment.r#type {
            InstructionType::IterableAssignment(var, _type, instruction) => {
                (var, self.interpret_instruction(*instruction)?)
            }
            _ => {
                println!("{:?}", assignment);
                unreachable!();
            }
        };
        match assignment_values {
            InstructionResult::Regex(values) => {
                for value in values {
                    self.environment
                        .insert(assignment_var.clone(), InstructionResult::String(value));
                    result = self.interpret_instruction(instruction.clone())?;
                }
            }
            _ => {
                return Err(InterpreterError::new(
                    InterpreterErrorType::IncompatibleTypes(
                        InstructionResult::Regex(Vec::new()),
                        assignment_values,
                    ),
                    "Expected an iterable",
                ));
            }
        }
        Ok(result)
    }

    fn interpret_variable(&self, var: String) -> Result<InstructionResult, InterpreterError> {
        match self.environment.get(&var) {
            Some(value) => Ok(value.clone()),
            None => unreachable!(),
        }
    }

    fn interpret_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        Ok(match instruction.r#type {
            InstructionType::StringLiteral(value) => InstructionResult::String(value),
            InstructionType::RegexLiteral(value) => InstructionResult::Regex(value),
            InstructionType::Addition(left, right) => self.interpret_addition(*left, *right)?,

            InstructionType::Variable(var) => self.interpret_variable(var)?,
            InstructionType::BuiltIn(builtin) => self.interpret_builtin(builtin)?,

            InstructionType::For(assignment, instruction) => {
                self.interpret_for(*assignment, *instruction)?
            }
            InstructionType::Block(instructions) => self.interpret_block(instructions)?,
            InstructionType::Assignment(var, r#type, instruction) => {
                self.interpret_assignment(var, r#type, *instruction)?
            }

            InstructionType::None => InstructionResult::None,
            _ => {
                println!("{:?}", instruction);
                unreachable!();
            }
        })
    }
}

struct Process {
    child: Child,
}

impl Process {
    fn spawn(command: String) -> Result<Self, Error> {
        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        Ok(Process { child })
    }

    fn send(&mut self, input: &str) -> Result<(), Error> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            stdin.write_all(input.as_bytes())?;
        }
        Ok(())
    }

    fn get_lines(&mut self) -> Result<Vec<String>, Error> {
        let stdout = self.child.stdout.as_mut().ok_or_else(|| {
            Error::new(
                ErrorKind::BrokenPipe,
                "Failed to capture child process stdout",
            )
        })?;

        let mut reader = BufReader::new(stdout);
        let mut output = Vec::new();

        loop {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            output.push(line.trim().to_string());
        }

        Ok(output)
    }

    fn kill(&mut self) -> Result<(), Error> {
        self.child.kill()
    }
}

pub struct Interpreter {
    program: Vec<Instruction>,
}

impl Interpreter {
    pub fn new(program: Vec<Instruction>) -> Self {
        Self { program }
    }

    fn interpret_test(&self, instruction: Instruction) {
        match instruction.r#type {
            InstructionType::Test(instruction, name, file) => {
                let mut test = Test::new(name, file, *instruction);
                test.run();
            }
            _ => panic!("Unexpected instruction {:?}", instruction),
        }
    }

    pub fn interpret(&self) {
        for test in self.program.clone().into_iter() {
            self.interpret_test(test);
        }
    }
}
