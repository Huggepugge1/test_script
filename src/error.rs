use crate::instruction::InstructionResult;
use crate::r#type::Type;
use crate::token::{PrintStyle, Token, TokenType};
use crate::variable::{SnakeCase, Variable};

use colored::Colorize;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LexerError<'a> {
    FileNotFound(&'a PathBuf),
    FileExtensionNotTesc(&'a PathBuf),
    PermissionDenied(&'a PathBuf),

    Unknown(&'a PathBuf, std::io::Error),
}

impl<'a> LexerError<'a> {
    pub fn print(&self) {
        match &self {
            LexerError::FileNotFound(path) => {
                let error_msg = format!("File not found: `{}`", path.display());
                eprintln!("{}{}\n", "error: ".bright_red(), error_msg);
            }
            LexerError::FileExtensionNotTesc(path) => {
                let error_msg = format!("File extension must be `tesc`: `{}`", path.display());
                eprintln!(
                    "{}{}\n\
                     {}{} change this to `tesc`\n",
                    "error: ".bright_red(),
                    error_msg,
                    " ".repeat(
                        "error: ".len() + error_msg.len()
                            - 1
                            - path.extension().unwrap().to_string_lossy().len()
                    ),
                    "^".repeat(path.extension().unwrap().to_string_lossy().len())
                        .bright_yellow()
                );
            }
            LexerError::PermissionDenied(path) => {
                let error_msg = format!("Permission denied: `{}`", path.display());
                eprintln!("{}{}\n", "error: ".bright_red(), error_msg);
            }
            LexerError::Unknown(path, e) => {
                let error_msg = format!("Unknown error: `{}`", path.display());
                eprintln!("{}{}\n", "error: ".bright_red(), error_msg);
                eprintln!("Rust error: {}\n", e);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorType {
    UnexpectedToken(TokenType),

    UnexpectedEndOfFile,
    UnclosedDelimiter(TokenType),

    MismatchedType {
        expected: Vec<Type>,
        actual: Type,
    },
    MismatchedArguments {
        expected: usize,
        actual: usize,
    },
    MismatchedTokenType {
        expected: TokenType,
        actual: TokenType,
    },

    GlobalScope(TokenType),

    TypeCast {
        from: Type,
        to: Type,
    },

    RegexError,

    IdentifierNotDefined(String),

    ConstantReassignment(Variable),

    VaribleTypeAnnotation,

    None,
}

impl std::fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseErrorType::UnexpectedToken(token) => {
                let token = match token {
                    TokenType::Semicolon
                    | TokenType::OpenParen
                    | TokenType::CloseParen
                    | TokenType::OpenBlock
                    | TokenType::CloseBlock
                    | TokenType::Colon
                    | TokenType::Type { .. } => format!("`{token}`"),
                    _ => format!("{token}"),
                };
                write!(f, "Unexpected token: {}", token)
            }

            ParseErrorType::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
            ParseErrorType::UnclosedDelimiter(token) => {
                let token = match token {
                    TokenType::OpenParen
                    | TokenType::CloseParen
                    | TokenType::OpenBlock
                    | TokenType::CloseBlock => format!("`{token}`"),
                    _ => unreachable!(),
                };
                write!(f, "Unclosed delimiter: {}", token)
            }

            ParseErrorType::MismatchedType { expected, actual } => match expected.len() {
                1 => write!(
                    f,
                    "Type error: Expected `{}`, found `{}`",
                    expected[0], actual
                ),
                _ => write!(
                    f,
                    "Type error: Expected one of {}, found `{}`",
                    expected
                        .into_iter()
                        .map(|r#type| format!("`{type}`"))
                        .collect::<Vec<String>>()
                        .join(", "),
                    actual
                ),
            },

            ParseErrorType::MismatchedTokenType { expected, actual } => {
                let expected = match expected {
                    TokenType::Semicolon
                    | TokenType::OpenParen
                    | TokenType::CloseParen
                    | TokenType::OpenBlock
                    | TokenType::CloseBlock
                    | TokenType::Colon
                    | TokenType::Type { .. } => format!("`{expected}`"),
                    _ => format!("{expected}"),
                };
                write!(f, "Expected {expected}, found {actual}")
            }

            ParseErrorType::MismatchedArguments { expected, actual } => {
                write!(f, "Expected {} arguments, found {}", expected, actual)
            }

            ParseErrorType::GlobalScope(token) => {
                write!(f, "Unexpected token in global scope: {token}")
            }

            ParseErrorType::TypeCast { from, to } => {
                write!(f, "Cannot cast `{from}` to `{to}`")
            }

            ParseErrorType::RegexError => write!(f, "Regex syntax not supported"),

            ParseErrorType::IdentifierNotDefined(identifier) => {
                write!(f, "Identifier `{identifier}` not defined")
            }
            ParseErrorType::ConstantReassignment(constant) => {
                write!(f, "Cannot reassign constant `{}`", constant.name)
            }
            ParseErrorType::VaribleTypeAnnotation => {
                write!(f, "Type annotations are required")
            }

            ParseErrorType::None => write!(f, ""),
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub r#type: ParseErrorType,
    token: Token,
}

impl ParseError {
    pub fn new(r#type: ParseErrorType, token: Token) -> ParseError {
        ParseError { r#type, token }
    }

    pub fn none() -> ParseError {
        ParseError {
            r#type: ParseErrorType::None,
            token: Token::none(),
        }
    }

    pub fn print(&self) {
        if self.r#type == ParseErrorType::None {
            return;
        }

        match &self.r#type {
            ParseErrorType::MismatchedTokenType {
                expected: TokenType::Semicolon,
                actual: _actual,
            } => match &self.token.last_token {
                Some(last_token) => {
                    eprintln!(
                        "{}{}              \n\
                         In: {}:{}:{}      \n\
                         {}                \n\
                         {}                \n",
                        "error: ".bright_red(),
                        self.r#type,
                        last_token.file,
                        last_token.row,
                        last_token.column + last_token.len(),
                        last_token
                            .insert_tokens(vec![TokenType::Semicolon], "add a semicolon here"),
                        self.token.as_string(PrintStyle::Help("unexpected token")),
                    )
                }
                None => {
                    eprintln!(
                        "{}{}              \n\
                         In: {}:{}:{}      \n\
                         {}                \n",
                        "error: ".bright_red(),
                        self.r#type,
                        self.token.file,
                        self.token.row,
                        self.token.column,
                        self.token.as_string(PrintStyle::Error),
                    )
                }
            },
            ParseErrorType::ConstantReassignment(var) => {
                eprintln!(
                    "{}{}              \n\
                     In: {}:{}:{}      \n\
                     {}                \n\
                     {}                \n",
                    "error: ".bright_red(),
                    self.r#type,
                    self.token.file,
                    var.declaration_token.row,
                    var.declaration_token.column,
                    var.declaration_token
                        .as_string(PrintStyle::Help("consider changing to `let`")),
                    self.token.as_string(PrintStyle::Error),
                )
            }

            ParseErrorType::VaribleTypeAnnotation => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {}                \n\
                 {}                \n",
                "error: ".bright_red(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.insert_tokens(
                    vec![TokenType::Colon, TokenType::Type { value: Type::Any }],
                    "add a type annotation here"
                ),
                self.token.as_string(PrintStyle::Error),
            ),
            _ => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {}                \n",
                "error: ".bright_red(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Error),
            ),
        }
    }
}

pub enum ParseWarningType<'a> {
    TrailingSemicolon,
    EmptyBlock,

    UnusedValue,
    UnusedVariable,
    VariableNotRead,
    VariableNeverReAssigned,

    ConstantNotUpperCase(String),
    VariableNotSnakeCase(String),

    SelfAssignment,

    NoBlock(&'a Token),

    MagicLiteral(Type),
}

pub struct ParseWarning<'a> {
    pub r#type: ParseWarningType<'a>,
    pub token: Token,
}

impl<'a> std::fmt::Display for ParseWarningType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseWarningType::TrailingSemicolon => write!(f, "Trailing semicolon"),
            ParseWarningType::EmptyBlock => write!(f, "Empty block"),
            ParseWarningType::UnusedValue => write!(f, "Unused value"),
            ParseWarningType::UnusedVariable => write!(f, "Unused variable"),
            ParseWarningType::VariableNotRead => {
                write!(f, "Variable is not read after assignment")
            }
            ParseWarningType::VariableNeverReAssigned => {
                write!(f, "Variable is never reassigned")
            }
            ParseWarningType::ConstantNotUpperCase(_identifier) => {
                write!(f, "Constants should be in UPPER_SNAKE_CASE")
            }
            ParseWarningType::VariableNotSnakeCase(_identifier) => {
                write!(f, "Variables should be in snake_case")
            }
            ParseWarningType::SelfAssignment => write!(f, "Assignment without effect"),
            ParseWarningType::NoBlock(_) => write!(f, "A block should be used here"),
            ParseWarningType::MagicLiteral(r#type) => write!(f, "Magic {type} detected"),
        }
    }
}

impl<'a> ParseWarning<'a> {
    pub fn new(r#type: ParseWarningType, token: Token) -> ParseWarning {
        ParseWarning { r#type, token }
    }

    pub fn print(&self, disable_warnings: bool) {
        if disable_warnings {
            return;
        }
        match &self.r#type {
            ParseWarningType::TrailingSemicolon => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {} {}             \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Warning),
                "remove this semicolon".bright_yellow(),
            ),
            ParseWarningType::EmptyBlock => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {} {}             \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Warning),
                "remove this block".bright_yellow(),
            ),
            ParseWarningType::UnusedValue => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {}                \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Warning),
            ),
            ParseWarningType::UnusedVariable => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {} {}             \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Warning),
                "prefix with `_` to suppress this warning".bright_yellow(),
            ),
            ParseWarningType::VariableNotRead => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {}                \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Warning),
            ),
            ParseWarningType::VariableNeverReAssigned => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {} {}             \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Warning),
                "consider changing to `const`".bright_yellow(),
            ),
            ParseWarningType::ConstantNotUpperCase(identifier) => {
                let new_name = identifier.to_upper_snake_case();
                eprintln!(
                    "{}{}              \n\
                     In: {}:{}:{}      \n\
                     {} {}             \n",
                    "warning: ".bright_yellow(),
                    self.r#type,
                    self.token.file,
                    self.token.row,
                    self.token.column,
                    self.token.as_string(PrintStyle::Warning),
                    format!("consider changing the name to {new_name}").bright_yellow(),
                )
            }
            ParseWarningType::VariableNotSnakeCase(identifier) => {
                let new_name = identifier.to_snake_case();
                eprintln!(
                    "{}{}              \n\
                     In: {}:{}:{}      \n\
                     {} {}             \n",
                    "warning: ".bright_yellow(),
                    self.r#type,
                    self.token.file,
                    self.token.row,
                    self.token.column,
                    self.token.as_string(PrintStyle::Warning),
                    format!("consider changing the name to {new_name}").bright_yellow(),
                )
            }
            ParseWarningType::SelfAssignment => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {}                \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Warning),
            ),
            ParseWarningType::NoBlock(token) => match &self.token.last_token {
                Some(last_token) => {
                    eprintln!(
                        "{}{}              \n\
                             In: {}:{}:{}      \n\
                             {}                \n",
                        "warning: ".bright_yellow(),
                        self.r#type,
                        last_token.file,
                        last_token.row,
                        last_token.column + last_token.len(),
                        last_token.wrap_in_block(token),
                    )
                }
                _ => unreachable!(),
            },
            ParseWarningType::MagicLiteral(_type) => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {} {}             \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.file,
                self.token.row,
                self.token.column,
                self.token.as_string(PrintStyle::Warning),
                "consider using a named constant".bright_yellow(),
            ),
        }
    }
}

pub enum InterpreterError {
    TypeCast {
        result: InstructionResult,
        from: Type,
        to: Type,
    },
    TestFailed(String),
}

impl InterpreterError {
    pub fn print(&self) {
        match &self {
            InterpreterError::TypeCast { result, from, to } => {
                eprintln!("Type cast error: Failed to cast `{from} {result}` to `{to}`\n");
            }
            InterpreterError::TestFailed(message) => {
                eprintln!("Test failed: {message}");
            }
        }
    }
}
