#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Regex,

    Int,
    Float,

    Bool,

    Vector(Box<Type>),

    None,

    Iterable,

    Any,

    Invalid(String),
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
            v => {
                if v.starts_with("[") && v.ends_with("]") {
                    Type::Vector(Box::new(Type::from(&v[1..v.len() - 1])))
                } else {
                    Type::Invalid(v.to_string())
                }
            }
        }
    }

    pub fn iterable(&self) -> bool {
        matches!(
            self,
            Self::Regex | Self::Vector(_) | Self::Iterable | Self::Any
        )
    }

    pub fn iterable_type(&self) -> Self {
        match self {
            Self::Regex => Self::String,
            Self::Vector(t) => *t.clone(),
            _ => unreachable!(),
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
            Type::Vector(t) => write!(f, "[{}]", t),
            Type::None => write!(f, "none"),

            Type::Iterable => write!(f, "iterable"),

            Type::Any => write!(f, "T"),

            Type::Invalid(v) => write!(f, "`Invalid type {}`", v),
        }
    }
}
