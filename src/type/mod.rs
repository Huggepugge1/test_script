#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Regex,
    Int,
    Float,
    Bool,
    None,

    Iterable(Box<Type>),

    Any,
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

            Type::Iterable(inner) => write!(f, "Iter<{}>", inner),

            Type::Any => write!(f, "T"),
        }
    }
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

    pub fn can_cast_to(&self, other: &Type) -> bool {
        if other == &Type::String {
            return true;
        }
        match self {
            Type::String => matches!(other, Type::Int | Type::Float | Type::Bool),

            Type::Int => matches!(other, Type::Float | Type::Bool),
            Type::Float => matches!(other, Type::Int | Type::Bool),

            Type::Bool => matches!(other, Type::Int | Type::Float),

            _ => false,
        }
    }

    pub fn is_iterable(&self) -> bool {
        matches!(self, Type::Regex)
    }

    pub fn get_iterable_inner_type(&self) -> Self {
        match self {
            Type::Regex => Type::String,
            Type::Iterable(inner_type) => *inner_type.clone(),
            _ => unreachable!(),
        }
    }
}
