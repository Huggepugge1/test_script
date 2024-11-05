use crate::token::{Token, TokenCollection, TokenType};

fn identifier_type(value: &String) -> TokenType {
    match value.as_str() {
        "for" => TokenType::Keyword,
        "input" | "output" => TokenType::BuiltIn,
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

    let mut current = String::new();
    while let Some(next) = contents.peek() {
        if *next == '\n' {
            *line += 1;
            *column = 1;
        }
        if *next == '"' {
            break;
        }
        current.push(*next);
        contents.next();
        *column += 1;
    }

    Token::new(TokenType::Literal, &current, start_line, start_column)
}

pub fn tokenize(contents: String) -> TokenCollection {
    let mut tokens = TokenCollection::new(Vec::new());
    let mut current = String::new();
    let mut line = 1;
    let mut column = 1;

    let mut contents = contents.chars().peekable();
    while let Some(c) = contents.peek() {
        current.push(*c);
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                tokens.push(tokenize_identifier(&mut contents, line, &mut column))
            }
            '{' => tokens.push(Token::new(TokenType::OpenBlock, &current, line, column)),
            '}' => tokens.push(Token::new(TokenType::CloseBlock, &current, line, column)),
            '(' => tokens.push(Token::new(TokenType::OpenParen, &current, line, column)),
            ')' => tokens.push(Token::new(TokenType::CloseParen, &current, line, column)),
            '"' => tokens.push(tokenize_string_literal(
                &mut contents,
                &mut line,
                &mut column,
            )),
            ';' => tokens.push(Token::new(TokenType::SemiColon, &current, line, column)),
            '\n' => {
                line += 1;
                column = 1;
            }
            ' ' | '\t' => (),
            _ => panic!("Unexpected character: \"{}\"", c),
        }
        current = String::new();
        column += 1;
        contents.next();
    }

    tokens
}
