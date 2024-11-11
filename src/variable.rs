use crate::r#type::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub r#const: bool,
    pub r#type: Type,
}

impl Variable {
    pub fn new(name: String, r#const: bool, r#type: Type) -> Self {
        Self {
            name,
            r#const,
            r#type,
        }
    }
}
