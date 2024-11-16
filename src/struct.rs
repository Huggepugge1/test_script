use std::io::{self, BufRead, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct InteractiveProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl InteractiveProcess {
    /// Creates a new process and initializes pipes
    fn new(command: &str, args: &[&str]) -> io::Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Optionally handle stderr as well
            .spawn()?;

        let stdin = child.stdin.take().expect("Failed to capture stdin");
        let stdout = child.stdout.take().expect("Failed to capture stdout");

        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    /// Write to the child process's stdin
    fn write(&mut self, input: &str) -> io::Result<()> {
        writeln!(self.stdin, "{}", input)?; // Write the input to stdin
        self.stdin.flush()?; // Ensure the data is sent immediately
        Ok(())
    }

    /// Read a line from the child process's stdout
    fn read_line(&mut self) -> io::Result<String> {
        let mut reader = io::BufReader::new(&mut self.stdout);
        let mut output = String::new();
        reader.read_line(&mut output)?; // Read one line
        Ok(output)
    }

    /// Terminate the child process and handle the exit code
    fn terminate(mut self) -> io::Result<()> {
        let status = self.child.wait()?; // Wait for the child to exit
        if status.success() {
            println!("Process exited successfully.");
        } else {
            eprintln!("Process exited with code: {:?}", status.code());
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    // Create an interactive process (e.g., bash)
    let mut process = InteractiveProcess::new("bash", &[])?;

    // Interact with the process
    process.write("echo Hello from Rust")?;
    let output = process.read_line()?;
    println!("Received: {}", output);

    process.write("echo Hello from Rust")?;
    let output = process.read_line()?;
    println!("Received: {}", output);

    // Terminate the process
    process.write("exit")?;
    process.terminate()?;

    Ok(())
}
