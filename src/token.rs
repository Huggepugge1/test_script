#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    StringLiteral,
    RegexLiteral,
    IntegerLiteral,
    BooleanLiteral,

    Keyword,
    BuiltIn,

    Type,
    Colon,

    Identifier,

    OpenBlock,
    CloseBlock,

    OpenParen,
    CloseParen,

    TypeCast,
    AssignmentOperator,

    UnaryOperator,
    BinaryOperator,

    Semicolon,

    None,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::StringLiteral => write!(f, "String literal"),
            TokenType::RegexLiteral => write!(f, "Regex literal"),
            TokenType::IntegerLiteral => write!(f, "Integer literal"),
            TokenType::BooleanLiteral => write!(f, "Boolean literal"),

            TokenType::Keyword => write!(f, "Keyword"),
            TokenType::BuiltIn => write!(f, "BuiltIn"),

            TokenType::Type => write!(f, "Type"),
            TokenType::Colon => write!(f, "Colon"),

            TokenType::Identifier => write!(f, "Identifier"),

            TokenType::OpenBlock => write!(f, "OpenBlock"),
            TokenType::CloseBlock => write!(f, "CloseBlock"),
            TokenType::OpenParen => write!(f, "OpenParen"),
            TokenType::CloseParen => write!(f, "CloseParen"),

            TokenType::TypeCast => write!(f, "Type cast"),
            TokenType::AssignmentOperator => write!(f, "Assignment operator"),

            TokenType::UnaryOperator => write!(f, "Unary operator"),
            TokenType::BinaryOperator => write!(f, "Binary operator"),

            TokenType::Semicolon => write!(f, "Semicolon"),

            TokenType::None => write!(f, ""),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub value: String,
    pub line: u32,
    pub column: u32,
}

impl Token {
    pub fn new(r#type: TokenType, value: &String, line: u32, column: u32) -> Token {
        Token {
            r#type,
            value: value.to_string(),
            line,
            column,
        }
    }

    pub fn none() -> Token {
        Token {
            r#type: TokenType::None,
            value: String::new(),
            line: 0,
            column: 0,
        }
    }

    pub fn binary_operator(&self) -> bool {
        self.r#type == TokenType::BinaryOperator
            || self.r#type == TokenType::AssignmentOperator
            || self.r#type == TokenType::TypeCast
    }
}

#[derive(Debug, Clone)]
pub struct TokenCollection {
    pub tokens: Vec<Token>,
    pub index: usize,
    pub started: bool,
}

impl TokenCollection {
    pub fn new(tokens: Vec<Token>) -> TokenCollection {
        TokenCollection {
            tokens,
            index: 0,
            started: false,
        }
    }

    pub fn current(&self) -> Option<Token> {
        if self.index >= self.tokens.len() {
            None
        } else if self.started {
            Some(self.tokens[self.index].clone())
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<Token> {
        if (self.index + 1) >= self.tokens.len() {
            None
        } else if self.started {
            Some(self.tokens[self.index + 1].clone())
        } else {
            Some(self.tokens[self.index].clone())
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        if (self.index + 1) >= self.tokens.len() {
            return None;
        }
        if !self.started {
            self.started = true;
        } else {
            self.index += 1;
        }
        let result = self.current();
        result
    }

    pub fn back(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn advance_to_next_instruction(&mut self) {
        while let Some(token) = self.next() {
            if token.r#type == TokenType::Semicolon || token.r#type == TokenType::CloseBlock {
                break;
            } else if token.r#type == TokenType::OpenBlock {
                self.back();
                break;
            }
        }
    }
}
