#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    String,
    Regex,
    Int,
    Float,
    Bool,
    None,

    Iterable,

    Any,
}

impl Type {
    pub fn from(value: &str) -> Self {
        match value {
            "string" => Type::String,
            "regex" => Type::Regex,
            "int" => Type::Int,
            "float" => Type::Float,
            "bool" => Type::Bool,
            "none" => Type::None,
            _ => panic!("Invalid type"),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "string"),
            Type::Regex => write!(f, "regex"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::None => write!(f, "none"),

            Type::Iterable => write!(f, "iterable"),

            Type::Any => write!(f, "T"),
        }
    }
}
