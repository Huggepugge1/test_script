use crate::error::LexerError;
use crate::exitcode::ExitCode;
use crate::{cli, interpreter, lexer, parser, type_checker};

use std::io::ErrorKind;

pub fn run(args: cli::Args) {
    let mut contents = match std::fs::read_to_string(args.file.clone()) {
        Ok(contents) => contents,
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => {
                LexerError::PermissionDenied(&args.file).print();
                std::process::exit(ExitCode::SourcePermissionDenied as i32);
            }
            _ => {
                LexerError::Unknown(&args.file, e).print();
                std::process::exit(ExitCode::Unknown as i32);
            }
        },
    };
    let tokens = lexer::Lexer::new(&mut contents, args.clone()).tokenize();

    let program = parser::Parser::new(tokens, args.clone()).parse();

    let type_check = match &program {
        Ok(program) => type_checker::TypeChecker::new(program.clone(), args.clone()).check(),
        Err(program) => type_checker::TypeChecker::new(program.clone(), args.clone()).check(),
    };

    if let Ok(program) = program {
        if type_check.is_ok() {
            interpreter::Interpreter::new(program, args.clone()).interpret();
        }
    }
}
