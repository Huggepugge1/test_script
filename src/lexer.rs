use crate::cli::Args;
use crate::r#type::Type;
use crate::token::{Token, TokenCollection, TokenType};

use std::path::PathBuf;

pub struct Lexer<'a> {
    lines: Vec<String>,
    contents: std::iter::Peekable<std::str::Chars<'a>>,
    file: PathBuf,
    tokens: Vec<Token>,

    row: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(contents: &'a mut str, args: Args) -> Lexer<'a> {
        let lines = contents.lines().map(|s| s.to_string()).collect();
        let contents = contents.chars().peekable();

        let row = 1;
        let column = 1;

        let tokens = Vec::new();

        Lexer {
            lines,
            contents,
            file: args.file,
            tokens,

            row,
            column,
        }
    }

    fn make_token(&self, r#type: TokenType) -> Token {
        Token {
            r#type,
            file: self.file.to_str().unwrap().to_string(),
            row: self.row,
            column: self.column,

            line: self.get_line(),
            last_token: match self.tokens.last() {
                Some(token) => {
                    let mut token = token.clone();
                    token.last_token = None;
                    Some(Box::new(token))
                }
                None => None,
            },
        }
    }

    fn get_line(&self) -> String {
        self.lines[self.row - 1].clone()
    }

    fn identifier_type(&mut self, value: &String) -> TokenType {
        match value.as_str() {
            "for" | "let" | "const" | "if" | "else" | "fn" => TokenType::Keyword {
                value: value.to_string(),
            },
            "string" | "regex" | "int" | "float" | "bool" | "none" => TokenType::Type {
                value: Type::from(value),
            },
            "true" | "false" => TokenType::BooleanLiteral {
                value: value.parse::<bool>().unwrap(),
            },
            "in" => TokenType::IterableAssignmentOperator,
            "as" => TokenType::TypeCast,
            "input" | "output" | "print" | "println" => TokenType::BuiltIn {
                name: value.to_string(),
            },
            _ => TokenType::Identifier {
                value: value.to_string(),
            },
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
        let token = self.make_token(token_type);
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

        current = current.replace("\\n", "\n");
        current = current.replace("\\t", "\t");
        current = current.replace("\\r", "\r");
        let token = self.make_token(TokenType::StringLiteral { value: current });
        self.row = new_row;
        self.column = new_column;
        token
    }

    pub fn tokenize_regex_literal(&mut self) -> Token {
        let mut new_row = self.row;
        let mut new_column = self.column + 1;
        let mut current = String::from("`");

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

        let token = self.make_token(TokenType::RegexLiteral { value: current });
        self.row = new_row;
        self.column = new_column;
        token
    }

    pub fn tokenize_number_literal(&mut self) -> Token {
        let mut length = 0;
        let mut current = String::new();
        let mut float = false;
        while let Some(next) = self.contents.peek() {
            if *next == '.' {
                if float {
                    panic!("Unexpected character: \".\"");
                }
                float = true;
            } else if !next.is_ascii_digit() {
                break;
            }
            current.push(*next);
            self.contents.next();
            length += 1;
        }

        let token = match float {
            false => self.make_token(TokenType::IntegerLiteral {
                value: current.parse::<i64>().unwrap(),
            }),
            true => self.make_token(TokenType::FloatLiteral {
                value: current.parse::<f64>().unwrap(),
            }),
        };
        self.column += length;
        token
    }

    pub fn tokenize(&mut self) -> TokenCollection {
        while let Some(c) = self.contents.peek() {
            match c {
                '{' => self.tokens.push(self.make_token(TokenType::OpenBlock)),
                '}' => self.tokens.push(self.make_token(TokenType::CloseBlock)),
                '(' => self.tokens.push(self.make_token(TokenType::OpenParen)),
                ')' => self.tokens.push(self.make_token(TokenType::CloseParen)),
                ';' => self.tokens.push(self.make_token(TokenType::Semicolon)),
                ',' => self.tokens.push(self.make_token(TokenType::Comma)),
                '+' => self.tokens.push(self.make_token(TokenType::BinaryOperator {
                    value: "+".to_string(),
                })),
                '-' => self.tokens.push(self.make_token(TokenType::BinaryOperator {
                    value: "-".to_string(),
                })),
                '*' => self.tokens.push(self.make_token(TokenType::BinaryOperator {
                    value: "*".to_string(),
                })),
                '/' => {
                    self.contents.next();
                    if let Some('/') = self.contents.peek() {
                        for next in self.contents.by_ref() {
                            if next == '\n' {
                                break;
                            }
                        }
                    } else {
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: "/".to_string(),
                        }));
                        self.column += 1;
                        continue;
                    }
                    self.column = 1;
                    self.row += 1;
                    continue;
                }
                '%' => self.tokens.push(self.make_token(TokenType::BinaryOperator {
                    value: "%".to_string(),
                })),
                ':' => self.tokens.push(self.make_token(TokenType::Colon)),
                '<' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('=') = self.contents.peek() {
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: "<=".to_string(),
                        }));
                        length += 1;
                        self.contents.next();
                    } else {
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: "<".to_string(),
                        }));
                    }
                    self.column += length;
                    continue;
                }
                '>' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('=') = self.contents.peek() {
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: ">=".to_string(),
                        }));
                        length += 1;
                        self.contents.next();
                    } else {
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: ">".to_string(),
                        }));
                    }
                    self.column += length;
                }
                '=' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('=') = self.contents.peek() {
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: "==".to_string(),
                        }));
                        length += 1;
                        self.contents.next();
                    } else {
                        self.tokens
                            .push(self.make_token(TokenType::AssignmentOperator));
                    }
                    self.column += length;
                    continue;
                }
                '!' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('=') = self.contents.peek() {
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: "!=".to_string(),
                        }));
                        length += 1;
                        self.contents.next();
                    } else {
                        self.tokens.push(self.make_token(TokenType::UnaryOperator {
                            value: "!".to_string(),
                        }));
                    }
                    self.column += length;
                    continue;
                }
                '&' => {
                    self.contents.next();
                    let mut length = 1;
                    if let Some('&') = self.contents.peek() {
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: "&&".to_string(),
                        }));
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
                        self.tokens.push(self.make_token(TokenType::BinaryOperator {
                            value: "||".to_string(),
                        }));
                        length += 1;
                        self.contents.next();
                    } else {
                        panic!("Unexpected character: \"|\"");
                    }
                    self.column += length;
                    continue;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let token = self.tokenize_identifier();
                    self.tokens.push(token);
                    continue;
                }
                '"' => {
                    let token = self.tokenize_string_literal();
                    self.tokens.push(token);
                    continue;
                }
                '`' => {
                    let token = self.tokenize_regex_literal();
                    self.tokens.push(token);
                    continue;
                }
                '0'..='9' => {
                    let token = self.tokenize_number_literal();
                    self.tokens.push(token);
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

        TokenCollection::new(self.tokens.clone())
    }
}
