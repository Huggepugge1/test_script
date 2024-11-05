use crate::lexer::Token;

#[derive(Debug)]
pub enum ParseErrorType {
    Warning,
    Error,
}

impl std::fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseErrorType::Warning => write!(f, "Warning"),
            ParseErrorType::Error => write!(f, "Error"),
        }
    }
}

pub struct ParseError {
    r#type: ParseErrorType,
    token: Token,
    string: String,
    hint: Option<String>,
}

impl ParseError {
    pub fn new(
        r#type: ParseErrorType,
        token: Token,
        string: impl Into<String>,
        hint: Option<impl Into<String>>,
    ) -> ParseError {
        ParseError {
            r#type,
            token,
            string: string.into(),
            hint: hint.map(|s| s.into()),
        }
    }

    pub fn print(&self) {
        let hint = if let Some(hint) = &self.hint {
            format!("\nHint: {}\n", hint)
        } else {
            "\n".to_string()
        };
        eprintln!(
            "{}: {} {:?}, {}:{}{}",
            self.r#type, self.string, self.token.value, self.token.line, self.token.column, hint
        );
    }
}
