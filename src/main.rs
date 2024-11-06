mod cli;
mod error;
mod instruction;
mod interpreter;
mod lexer;
mod parser;
mod regex;
mod test;
mod token;

fn main() {
    cli::run();
}
