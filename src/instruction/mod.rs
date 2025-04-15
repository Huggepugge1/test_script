pub mod assignment;
pub mod binary;
pub mod block;
pub mod boolean;
pub mod builtin;
pub mod conditional;
pub mod float;
pub mod r#for;
pub mod function;
pub mod function_call;
pub mod integer;
pub mod paren;
pub mod regex;
pub mod string;
pub mod test;
pub mod type_cast;
pub mod unary;
pub mod variable;

use assignment::iterable_assignment::IterableAssignment;
use assignment::Assignment;
use binary::Binary;
use block::Block;
use boolean::BooleanLiteral;
use builtin::BuiltIn;
use conditional::Conditional;
use float::FloatLiteral;
use function::Function;
use function_call::FunctionCall;
use integer::IntegerLiteral;
use paren::Paren;
use r#for::For;
use regex::RegexLiteral;
use string::StringLiteral;
use test::TestInstruction;
use type_cast::TypeCast;
use unary::Unary;
use variable::Variable;

use crate::environment::{Environment, ParserEnvironment};
use crate::error::{InterpreterError, ParserMessage};
use crate::interpreter::Interpret;
use crate::process::Process;
use crate::r#type::Type;
use crate::token::{Token, TokenType};
use crate::type_checker::TypeCheck;

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionResult {
    String(String),
    Regex(Vec<InstructionResult>),
    Int(i64),
    Float(f64),
    Bool(bool),
    None,
}

impl std::fmt::Display for InstructionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionResult::String(s) => write!(f, "{}", s),
            InstructionResult::Regex(s) => write!(f, "{:?}", s),
            InstructionResult::Int(i) => write!(f, "{}", i),
            InstructionResult::Float(i) => write!(f, "{}", i),
            InstructionResult::Bool(b) => write!(f, "{}", b),
            InstructionResult::None => write!(f, "()"),
        }
    }
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
            match &self.r#type {
                InstructionType::StringLiteral(value) => value.interpret().to_string(),
                InstructionType::RegexLiteral(value) => format!("{:?}", value),
                InstructionType::IntegerLiteral(value) => value.to_string(),
                InstructionType::FloatLiteral(value) => value.to_string(),
                InstructionType::BooleanLiteral(value) => value.to_string(),

                InstructionType::BuiltIn(built_in) => built_in.to_string(),

                InstructionType::Block(block) => block.to_string(),
                InstructionType::Paren(paren) => paren.to_string(),

                InstructionType::Test(test) => test.to_string(),

                InstructionType::Function(function) => function.to_string(),

                InstructionType::For(r#for) => r#for.to_string(),

                InstructionType::Conditional(conditional) => conditional.to_string(),

                InstructionType::Assignment(assignment) => assignment.to_string(),
                InstructionType::IterableAssignment(iterable_assignment) =>
                    iterable_assignment.to_string(),
                InstructionType::Variable(variable) => variable.to_string(),

                InstructionType::FunctionCall(function_call) => function_call.to_string(),
                InstructionType::UnaryOperation(unary_operation) => unary_operation.to_string(),
                InstructionType::BinaryOperation(binary_operation) => binary_operation.to_string(),

                InstructionType::TypeCast(type_cast) => type_cast.to_string(),

                InstructionType::None => String::new(),
            }
        )
    }
}

impl TypeCheck for Instruction {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        _token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        self.r#type.type_check(environment, &self.token, messages)
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

impl Interpret for Instruction {
    fn interpret(
        &self,
        environment: &mut Environment,
        process: &mut Option<&mut Process>,
    ) -> Result<InstructionResult, InterpreterError> {
        Ok(match &self.r#type {
            InstructionType::StringLiteral(string) => string.interpret(),
            InstructionType::RegexLiteral(regex) => regex.interpret(),
            InstructionType::IntegerLiteral(integer) => integer.interpret(),
            InstructionType::FloatLiteral(float) => float.interpret(),
            InstructionType::BooleanLiteral(bool) => bool.interpret(),

            InstructionType::BuiltIn(builtin) => builtin.interpret(environment, process)?,

            InstructionType::Block(block) => block.interpret(environment, process)?,
            InstructionType::Paren(paren) => paren.interpret(environment, process)?,

            InstructionType::For(r#for) => r#for.interpret(environment, process)?,
            InstructionType::Function(function) => function.interpret(environment)?,

            InstructionType::Conditional(conditional) => {
                conditional.interpret(environment, process)?
            }

            InstructionType::Assignment(assignment) => {
                assignment.interpret(environment, process)?
            }

            InstructionType::Variable(variable) => variable.interpret(environment)?,

            InstructionType::FunctionCall(function_call) => {
                function_call.interpret(environment, process)?
            }

            InstructionType::None => InstructionResult::None,

            InstructionType::UnaryOperation(unary_operation) => {
                unary_operation.interpret(environment, process)?
            }
            InstructionType::BinaryOperation(binary_operation) => {
                binary_operation.interpret(environment, process)?
            }

            InstructionType::TypeCast(type_cast) => type_cast.interpret(environment, process)?,
            _ => {
                unreachable!();
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    StringLiteral(StringLiteral),
    RegexLiteral(RegexLiteral),
    IntegerLiteral(IntegerLiteral),
    FloatLiteral(FloatLiteral),
    BooleanLiteral(BooleanLiteral),

    BuiltIn(BuiltIn),

    Block(Block),
    Paren(Paren),

    Test(TestInstruction),
    Function(Function),

    For(For),

    Conditional(Conditional),

    Assignment(Assignment),
    IterableAssignment(IterableAssignment),

    Variable(Variable),

    FunctionCall(FunctionCall),

    UnaryOperation(Unary),
    BinaryOperation(Binary),

    TypeCast(TypeCast),

    None,
}

impl TypeCheck for InstructionType {
    fn type_check(
        &self,
        environment: &mut ParserEnvironment,
        token: &Token,
        messages: &mut Vec<ParserMessage>,
    ) -> Type {
        match self {
            InstructionType::StringLiteral(string) => {
                string.type_check(environment, token, messages)
            }
            InstructionType::RegexLiteral(regex) => regex.type_check(environment, token, messages),
            InstructionType::IntegerLiteral(integer) => {
                integer.type_check(environment, token, messages)
            }
            InstructionType::FloatLiteral(float) => float.type_check(environment, token, messages),
            InstructionType::BooleanLiteral(boolean) => {
                boolean.type_check(environment, token, messages)
            }

            InstructionType::BuiltIn(built_in) => built_in.type_check(environment, token, messages),

            InstructionType::Block(block) => block.type_check(environment, token, messages),
            InstructionType::Paren(paren) => paren.type_check(environment, token, messages),

            InstructionType::Test(test) => test.type_check(environment, token, messages),
            InstructionType::Function(function) => {
                function.type_check(environment, token, messages)
            }

            InstructionType::For(r#for) => r#for.type_check(environment, token, messages),

            InstructionType::Conditional(conditional) => {
                conditional.type_check(environment, token, messages)
            }

            InstructionType::Assignment(assignment) => {
                assignment.type_check(environment, token, messages)
            }
            InstructionType::IterableAssignment(iterable_assignment) => {
                iterable_assignment.type_check(environment, token, messages)
            }

            InstructionType::Variable(variable) => {
                variable.type_check(environment, token, messages)
            }

            InstructionType::FunctionCall(function_call) => {
                function_call.type_check(environment, token, messages)
            }

            InstructionType::UnaryOperation(unary_operation) => {
                unary_operation.type_check(environment, token, messages)
            }
            InstructionType::BinaryOperation(binary_operation) => {
                binary_operation.type_check(environment, token, messages)
            }

            InstructionType::TypeCast(type_cast) => {
                type_cast.type_check(environment, token, messages)
            }

            InstructionType::None => {
                messages.push(ParserMessage::warning_trailing_semicolon(token.clone()));
                Type::None
            }
        }
    }
}
