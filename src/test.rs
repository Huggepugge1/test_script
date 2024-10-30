use crate::{cli, interpreter, lexer, parser};

pub fn run(args: cli::Args) {
    let contents = std::fs::read_to_string(args.file).expect("File not found");

    let tokens = lexer::tokenize(contents);
    let program = parser::parse(tokens, args.max_size);
    interpreter::interpret(program);
}
