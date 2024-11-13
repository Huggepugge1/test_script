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

impl UnaryOperator {
    pub fn value(&self) -> Self {
        match self {
            UnaryOperator::Not => Self::Not,
            UnaryOperator::Negation => Self::Negation,
        }
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

impl Instruction {
    pub const NONE: Instruction = Instruction {
        r#type: InstructionType::None,
        token: Token {
            r#type: TokenType::None,
            value: String::new(),
            line: 0,
            column: 0,
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
