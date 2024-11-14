use crate::interpreter::InstructionResult;
use crate::r#type::Type;
use crate::token::{PrintStyle, Token, TokenType};

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

    VariableNotDefined,
    ConstantReassignment(String),

    TestError,

    NotImplemented,

    None,
}

impl std::fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseErrorType::UnexpectedToken(token) => {
                write!(f, "Unexpected token: {}", token)
            }

            ParseErrorType::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
            ParseErrorType::UnclosedDelimiter(token) => {
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
                write!(
                    f,
                    "Mismatched token type: Expected {expected}, got {actual}",
                )
            }

            ParseErrorType::TypeCast { from, to } => {
                write!(f, "Type cast error: Cannot cast {} to {}", from, to)
            }

            ParseErrorType::RegexError => write!(f, "Regex error"),

            ParseErrorType::VariableNotDefined => write!(f, "Variable not defined"),
            ParseErrorType::ConstantReassignment(constant) => {
                write!(f, "Cannot reassign constant `{constant}`")
            }

            ParseErrorType::TestError => write!(f, "Test error"),

            ParseErrorType::NotImplemented => write!(f, "Not implemented"),

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
                        self.token.row,
                        self.token.column,
                        last_token.expected_semicolon(),
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
}

pub struct ParseWarning {
    pub r#type: ParseWarningType,
    pub token: Token,
    pub hint: String,
}

impl std::fmt::Display for ParseWarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseWarningType::TrailingSemicolon => write!(f, "Trailing semicolon"),
        }
    }
}

impl ParseWarning {
    pub fn new(r#type: ParseWarningType, token: Token, hint: impl Into<String>) -> ParseWarning {
        ParseWarning {
            r#type,
            token,
            hint: hint.into(),
        }
    }

    pub fn print(&self, disable_warnings: bool) {
        if disable_warnings {
            return;
        }
        eprintln!(
            "{}{}              \n\
                               \n\
             {} {}              \n",
            "warning: ".bright_yellow(),
            self.r#type,
            self.token.as_string(PrintStyle::Warning),
            "remove this semicolon".bright_yellow(),
        );
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
