mod cli;
mod environment;
mod error;
mod instruction;
mod interpreter;
mod lexer;
mod parser;
mod regex;
mod test;
mod token;
mod r#type;

fn main() {
    cli::run();
}
