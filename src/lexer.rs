use crate::token::{Token, TokenCollection, TokenType};

pub struct Lexer<'a> {
    lines: Vec<String>,
    contents: std::iter::Peekable<std::str::Chars<'a>>,

    row: u32,
    column: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(contents: &'a mut String) -> Lexer<'a> {
        let lines = contents.lines().map(|s| s.to_string()).collect();
        let contents = contents.chars().peekable().to_owned().clone();

        let row = 1;
        let column = 1;

        Lexer {
            lines,
            contents,
            row,
            column,
        }
    }

    fn make_token(&self, r#type: TokenType, value: &str) -> Token {
        Token {
            r#type,
            value: value.to_string(),
            row: self.row,
            column: self.column,
            line: self.get_line(self.row),
        }
    }

    fn get_line(&self, line: u32) -> String {
        self.lines[line as usize - 1].clone()
    }

    fn identifier_type(&mut self, value: &String) -> TokenType {
        match value.as_str() {
            "for" | "let" | "const" | "if" | "else" => TokenType::Keyword,
            "string" | "regex" | "int" | "bool" => TokenType::Type,
            "true" | "false" => TokenType::BooleanLiteral,
            "in" => TokenType::AssignmentOperator,
            "as" => TokenType::TypeCast,
            "input" | "output" | "print" | "println" => TokenType::BuiltIn,
            _ => TokenType::Identifier,
        }
    }

    pub fn tokenize_identifier(&mut self) -> Token {
        let mut length = 0;
        let mut current = String::new();

        while let Some(next) = self.contents.peek() {
            if !(next.is_alphanumeric() || *next == '_') {
                break;
            }
            current.push(*next);
            self.contents.next();
            length += 1;
        }

        let token_type = self.identifier_type(&current);
        let token = self.make_token(token_type, &current);
        self.column += length;
        token
    }

    pub fn tokenize_string_literal(&mut self) -> Token {
        let mut new_row = self.row;
        let mut new_column = self.column + 1;
        let mut current = String::from("\"");

        self.contents.next();

        while let Some(next) = self.contents.peek() {
            if *next == '\n' {
                new_row += 1;
                new_column = 1;
            }
            current.push(*next);
            new_column += 1;
            if *next == '"' {
                break;
            }
            self.contents.next();
        }

        self.contents.next();

        let token = self.make_token(TokenType::StringLiteral, &current);
        self.row = new_row;
        self.column = new_column;
        token
    }

    pub fn tokenize_regex_literal(&mut self) -> Token {
        let mut new_row = self.row;
        let mut new_column = self.column + 1;
        let mut current = String::new();

        self.contents.next();

        while let Some(next) = self.contents.peek() {
            if *next == '\n' {
                new_row += 1;
                new_column = 1;
            }
            current.push(*next);
            new_column += 1;
            if *next == '`' {
                break;
            }
            self.contents.next();
        }

        self.contents.next();

        let token = self.make_token(TokenType::RegexLiteral, &current);
        self.row = new_row;
        self.column = new_column;
        token
    }

    pub fn tokenize_integer_literal(&mut self) -> Token {
        let mut length = 0;
        let mut current = String::new();
        while let Some(next) = self.contents.peek() {
            if !next.is_ascii_digit() {
                break;
            }
            current.push(*next);
            self.contents.next();
            length += 1;
        }

        let token = self.make_token(TokenType::IntegerLiteral, &current);
        self.column += length;
        token
    }

    pub fn tokenize(&mut self) -> TokenCollection {
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(c) = self.contents.peek() {
            match c {
                '{' => tokens.push(self.make_token(TokenType::OpenBlock, &"{")),
                '}' => tokens.push(self.make_token(TokenType::CloseBlock, &"}")),
                '(' => tokens.push(self.make_token(TokenType::OpenParen, &"(")),
                ')' => tokens.push(self.make_token(TokenType::CloseParen, &")")),
                ';' => tokens.push(self.make_token(TokenType::Semicolon, &";")),
                '+' => tokens.push(self.make_token(TokenType::BinaryOperator, &"+")),
                '-' => tokens.push(self.make_token(TokenType::BinaryOperator, &"-")),
                '*' => tokens.push(self.make_token(TokenType::BinaryOperator, &"*")),
                '/' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('/') = self.contents.peek() {
                        while let Some(next) = self.contents.next() {
                            if next == '\n' {
                                break;
                            }
                            length += 1;
                        }
                    } else {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &"/"));
                    }
                    self.column += length;
                    continue;
                }
                ':' => tokens.push(self.make_token(TokenType::Colon, &":")),
                '<' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('=') = self.contents.peek() {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &"<="));
                        length += 1;
                        self.contents.next();
                    } else {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &"<"));
                    }
                    self.column += length;
                    continue;
                }
                '>' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('=') = self.contents.peek() {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &">="));
                        length += 1;
                        self.contents.next();
                    } else {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &">"));
                    }
                    self.column += length;
                }
                '=' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('=') = self.contents.peek() {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &"=="));
                        length += 1;
                        self.contents.next();
                    } else {
                        tokens.push(self.make_token(TokenType::AssignmentOperator, &"="));
                    }
                    self.column += length;
                    continue;
                }
                '!' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('=') = self.contents.peek() {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &"!="));
                        length += 1;
                        self.contents.next();
                    } else {
                        tokens.push(self.make_token(TokenType::UnaryOperator, &"!"));
                    }
                    self.column += length;
                    continue;
                }
                '&' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('&') = self.contents.peek() {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &"&&"));
                        length += 1;
                        self.contents.next();
                    } else {
                        panic!("Unexpected character: \"&\"");
                    }
                    self.column += length;
                    continue;
                }
                '|' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('|') = self.contents.peek() {
                        tokens.push(self.make_token(TokenType::BinaryOperator, &"||"));
                        length += 1;
                        self.contents.next();
                    } else {
                        panic!("Unexpected character: \"|\"");
                    }
                    self.column += length;
                    continue;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    tokens.push(self.tokenize_identifier());
                    continue;
                }
                '"' => {
                    tokens.push(self.tokenize_string_literal());
                    continue;
                }
                '`' => {
                    tokens.push(self.tokenize_regex_literal());
                    continue;
                }
                '0'..='9' => {
                    tokens.push(self.tokenize_integer_literal());
                    continue;
                }
                '\n' => {
                    self.row += 1;
                    self.column = 1;
                    self.contents.next();
                    continue;
                }
                ' ' | '\t' => (),
                _ => panic!("Unexpected character: \"{}\"", c),
            }
            self.column += 1;
            self.contents.next();
        }

        TokenCollection::new(tokens)
    }
}
