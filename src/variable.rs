use crate::r#type::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub r#type: Type,
}

impl Variable {
    pub fn new(name: String, r#type: Type) -> Self {
        Self { name, r#type }
    }
}
