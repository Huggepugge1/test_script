#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    String,
    Regex,
    None,

    Iterable,
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

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "String"),
            Type::Regex => write!(f, "Regex"),
            Type::None => write!(f, "()"),

            Type::Iterable => write!(f, "Iterable"),
        }
    }
}
