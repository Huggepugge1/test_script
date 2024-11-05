#[derive(Debug, PartialEq)]
pub enum TokenType {
    Literal,
    Identifier,
    OpenBlock,
    CloseBlock,
    OpenParen,
    CloseParen,
    SemiColon,
}

#[derive(Debug, PartialEq)]
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

pub fn tokenize(contents: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    let mut line = 1;
    let mut column = 1;
    while i < contents.len() {
        let c = contents.chars().nth(i).unwrap();
        current.push(c);
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                let start_column = column;
                while i + 1 < contents.len() {
                    let next = contents.chars().nth(i + 1).unwrap();
                    if !(next.is_alphanumeric() || next == '_') {
                        break;
                    }
                    current.push(next);
                    i += 1;
                    column += 1;
                }
                tokens.push(Token::new(
                    TokenType::Identifier,
                    &current,
                    line,
                    start_column,
                ));
            }
            '{' => tokens.push(Token::new(TokenType::OpenBlock, &current, line, column)),
            '}' => tokens.push(Token::new(TokenType::CloseBlock, &current, line, column)),
            '(' => tokens.push(Token::new(TokenType::OpenParen, &current, line, column)),
            ')' => tokens.push(Token::new(TokenType::CloseParen, &current, line, column)),
            '"' => {
                current.pop();
                let start_line = line;
                let start_column = column;
                while i + 1 < contents.len() {
                    let next = contents.chars().nth(i + 1).unwrap();
                    i += 1;
                    if next == '"' {
                        break;
                    }
                    if next == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    current.push(next);
                }

                tokens.push(Token::new(
                    TokenType::Literal,
                    &current,
                    start_line,
                    start_column,
                ));
            }
            ';' => tokens.push(Token::new(TokenType::SemiColon, &current, line, column)),
            '\n' => {
                line += 1;
                column = 1;
            }
            ' ' | '\t' => (),
            _ => panic!("Unexpected character: \"{:?}\"", c),
        }
        current = String::new();
        column += 1;
        i += 1;
    }

    tokens
}
