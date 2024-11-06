#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Literal,
    Keyword,
    BuiltIn,
    Identifier,
    OpenBlock,
    CloseBlock,
    OpenParen,
    CloseParen,
    Semicolon,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::Literal => write!(f, "Literal"),
            TokenType::Keyword => write!(f, "Keyword"),
            TokenType::BuiltIn => write!(f, "BuiltIn"),
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::OpenBlock => write!(f, "OpenBlock"),
            TokenType::CloseBlock => write!(f, "CloseBlock"),
            TokenType::OpenParen => write!(f, "OpenParen"),
            TokenType::CloseParen => write!(f, "CloseParen"),
            TokenType::Semicolon => write!(f, "Semicolon"),
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

    pub fn next(&mut self) -> Option<Token> {
        if !self.started {
            self.started = true;
        } else {
            self.index += 1;
        }
        let result = self.current();
        result
    }

    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub fn advance_to_next_instruction(&mut self) {
        while let Some(token) = self.next() {
            if token.r#type == TokenType::Semicolon {
                break;
            }
        }
        self.next();
    }
}
