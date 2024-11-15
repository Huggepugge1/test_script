use crate::r#type::Type;
use crate::token::{Token, TokenType};
use crate::variable::Variable;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub enum BinaryOperator {
    And,
    Or,

    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,

    Addition,
    Subtraction,
    Multiplication,
    Division,
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinaryOperator::And => "&&",
                BinaryOperator::Or => "||",

                BinaryOperator::Equal => "==",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::GreaterThan => ">",
                BinaryOperator::GreaterThanOrEqual => ">=",
                BinaryOperator::LessThan => "<",
                BinaryOperator::LessThanOrEqual => "<=",

                BinaryOperator::Addition => "+",
                BinaryOperator::Subtraction => "-",
                BinaryOperator::Multiplication => "*",
                BinaryOperator::Division => "/",
            }
        )
    }
}

impl BinaryOperator {
    pub fn value(&self) -> Self {
        match self {
            BinaryOperator::Addition => Self::Addition,
            BinaryOperator::Subtraction => Self::Addition,
            BinaryOperator::Multiplication => Self::Multiplication,
            BinaryOperator::Division => Self::Multiplication,

            BinaryOperator::Equal => Self::Equal,
            BinaryOperator::NotEqual => Self::Equal,
            BinaryOperator::GreaterThan => Self::Equal,
            BinaryOperator::GreaterThanOrEqual => Self::Equal,
            BinaryOperator::LessThan => Self::Equal,
            BinaryOperator::LessThanOrEqual => Self::Equal,
            BinaryOperator::And => Self::And,
            BinaryOperator::Or => Self::And,
        }
    }
}

impl std::cmp::Ord for BinaryOperator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().partial_cmp(&other.value()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negation,
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOperator::Not => "!",
                UnaryOperator::Negation => "-",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltIn {
    Input(Box<Instruction>),
    Output(Box<Instruction>),
    Print(Box<Instruction>),
    Println(Box<Instruction>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub r#type: InstructionType,
    pub token: Token,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.r#type {
                InstructionType::StringLiteral(ref value) => value.clone(),
                InstructionType::RegexLiteral(ref value) => format!("{:?}", value),
                InstructionType::IntegerLiteral(ref value) => value.to_string(),
                InstructionType::BooleanLiteral(ref value) => value.to_string(),

                InstructionType::BuiltIn(ref built_in) => match built_in {
                    BuiltIn::Input(ref instruction) => format!("input({})", instruction),
                    BuiltIn::Output(ref instruction) => format!("output({})", instruction),
                    BuiltIn::Print(ref instruction) => format!("print({})", instruction),
                    BuiltIn::Println(ref instruction) => format!("println({})", instruction),
                },

                InstructionType::Block(ref instructions) => {
                    let mut result = String::new();
                    for instruction in instructions {
                        result.push_str(&format!("{}\n", instruction));
                    }
                    result
                }
                InstructionType::Paren(ref instruction) => format!("({})", instruction),

                InstructionType::Test(ref left, ref operator, ref right) => {
                    format!("{} {} {}", left, operator, right)
                }
                InstructionType::For(ref variable, ref instruction) => {
                    format!("for {} in {}", variable, instruction)
                }
                InstructionType::Conditional {
                    ref condition,
                    ref instruction,
                    ref r#else,
                } => format!(
                    "if {} {{\n{}\n}} else {{\n{}\n}}",
                    condition, instruction, r#else
                ),

                InstructionType::Assignment {
                    ref variable,
                    ref instruction,
                } => format!("{} = {}", variable, instruction),
                InstructionType::IterableAssignment(ref variable, ref instruction) => {
                    format!("{} in {}", variable, instruction)
                }
                InstructionType::Variable(ref variable) => variable.to_string(),

                InstructionType::UnaryOperation {
                    ref operator,
                    ref instruction,
                } => format!("{}{}", operator, instruction),
                InstructionType::BinaryOperation {
                    ref operator,
                    ref left,
                    ref right,
                } => format!("{} {} {}", left, operator, right),

                InstructionType::TypeCast {
                    ref instruction,
                    ref r#type,
                } => format!("{} as {}", instruction, r#type),

                InstructionType::None => String::new(),
            }
        )
    }
}

impl Instruction {
    pub const NONE: Instruction = Instruction {
        r#type: InstructionType::None,
        token: Token {
            r#type: TokenType::None,
            file: String::new(),
            row: 0,
            column: 0,

            line: String::new(),
            last_token: None,
        },
    };

    pub fn new(r#type: InstructionType, token: Token) -> Self {
        Self { r#type, token }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    StringLiteral(String),
    RegexLiteral(Vec<String>),
    IntegerLiteral(i64),
    BooleanLiteral(bool),

    BuiltIn(BuiltIn),

    Block(Vec<Instruction>),
    Paren(Box<Instruction>),

    Test(Box<Instruction>, String, String),
    For(Box<Instruction>, Box<Instruction>),
    Conditional {
        condition: Box<Instruction>,
        instruction: Box<Instruction>,
        r#else: Box<Instruction>,
    },

    Assignment {
        variable: Variable,
        instruction: Box<Instruction>,
    },
    IterableAssignment(Variable, Box<Instruction>),
    Variable(Variable),

    UnaryOperation {
        operator: UnaryOperator,
        instruction: Box<Instruction>,
    },
    BinaryOperation {
        operator: BinaryOperator,
        left: Box<Instruction>,
        right: Box<Instruction>,
    },

    TypeCast {
        instruction: Box<Instruction>,
        r#type: Type,
    },

    None,
}
