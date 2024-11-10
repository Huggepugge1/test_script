use crate::token::{Token, TokenCollection, TokenType};

pub struct Lexer<'a> {
    contents: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(contents: &'a mut String) -> Lexer<'a> {
        let peekable = contents.chars().peekable().to_owned();
        Lexer {
            contents: peekable.clone(),
        }
    }

    fn identifier_type(&mut self, value: &String) -> TokenType {
        match value.as_str() {
            "for" | "let" => TokenType::Keyword,
            "string" | "regex" | "int" => TokenType::Type,
            "in" => TokenType::AssignmentOperator,
            "as" => TokenType::BinaryOperator,
            "input" | "output" | "print" | "println" => TokenType::BuiltIn,
            _ => TokenType::Identifier,
        }
    }

    pub fn tokenize_identifier(&mut self, line: u32, column: &mut u32) -> Token {
        let start_column = column.clone();
        let mut current = String::new();
        while let Some(next) = self.contents.peek() {
            if !(next.is_alphanumeric() || *next == '_') {
                break;
            }
            current.push(*next);
            self.contents.next();
            *column += 1;
        }

        Token::new(self.identifier_type(&current), &current, line, start_column)
    }

    pub fn tokenize_string_literal(&mut self, line: &mut u32, column: &mut u32) -> Token {
        let start_line = line.clone();
        let start_column = column.clone();
        self.contents.next();

        let mut current = String::new();
        while let Some(next) = self.contents.next() {
            if next == '\n' {
                *line += 1;
                *column = 1;
            }
            if next == '"' {
                break;
            }
            current.push(next);
            *column += 1;
        }

        Token::new(TokenType::StringLiteral, &current, start_line, start_column)
    }

    pub fn tokenize_regex_literal(&mut self, line: &mut u32, column: &mut u32) -> Token {
        let start_line = line.clone();
        let start_column = column.clone();

        let mut current = String::new();
        while let Some(next) = self.contents.next() {
            if next == '\n' {
                *line += 1;
                *column = 1;
            }
            if next == '/' {
                break;
            }
            current.push(next);
            *column += 1;
        }

        Token::new(TokenType::RegexLiteral, &current, start_line, start_column)
    }

    pub fn tokenize(&mut self) -> TokenCollection {
        let mut line = 1;
        let mut column = 1;

        let mut tokens = Vec::new();

        while let Some(c) = self.contents.peek() {
            match c {
                '{' => tokens.push(Token::new(
                    TokenType::OpenBlock,
                    &"{".to_string(),
                    line,
                    column,
                )),
                '}' => tokens.push(Token::new(
                    TokenType::CloseBlock,
                    &"}".to_string(),
                    line,
                    column,
                )),
                '(' => tokens.push(Token::new(
                    TokenType::OpenParen,
                    &"(".to_string(),
                    line,
                    column,
                )),
                ')' => tokens.push(Token::new(
                    TokenType::CloseParen,
                    &")".to_string(),
                    line,
                    column,
                )),
                ';' => tokens.push(Token::new(
                    TokenType::Semicolon,
                    &";".to_string(),
                    line,
                    column,
                )),
                '+' => tokens.push(Token::new(
                    TokenType::BinaryOperator,
                    &"+".to_string(),
                    line,
                    column,
                )),
                ':' => tokens.push(Token::new(TokenType::Colon, &":".to_string(), line, column)),
                '=' => tokens.push(Token::new(
                    TokenType::AssignmentOperator,
                    &"=".to_string(),
                    line,
                    column,
                )),
                'a'..='z' | 'A'..='Z' | '_' => {
                    tokens.push(self.tokenize_identifier(line, &mut column));
                    continue;
                }
                '0'..='9' => {
                    let start_column = column.clone();
                    let mut current = String::new();
                    while let Some(next) = self.contents.peek() {
                        if !next.is_numeric() {
                            break;
                        }
                        current.push(*next);
                        self.contents.next();
                        column += 1;
                    }
                    tokens.push(Token::new(
                        TokenType::IntegerLiteral,
                        &current,
                        line,
                        start_column,
                    ));
                    continue;
                }
                '/' => {
                    self.contents.next();
                    if self.contents.peek() == Some(&'/') {
                        while let Some(next) = self.contents.next() {
                            if next == '\n' {
                                line += 1;
                                column = 1;
                                break;
                            }
                            column += 1;
                        }
                        continue;
                    } else {
                        tokens.push(self.tokenize_regex_literal(&mut line, &mut column));
                    }
                    continue;
                }
                '"' => {
                    tokens.push(self.tokenize_string_literal(&mut line, &mut column));
                    continue;
                }
                '\n' => {
                    line += 1;
                    column = 1;
                    self.contents.next();
                    continue;
                }
                ' ' | '\t' => (),
                _ => panic!("Unexpected character: \"{}\"", c),
            }
            column += 1;
            self.contents.next();
        }

        TokenCollection::new(tokens)
    }
}
