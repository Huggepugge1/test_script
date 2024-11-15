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
