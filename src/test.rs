use crate::{cli, interpreter, lexer, parser, type_checker};

pub fn run(args: cli::Args) {
    let mut contents = std::fs::read_to_string(args.file.clone()).expect("File not found");

    let tokens = lexer::Lexer::new(&mut contents).tokenize();

    let program = parser::Parser::new(tokens, args.clone()).parse();

    match program {
        Ok(program) => match type_checker::TypeChecker::new(program.clone(), args).check() {
            Ok(()) => interpreter::Interpreter::new(program).interpret(),
            Err(_) => (),
        },
        Err(_) => (),
    }
}
