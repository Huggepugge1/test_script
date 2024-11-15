use crate::interpreter::InstructionResult;
use crate::r#type::Type;
use crate::token::{PrintStyle, Token, TokenType};
use crate::variable::Variable;

use colored::Colorize;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorType {
    UnexpectedToken(TokenType),

    UnexpectedEndOfFile,
    UnclosedDelimiter(TokenType),

    MismatchedType {
        expected: Vec<Type>,
        actual: Type,
    },
    MismatchedTokenType {
        expected: TokenType,
        actual: TokenType,
    },

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
                                           \n\
                         {}                \n",
                        "error: ".bright_red(),
                        self.r#type,
                        self.token.file,
                        last_token.row,
                        last_token.column + last_token.len(),
                        last_token
                            .insert_tokens(vec![TokenType::Semicolon], "add a semicolon here"),
                        self.token.as_string(PrintStyle::Error),
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
                                   \n\
                 {}                \n",
                    "error: ".bright_red(),
                    self.r#type,
                    self.token.file,
                    var.token.row,
                    var.token.column,
                    var.token
                        .as_string(PrintStyle::Help("consider changing to `let`")),
                    self.token.as_string(PrintStyle::Error),
                )
            }

            ParseErrorType::VaribleTypeAnnotation => eprintln!(
                "{}{}              \n\
                 In: {}:{}:{}      \n\
                 {}                \n\
                                   \n\
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

pub enum ParseWarningType {
    TrailingSemicolon,
    EmptyBlock,

    UnusedValue,
    UnusedVariable,
}

pub struct ParseWarning {
    pub r#type: ParseWarningType,
    pub token: Token,
}

impl std::fmt::Display for ParseWarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseWarningType::TrailingSemicolon => write!(f, "Trailing semicolon"),
            ParseWarningType::EmptyBlock => write!(f, "Empty block"),
            ParseWarningType::UnusedValue => write!(f, "Unused value"),
            ParseWarningType::UnusedVariable => write!(f, "Unused variable"),
        }
    }
}

impl ParseWarning {
    pub fn new(r#type: ParseWarningType, token: Token) -> ParseWarning {
        ParseWarning { r#type, token }
    }

    pub fn print(&self, disable_warnings: bool) {
        if disable_warnings {
            return;
        }
        match self.r#type {
            ParseWarningType::TrailingSemicolon => eprintln!(
                "{}{}              \n\
                                   \n\
                 {} {}             \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.as_string(PrintStyle::Warning),
                "remove this semicolon".bright_yellow(),
            ),
            ParseWarningType::EmptyBlock => eprintln!(
                "{}{}              \n\
                                   \n\
                 {} {}             \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.as_string(PrintStyle::Warning),
                "remove this block".bright_yellow(),
            ),
            ParseWarningType::UnusedValue => eprintln!(
                "{}{}              \n\
                                   \n\
                 {}                \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.as_string(PrintStyle::Warning),
            ),
            ParseWarningType::UnusedVariable => eprintln!(
                "{}{}              \n\
                                   \n\
                 {}                \n",
                "warning: ".bright_yellow(),
                self.r#type,
                self.token.as_string(PrintStyle::Warning),
            ),
        }
    }
}

pub enum InterpreterErrorType {
    TypeCastError {
        result: InstructionResult,
        from: Type,
        to: Type,
    },
}

impl std::fmt::Display for InterpreterErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpreterErrorType::TypeCastError { result, from, to } => {
                write!(
                    f,
                    "Type cast error: Could not cast {from} \"{result}\" to `{to}`",
                )
            }
        }
    }
}

pub struct InterpreterError {
    pub r#type: InterpreterErrorType,
    hint: String,
}

impl InterpreterError {
    pub fn new(r#type: InterpreterErrorType, hint: impl Into<String>) -> InterpreterError {
        InterpreterError {
            r#type,
            hint: hint.into(),
        }
    }

    pub fn print(&self) {
        eprintln!(
            "Error while executing: {}\n\
                   Hint: {}\n",
            self.r#type, self.hint
        );
    }
}
