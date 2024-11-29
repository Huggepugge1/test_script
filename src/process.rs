use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::error::InterpreterError;
use crate::exitcode::ExitCode;

pub struct Process {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    debug: bool,
}

fn split_command(command: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in command.chars() {
        match c {
            '\'' if in_quotes => {
                in_quotes = false; // End of a quoted argument
            }
            '\'' => {
                in_quotes = true; // Start of a quoted argument
            }
            ' ' if !in_quotes => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(c); // Part of the current argument
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

impl Process {
    pub fn new(command: &str, debug: bool) -> Self {
        let command_vec = split_command(command);
        let child = Command::new(command_vec[0].clone())
            .args(command_vec[1..].iter())
            .spawn();

        match child {
            Ok(mut child) => {
                let _ = child.kill();
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!("Failed to find command: {}", command);
                    std::process::exit(ExitCode::ProcessNotFound as i32);
                }
                ErrorKind::PermissionDenied => {
                    eprintln!("Permission denied to run command: {}", command);
                    std::process::exit(ExitCode::ProcessPermissionDenied as i32);
                }
                _ => {
                    eprintln!("Failed to run command: {}", command);
                    std::process::exit(ExitCode::Unknown as i32);
                }
            },
        }

        let mut child = match Command::new("stdbuf")
            .arg("-o0")
            .arg("-e0")
            .args(command_vec.iter())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!("Failed to find command: {}", command);
                    std::process::exit(ExitCode::ProcessNotFound as i32);
                }
                ErrorKind::PermissionDenied => {
                    eprintln!("Permission denied to run command: {}", command);
                    std::process::exit(ExitCode::ProcessPermissionDenied as i32);
                }
                _ => {
                    eprintln!("Failed to run command: {}", command);
                    std::process::exit(ExitCode::Unknown as i32);
                }
            },
        };

        let stdin = child.stdin.take().expect("Failed to capture stdin");
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let reader = BufReader::new(stdout);

        Self {
            child,
            stdin,
            reader,
            debug,
        }
    }

    pub fn send(&mut self, input: &str) -> Result<(), InterpreterError> {
        let lines = input.split('\n');
        for line in lines {
            if self.debug {
                println!("Sending: {}", line);
            }
            writeln!(self.stdin, "{}", line).map_err(|_| {
                InterpreterError::TestFailed("Failed to write to stdin".to_string())
            })?;
            self.stdin
                .flush()
                .map_err(|_| InterpreterError::TestFailed("Failed to flush stdin".to_string()))?;
        }
        if self.debug {
            println!("Sent: {}", input);
        }
        Ok(())
    }

    pub fn read_line(&mut self, expected: String) -> Result<(), InterpreterError> {
        if self.debug {
            println!("Reading line");
        }

        for line in expected.lines() {
            let mut output = String::new();
            self.reader
                .read_line(&mut output)
                .map_err(|_| InterpreterError::TestFailed("Failed to read line".to_string()))?;

            if self.debug {
                println!("Read: {}", output);
            }

            if output.trim_end() != line {
                return Err(InterpreterError::TestFailed(format!(
                    "Expected: `{}`, got: `{}`",
                    line,
                    output.trim_end()
                )));
            }
        }
        Ok(())
    }

    pub fn terminate(&mut self) -> Result<(), InterpreterError> {
        let status = self.child.wait().map_err(|_| {
            InterpreterError::TestFailed("Failed to wait for child process".to_string())
        })?;

        if let Some(signal) = status.signal() {
            return Err(InterpreterError::TestFailed(format!(
                "Process terminated by signal: {}",
                signal
            )));
        }

        match status.code() {
            Some(0) => Ok(()),
            Some(code) => Err(InterpreterError::TestFailed(format!(
                "Process exited with code: {}",
                code
            ))),
            None => Err(InterpreterError::TestFailed(
                "Process terminated without exit code".to_string(),
            )),
        }
    }
}
