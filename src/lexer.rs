use crate::token::{Token, TokenCollection, TokenType};

fn identifier_type(value: &String) -> TokenType {
    match value.as_str() {
        "for" => TokenType::Keyword,
        "in" => TokenType::AssignmentOperator,
        "input" | "output" | "print" | "println" => TokenType::BuiltIn,
        _ => TokenType::Identifier,
    }
}

pub fn tokenize_identifier(
    contents: &mut std::iter::Peekable<std::str::Chars<'_>>,
    line: u32,
    column: &mut u32,
) -> Token {
    let start_column = column.clone();
    let mut current = String::new();
    while let Some(next) = contents.peek() {
        if !(next.is_alphanumeric() || *next == '_') {
            break;
        }
        current.push(*next);
        contents.next();
        *column += 1;
    }

    Token::new(identifier_type(&current), &current, line, start_column)
}

pub fn tokenize_string_literal(
    contents: &mut std::iter::Peekable<std::str::Chars<'_>>,
    line: &mut u32,
    column: &mut u32,
) -> Token {
    let start_line = line.clone();
    let start_column = column.clone();
    contents.next();

    let mut current = String::new();
    while let Some(next) = contents.next() {
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

pub fn tokenize_regex_literal(
    contents: &mut std::iter::Peekable<std::str::Chars<'_>>,
    line: &mut u32,
    column: &mut u32,
) -> Token {
    let start_line = line.clone();
    let start_column = column.clone();

    let mut current = String::new();
    while let Some(next) = contents.next() {
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

pub fn tokenize(contents: String) -> TokenCollection {
    let mut tokens = TokenCollection::new(Vec::new());
    let mut line = 1;
    let mut column = 1;

    let mut contents = contents.chars().peekable();
    while let Some(c) = contents.peek() {
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
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                tokens.push(tokenize_identifier(&mut contents, line, &mut column));
                continue;
            }
            '/' => {
                contents.next();
                if contents.peek() == Some(&'/') {
                    while let Some(next) = contents.next() {
                        if next == '\n' {
                            line += 1;
                            column = 1;
                            break;
                        }
                        column += 1;
                    }
                    continue;
                } else {
                    tokens.push(tokenize_regex_literal(
                        &mut contents,
                        &mut line,
                        &mut column,
                    ));
                }
                continue;
            }
            '"' => {
                tokens.push(tokenize_string_literal(
                    &mut contents,
                    &mut line,
                    &mut column,
                ));
                continue;
            }
            '\n' => {
                line += 1;
                column = 1;
                contents.next();
                continue;
            }
            ' ' | '\t' => (),
            _ => panic!("Unexpected character: \"{}\"", c),
        }
        column += 1;
        contents.next();
    }

    tokens
}
