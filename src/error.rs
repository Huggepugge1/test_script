use crate::token::{Token, TokenType};

pub enum ParseErrorType {
    Warning,
    UnexpectedToken,
    UnexpectedEndOfFile,
    MissingSemicolon,
    MismatchedType(TokenType, TokenType),
    RegexError,
    NotImplemented,
}

impl std::fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseErrorType::Warning => write!(f, "Warning"),
            ParseErrorType::UnexpectedToken => write!(f, "Unexpected token"),
            ParseErrorType::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
            ParseErrorType::MissingSemicolon => write!(f, ""),
            ParseErrorType::MismatchedType(type1, type2) => {
                write!(f, "Mismatched type: {} got {}", type1, type2)
            }
            ParseErrorType::NotImplemented => write!(f, "Not implemented"),
            ParseErrorType::RegexError => write!(f, "Regex error"),
        }
    }
}

pub struct ParseError {
    r#type: ParseErrorType,
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
            "Error: {}\n\
             {:?}, {}:{}\n\
             {}",
            self.r#type, self.token.value, self.token.line, self.token.column, self.hint
        );
    }
}

pub enum ParseWarningType {
    UnusedLiteral,
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
            "Warning: {}\n\
             {:?}, {}:{}\n\
             {}",
            self.r#type, self.token.value, self.token.line, self.token.column, self.hint
        );
    }
}
