use crate::{cli, interpreter, lexer, parser};

pub fn run(args: cli::Args) {
    let contents = std::fs::read_to_string(args.file).expect("File not found");

    let tokens = lexer::tokenize(contents);
    let mut parser = parser::Parser::new(tokens, args.max_size);
    let program = parser.parse();
    match program {
        Ok(program) => interpreter::Interpreter::new(program).interpret(),
        Err(_) => (),
    }
}
