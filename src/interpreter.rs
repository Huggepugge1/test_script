use crate::instruction::{BuiltIn, Instruction, InstructionType};

use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

struct Test {
    name: String,
    file: PathBuf,
    expressions: Vec<Instruction>,
    input: Vec<String>,
    output: Vec<String>,
}

impl Test {
    fn new(name: String, file: PathBuf, expressions: Vec<Instruction>) -> Self {
        Self {
            name,
            file,
            expressions,
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
            self.interpret_expression(expression);
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

    fn interpret_builtin(&mut self, builtin: BuiltIn) -> String {
        let value = match builtin.clone() {
            BuiltIn::Input(value) => self.interpret_expression(*value),
            BuiltIn::Output(value) => self.interpret_expression(*value),
        };
        match builtin {
            BuiltIn::Input(_) => self.input.push(value.clone()),
            BuiltIn::Output(_) => self.output.push(value.clone()),
        }
        return value;
    }

    fn interpret_expression(&mut self, instruction: Instruction) -> String {
        match instruction.r#type {
            InstructionType::StringLiteral(value) => value,
            InstructionType::BuiltIn(builtin) => self.interpret_builtin(builtin),
            InstructionType::None => String::new(),
            _ => panic!("Unexpected instruction {:?}", instruction),
        }
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

fn interpret_test(test: Instruction) {
    match test.r#type {
        InstructionType::Test(expressions, name, file) => {
            let mut test = Test::new(name, file, expressions);
            test.run();
        }
        _ => panic!("Unexpected instruction {:?}", test),
    }
}

pub fn interpret(program: Vec<Instruction>) {
    for test in program {
        interpret_test(test);
    }
}
