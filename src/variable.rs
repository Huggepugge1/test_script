use crate::r#type::Type;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub r#const: bool,
    pub r#type: Type,

    pub declaration_token: Token,
    pub identifier_token: Token,

    pub last_assignment_token: Token,

    pub read: bool,
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}: {}",
            if self.r#const { "const " } else { "" },
            self.name,
            self.r#type
        )
    }
}

pub trait SnakeCase {
    fn is_snake_case(&self) -> bool;
    fn is_upper_snake_case(&self) -> bool;
    fn to_snake_case(&self) -> String;
    fn to_upper_snake_case(&self) -> String;
}

impl SnakeCase for std::string::String {
    fn is_snake_case(&self) -> bool {
        self.chars().all(|c| c.is_lowercase() || c == '_')
    }

    fn is_upper_snake_case(&self) -> bool {
        self.chars().all(|c| c.is_uppercase() || c == '_')
    }

    fn to_snake_case(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.chars().enumerate() {
            if i > 0
                && c.is_uppercase()
                && (i < self.len() - 1 && self.chars().nth(i + 1).unwrap().is_lowercase())
            {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }

    fn to_upper_snake_case(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.chars().enumerate() {
            if i > 0
                && c.is_uppercase()
                && (i < self.len() - 1 && self.chars().nth(i + 1).unwrap().is_lowercase())
            {
                result.push('_');
            }
            result.push(c.to_uppercase().next().unwrap());
        }
        result
    }
}
