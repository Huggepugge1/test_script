use crate::interpreter::InstructionResult;
use crate::r#type::Type;
use crate::token::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorType {
    UnexpectedToken,
    UnexpectedEndOfFile,

    MismatchedType {
        expected: Type,
        actual: Type,
    },
    MismatchedTypeBinary {
        expected_left: Type,
        actual_left: Type,
        expected_right: Type,
        actual_right: Type,
    },
    MismatchedTokenType(TokenType, TokenType),

    RegexError,

    VariableNotDefined,

    TestError,

    NotImplemented,

    None,
}

impl std::fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseErrorType::UnexpectedToken => write!(f, "Unexpected token"),
            ParseErrorType::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),

            ParseErrorType::MismatchedType { expected, actual } => {
                write!(f, "Mismatched type: Expected {}, got {}", expected, actual)
            }
            ParseErrorType::MismatchedTypeBinary {
                expected_left,
                actual_left,
                expected_right,
                actual_right,
            } => {
                write!(
                    f,
                    "Mismatched types: Expected `{}` and `{}`, got `{}` and `{}`",
                    expected_left, expected_right, actual_left, actual_right
                )
            }
            ParseErrorType::MismatchedTokenType(type1, type2) => {
                write!(
                    f,
                    "Mismatched token type: Expected {}, got {}",
                    type1, type2
                )
            }

            ParseErrorType::RegexError => write!(f, "Regex error"),
            ParseErrorType::VariableNotDefined => write!(f, "Variable not defined"),

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
    hint: String,
}

impl ParseError {
    pub fn new(r#type: ParseErrorType, token: Token, hint: impl Into<String>) -> ParseError {
        ParseError {
            r#type,
            token,
            hint: hint.into(),
        }
    }

    pub fn none() -> ParseError {
        ParseError {
            r#type: ParseErrorType::None,
            token: Token::none(),
            hint: String::new(),
        }
    }

    pub fn print(&self) {
        if self.r#type == ParseErrorType::None {
            return;
        }
        eprintln!(
            "Error: {} {:?}, {}:{}\n\
             Hint: {}\n",
            self.r#type, self.token.value, self.token.line, self.token.column, self.hint
        );
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
            "Warning: {}, {}:{}\n\
             Hint: {}\n",
            self.r#type, self.token.line, self.token.column, self.hint
        );
    }
}

pub enum InterpreterErrorType {
    IncompatibleTypes(InstructionResult, InstructionResult),
    IncompatibleTypesBinary(
        InstructionResult,
        InstructionResult,
        InstructionResult,
        InstructionResult,
    ),
}

impl std::fmt::Display for InterpreterErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpreterErrorType::IncompatibleTypes(expected, actual) => {
                write!(f, "Expected type: {}, got: {}", expected, actual)
            }
            InterpreterErrorType::IncompatibleTypesBinary(
                left_expected,
                left,
                right_expected,
                right,
            ) => write!(
                f,
                "Expected types: {} {} got: {} {}",
                left_expected, right_expected, left, right
            ),
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
