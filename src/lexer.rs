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
}

impl Token {
    pub fn new(r#type: TokenType, value: &String) -> Token {
        Token {
            r#type,
            value: value.to_string(),
        }
    }
}

pub fn tokenize(contents: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    while i < contents.len() {
        let c = contents.chars().nth(i).unwrap();
        current.push(c);
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                while i + 1 < contents.len() {
                    let next = contents.chars().nth(i + 1).unwrap();
                    if !(next.is_alphanumeric() || next == '_') {
                        break;
                    }
                    current.push(next);
                    i += 1;
                }
                tokens.push(Token::new(TokenType::Identifier, &current));
            }
            '{' => tokens.push(Token::new(TokenType::OpenBlock, &current)),
            '}' => tokens.push(Token::new(TokenType::CloseBlock, &current)),
            '(' => tokens.push(Token::new(TokenType::OpenParen, &current)),
            ')' => tokens.push(Token::new(TokenType::CloseParen, &current)),
            '"' => {
                current.pop();
                while i + 1 < contents.len() {
                    let next = contents.chars().nth(i + 1).unwrap();
                    i += 1;
                    if next == '"' {
                        break;
                    }
                    current.push(next);
                }

                tokens.push(Token::new(TokenType::Literal, &current));
            }
            ';' => tokens.push(Token::new(TokenType::SemiColon, &current)),
            '\n' => (),
            ' ' => (),
            _ => panic!("Unexpected character: {}", c),
        }
        current = String::new();
        i += 1;
    }

    tokens
}
