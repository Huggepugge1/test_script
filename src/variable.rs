use crate::r#type::Type;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub r#const: bool,
    pub r#type: Type,
    pub token: Token,
    pub used: bool,
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

impl Variable {
    pub fn new(name: String, r#const: bool, r#type: Type, token: Token) -> Self {
        Self {
            name,
            r#const,
            r#type,
            token,
            used: false,
        }
    }
}
