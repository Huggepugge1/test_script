#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    String,
    Regex,
}

impl Type {
    pub fn from(value: &str) -> Self {
        match value {
            "string" => Type::String,
            "regex" => Type::Regex,
            _ => panic!("Invalid type"),
        }
    }
}
