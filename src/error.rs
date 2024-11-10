use crate::token::{Token, TokenType};

#[derive(Debug)]
pub enum ParseErrorType {
    UnexpectedToken,
    UnexpectedEndOfFile,
    MismatchedType(TokenType, TokenType),
    RegexError,
    VariableNotDefined,

    TestError,

    NotImplemented,
}

impl std::fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseErrorType::UnexpectedToken => write!(f, "Unexpected token"),
            ParseErrorType::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
            ParseErrorType::MismatchedType(type1, type2) => {
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

    pub fn print(&self) {
        eprintln!(
            "Error: {} {:?}, {}:{}\n\
             Hint: {}\n",
            self.r#type, self.token.value, self.token.line, self.token.column, self.hint
        );
    }
}

pub enum ParseWarningType {
    UnusedLiteral,
    ExtraSemicolon,
}

pub struct ParseWarning {
    pub r#type: ParseWarningType,
    pub token: Token,
    pub hint: String,
}

impl std::fmt::Display for ParseWarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseWarningType::UnusedLiteral => write!(f, "Unused literal"),
            ParseWarningType::ExtraSemicolon => write!(f, "Extra semicolon"),
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

    pub fn print(&self) {
        eprintln!(
            "Warning: {} {:?}, {}:{}\n\
             Hint: {}\n",
            self.r#type, self.token.value, self.token.line, self.token.column, self.hint
        );
    }
}

pub enum InterpreterErrorType {
    IncompatibleTypes,
}

impl std::fmt::Display for InterpreterErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpreterErrorType::IncompatibleTypes => write!(f, "Incompatible types"),
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
