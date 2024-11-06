use crate::{cli, interpreter, lexer, parser};

pub fn run(args: cli::Args) {
    let contents = std::fs::read_to_string(args.file).expect("File not found");

    let tokens = lexer::tokenize(contents);
    let program = parser::Parser::new(tokens, args.max_size).parse();
    match program {
        Ok(program) => interpreter::Interpreter::new(program).interpret(),
        Err(_) => (),
    }
}
