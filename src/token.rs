use crate::r#type::Type;
use colored::Colorize;

pub enum PrintStyle<'a> {
    Warning,
    Error,
    Help(&'a str),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    StringLiteral { value: String },
    RegexLiteral { value: String },
    IntegerLiteral { value: i64 },
    FloatLiteral { value: f64 },
    BooleanLiteral { value: bool },

    Keyword { value: String },
    BuiltIn { value: String },

    Type { value: Type },
    Colon,

    Identifier { value: String },

    OpenBlock,
    CloseBlock,

    OpenParen,
    CloseParen,

    OpenBracket,
    CloseBracket,

    TypeCast,

    UnaryOperator { value: String },
    BinaryOperator { value: String },

    Semicolon,

    Comma,

    None,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::StringLiteral { value } => write!(f, "{value}"),
            TokenType::RegexLiteral { value } => write!(f, "{value}"),
            TokenType::IntegerLiteral { value } => write!(f, "`{value}`"),
            TokenType::FloatLiteral { value } => write!(f, "`{value}`"),
            TokenType::BooleanLiteral { value } => write!(f, "`{value}`"),

            TokenType::Keyword { value } => write!(f, "keyword `{value}`"),
            TokenType::BuiltIn { value } => write!(f, "built-in `{value}`"),

            TokenType::Type { value } => write!(f, "{value}"),
            TokenType::Colon => write!(f, ":"),

            TokenType::Identifier { value } => {
                if !value.is_empty() {
                    write!(f, "identifier `{value}`")
                } else {
                    write!(f, "identifier")
                }
            }

            TokenType::OpenBlock => write!(f, "{{"),
            TokenType::CloseBlock => write!(f, "}}"),
            TokenType::OpenParen => write!(f, "("),
            TokenType::CloseParen => write!(f, ")"),
            TokenType::OpenBracket => write!(f, "["),
            TokenType::CloseBracket => write!(f, "]"),

            TokenType::TypeCast => write!(f, "Keyword `as`"),
            TokenType::UnaryOperator { value } => write!(f, "unary operator `{value}`"),
            TokenType::BinaryOperator { value } => write!(f, "binary operator `{value}`"),

            TokenType::Semicolon => write!(f, ";"),
            TokenType::Comma => write!(f, ","),

            TokenType::None => write!(f, ""),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub file: String,
    pub row: usize,
    pub column: usize,

    pub line: String,
    pub last_token: Option<Box<Token>>,
}

impl Token {
    pub fn none() -> Self {
        Self {
            r#type: TokenType::None,
            file: String::new(),
            row: 0,
            column: 0,

            line: String::new(),
            last_token: None,
        }
    }

    pub fn binary_operator(&self) -> bool {
        matches!(
            &self.r#type,
            TokenType::BinaryOperator { .. } | TokenType::TypeCast
        )
    }

    pub fn len(&self) -> usize {
        match &self.r#type {
            TokenType::StringLiteral { value } => value.len(),
            TokenType::RegexLiteral { value } => value.len(),
            TokenType::IntegerLiteral { value } => value.to_string().len(),
            TokenType::FloatLiteral { value } => value.to_string().len(),
            TokenType::BooleanLiteral { value } => value.to_string().len(),

            TokenType::Keyword { value } => value.len(),
            TokenType::BuiltIn { value } => value.len(),

            TokenType::Type { value } => value.to_string().len(),
            TokenType::Colon => 1,

            TokenType::Identifier { value } => value.len(),

            TokenType::OpenBlock => 1,
            TokenType::CloseBlock => 1,

            TokenType::OpenParen => 1,
            TokenType::CloseParen => 1,

            TokenType::OpenBracket => 1,
            TokenType::CloseBracket => 1,

            TokenType::TypeCast => 2,

            TokenType::UnaryOperator { value } => value.len(),
            TokenType::BinaryOperator { value } => value.len(),

            TokenType::Semicolon => 1,
            TokenType::Comma => 1,

            TokenType::None => 0,
        }
    }

    const LINE_NUMBER_PADDING: usize = 4;

    pub fn as_string(&self, style: PrintStyle) -> String {
        let padding_length = usize::max(Self::LINE_NUMBER_PADDING, self.row.to_string().len());
        let padding = &" ".repeat(padding_length + self.column - 1);
        format!(
            "{:<4}{}      \n\
             {}{}",
            self.row.to_string().color(colored::Color::TrueColor {
                r: 0x9F,
                g: 0xFE,
                b: 0xBF,
            }),
            self.line,
            padding,
            match style {
                PrintStyle::Warning => "^".repeat(self.len()).bright_yellow().to_string(),
                PrintStyle::Error => "^".repeat(self.len()).bright_red().to_string(),
                PrintStyle::Help(message) =>
                    "^".repeat(self.len()).bright_blue().to_string() + " " + message,
            }
        )
    }

    pub fn insert_tokens(&self, tokens: Vec<TokenType>, message: &str) -> String {
        let token_len = self.column + self.len() - 1;
        let padding_length = usize::max(Self::LINE_NUMBER_PADDING, self.row.to_string().len());
        let padding = &" ".repeat(padding_length + token_len);

        let mut token_string = tokens
            .iter()
            .fold(String::new(), |acc, token| acc + &format!("{} ", token));

        if tokens[0] == TokenType::OpenBlock || tokens[0] == TokenType::CloseBlock {
            token_string = " ".to_string() + &token_string;
        }

        let new_line = self.line[0..token_len].to_string()
            + &token_string[..token_string.len() - 1]
            + &self.line[token_len..];

        format!(
            "{:<4}{}      \n\
             {}{} {}",
            self.row.to_string().color(colored::Color::TrueColor {
                r: 0x9F,
                g: 0xFE,
                b: 0xBF,
            }),
            new_line,
            padding,
            "+".repeat(token_string.len() - 1).bright_green(),
            message.bright_green()
        )
    }

    pub fn wrap_in_block(&self, close_token: &Token) -> String {
        let token_len = self.column + self.len();
        let padding_length = usize::max(Self::LINE_NUMBER_PADDING, self.row.to_string().len());
        let line_padding = " ".repeat(self.line.chars().take_while(|c| c.is_whitespace()).count());

        let start_line = if token_len > self.line.len() {
            self.line[..self.line.len()].to_string() + " {"
        } else {
            self.line[..token_len].to_string() + "{"
        };
        let start_line_padding = &" ".repeat(padding_length + token_len);

        let content_line = line_padding.clone()
            + "    "
            + &(if self.row == close_token.row {
                self.line[token_len..close_token.column].trim().to_string()
            } else {
                close_token.line.to_string()
            });

        let end_block_line = line_padding.clone() + "}";
        let end_block_padding = line_padding.clone() + &" ".repeat(padding_length);

        let close_token_len = close_token.column + close_token.len();
        let after_line = if close_token_len < close_token.line.len() {
            Some(line_padding + &close_token.line[close_token_len..])
        } else {
            None
        };

        let start_line = format!(
            "{:<4}{}\n{}{}",
            self.row.to_string().color(colored::Color::TrueColor {
                r: 0x9F,
                g: 0xFE,
                b: 0xBF,
            }),
            &start_line,
            start_line_padding,
            "+ open the block here".bright_green()
        );
        let content_line = format!(
            "{:<4}{}",
            (self.row + 1).to_string().color(colored::Color::TrueColor {
                r: 0x9F,
                g: 0xFE,
                b: 0xBF,
            }),
            &content_line,
        );
        let end_block_line = format!(
            "{:<4}{}\n{}{}",
            (usize::max(self.row + 2, close_token.row + 1))
                .to_string()
                .color(colored::Color::TrueColor {
                    r: 0x9F,
                    g: 0xFE,
                    b: 0xBF,
                }),
            &end_block_line,
            end_block_padding,
            "+ close the block here".bright_green()
        );
        match after_line {
            Some(after_line) => {
                let after_line = format!(
                    "{:<4}{}",
                    (usize::max(self.row + 3, close_token.row + 1))
                        .to_string()
                        .color(colored::Color::TrueColor {
                            r: 0x9F,
                            g: 0xFE,
                            b: 0xBF,
                        }),
                    &after_line,
                );

                format!(
                    "{}\n{}\n{}\n{}",
                    start_line, content_line, end_block_line, after_line
                )
            }
            None => format!("{}\n{}\n{}", start_line, content_line, end_block_line),
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
        self.current()
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

    pub fn is_empty(&self) -> bool {
        self.tokens.len() == self.index
    }
}
