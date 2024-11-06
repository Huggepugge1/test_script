use crate::error::{InterpreterError, InterpreterErrorType};
use crate::instruction::{BuiltIn, Instruction, InstructionType};

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
enum InstructionResult {
    String(String),
    Regex(Vec<String>),
    None,
}

struct Test {
    name: String,
    file: PathBuf,
    expressions: Vec<Instruction>,
    variables: HashMap<String, InstructionResult>,
    input: Vec<String>,
    output: Vec<String>,
}

impl Test {
    fn new(name: String, file: PathBuf, expressions: Vec<Instruction>) -> Self {
        Self {
            name,
            file,
            expressions,
            variables: HashMap::new(),
            input: Vec::new(),
            output: Vec::new(),
        }
    }

    fn run_test(&mut self) -> Result<(), ()> {
        let mut process = Process::spawn(self.file.clone()).expect("Failed to run test");
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
                    line, self.output[i]
                ));
                return Err(());
            }
        }

        process.kill().expect("Failed to kill process");
        Ok(())
    }

    fn run(&mut self) {
        for expression in self.expressions.clone() {
            match self.interpret_expression(expression) {
                Ok(_) => (),
                Err(e) => {
                    self.fail("Failed to interpret expression");
                    e.print();
                    return;
                }
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

    fn interpret_builtin(
        &mut self,
        builtin: BuiltIn,
    ) -> Result<InstructionResult, InterpreterError> {
        let value = match builtin.clone() {
            BuiltIn::Input(value) => self.interpret_expression(*value)?,
            BuiltIn::Output(value) => self.interpret_expression(*value)?,
            BuiltIn::Print(value) => self.interpret_expression(*value)?,
            BuiltIn::Println(value) => self.interpret_expression(*value)?,
        };
        let value = match value {
            InstructionResult::String(value) => value,
            InstructionResult::None => String::new(),
            _ => {
                return Err(InterpreterError::new(
                    InterpreterErrorType::IncompatibleTypes,
                    "Expected a string or nothing",
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

    fn interpret_for(
        &mut self,
        expressions: Vec<Instruction>,
        assignment: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let mut result = InstructionResult::None;
        let (assignment_var, assignment_values) = match assignment.r#type {
            InstructionType::Assignment(var, instruction) => {
                (var, self.interpret_expression(*instruction)?)
            }
            _ => {
                unreachable!();
            }
        };
        match assignment_values {
            InstructionResult::Regex(values) => {
                for value in values {
                    self.variables
                        .insert(assignment_var.clone(), InstructionResult::String(value));
                    for expression in expressions.clone() {
                        result = self.interpret_expression(expression)?;
                    }
                }
            }
            _ => {
                return Err(InterpreterError::new(
                    InterpreterErrorType::IncompatibleTypes,
                    "Expected an iterable",
                ));
            }
        }
        Ok(result)
    }

    fn interpret_variable(&self, var: String) -> Result<InstructionResult, InterpreterError> {
        match self.variables.get(&var) {
            Some(value) => Ok(value.clone()),
            None => Err(InterpreterError::new(
                InterpreterErrorType::IncompatibleTypes,
                "Variable not found",
            )),
        }
    }

    fn interpret_expression(
        &mut self,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        Ok(match instruction.r#type {
            InstructionType::StringLiteral(value) => InstructionResult::String(value),
            InstructionType::RegexLiteral(value) => InstructionResult::Regex(value),
            InstructionType::Variable(var) => self.interpret_variable(var)?,
            InstructionType::BuiltIn(builtin) => self.interpret_builtin(builtin)?,
            InstructionType::For(expressions, assignment) => {
                self.interpret_for(expressions, *assignment)?
            }
            InstructionType::None => InstructionResult::None,
            _ => {
                unreachable!();
            }
        })
    }
}

struct Process {
    child: Child,
}

impl Process {
    fn spawn(command: PathBuf) -> Result<Self, Error> {
        let child = Command::new(command)
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

    fn interpret_test(&self, test: Instruction) {
        match test.r#type {
            InstructionType::Test(expressions, name, file) => {
                let mut test = Test::new(name, file, expressions);
                test.run();
            }
            _ => panic!("Unexpected instruction {:?}", test),
        }
    }

    pub fn interpret(&self) {
        for test in self.program.clone().into_iter() {
            self.interpret_test(test);
        }
    }
}
