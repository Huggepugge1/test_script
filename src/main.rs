mod cli;
mod environment;
mod error;
mod exitcode;
mod instruction;
mod interpreter;
mod lexer;
mod parser;
mod process;
mod regex;
mod test;
mod token;
mod r#type;
mod type_checker;
mod variable;
mod white_listed_constants;

fn main() {
    cli::run();
}
