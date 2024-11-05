use crate::{cli, interpreter, lexer, parser};

pub fn run(args: cli::Args) {
    let contents = std::fs::read_to_string(args.file).expect("File not found");

    let mut tokens = lexer::tokenize(contents);
    println!("{:#?}", tokens);
    let program = parser::parse(&mut tokens, args.max_size);
    match program {
        Ok(program) => interpreter::interpret(program),
        Err(()) => (),
    }
}
