use crate::parser::{BuiltIn, Instruction};

use itertools::Itertools;
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

struct Test {
    name: String,
    file: PathBuf,
    expressions: Vec<Instruction>,
    input: Vec<Vec<String>>,
    output: Vec<Vec<String>>,
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

    fn run_test(&mut self, input: Vec<&String>, output: Vec<String>) -> Result<(), ()> {
        let mut process = Process::spawn(self.file.clone()).expect("Failed to run test");
        for line in input {
            process.sendline(&line).expect("Failed to send input");
        }

        let lines = process.get_lines().expect("Failed to get output");
        for (i, line) in lines.iter().enumerate() {
            if *line != output[i] {
                self.fail(&format!("Expected output: {}, got: {}", output[i], line));
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

        let combined_input = self.input.clone();
        let combined_input = combined_input
            .iter()
            .multi_cartesian_product()
            .collect::<Vec<_>>();

        let combined_output = self
            .input
            .iter()
            .zip(self.output.iter())
            .map(|(a, b)| {
                let extended: Vec<_> = b.iter().cycle().take(a.len()).cloned().collect::<Vec<_>>();
                extended
            })
            .multi_cartesian_product()
            .collect::<Vec<_>>();

        for (input, output) in combined_input.iter().zip(combined_output.iter()) {
            if self.run_test(input.clone(), output.clone()).is_err() {
                return;
            }
        }
        self.pass();
    }

    fn pass(&self) {
        println!("Test passed: {}", self.name);
    }

    fn fail(&self, message: &str) {
        eprintln!("{} failed: {}", self.name, message);
    }

    fn input(&mut self, value: Vec<String>) -> Vec<String> {
        self.input.push(value.clone());
        value
    }

    fn output(&mut self, value: Vec<String>) -> Vec<String> {
        self.output.push(value.clone());
        value
    }

    fn interpret_literal(&self, value: Vec<String>) -> Vec<String> {
        value
    }

    fn interpret_builtin(&mut self, builtin: BuiltIn) -> Vec<String> {
        let value = match builtin.clone() {
            BuiltIn::Input(value) => self.interpret_expression(*value),
            BuiltIn::Output(value) => self.interpret_expression(*value),
        };
        match builtin {
            BuiltIn::Input(_) => self.input(value),
            BuiltIn::Output(_) => self.output(value),
        }
    }

    fn interpret_expression(&mut self, instruction: Instruction) -> Vec<String> {
        match instruction {
            Instruction::Literal(value) => self.interpret_literal(value),
            Instruction::BuiltIn(builtin) => self.interpret_builtin(builtin),
            Instruction::None => Vec::new(),
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

    fn sendline(&mut self, input: &str) -> Result<(), Error> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            stdin.write_all(input.as_bytes())?;
            stdin.write_all(b"\n")?; // Send newline to simulate "Enter" key
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

        // Use a buffered reader for non-blocking line-by-line reading
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
    match test {
        Instruction::Test(expressions, name, file) => {
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
