use crate::{cli, interpreter, lexer, parser, type_checker};

pub fn run(args: cli::Args) {
    let mut contents = std::fs::read_to_string(args.file.clone()).unwrap();
    let tokens = lexer::Lexer::new(&mut contents, args.clone()).tokenize();

    let program = parser::Parser::new(tokens, args.clone()).parse();
    let type_check = match &program {
        Ok(program) => type_checker::TypeChecker::new(program.clone(), args.clone()).check(),
        Err(program) => type_checker::TypeChecker::new(program.clone(), args.clone()).check(),
    };

    match program {
        Ok(program) => match type_check {
            Ok(_) => interpreter::Interpreter::new(program).interpret(),
            Err(_) => (),
        },
        Err(_) => (),
    }
}
