use crate::environment::Environment;
use crate::error::{InterpreterError, InterpreterErrorType};
use crate::instruction::{BinaryOperator, BuiltIn, Instruction, InstructionType, UnaryOperator};
use crate::r#type::Type;
use crate::variable::Variable;

use expectrl::{spawn, Session, WaitStatus};

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionResult {
    String(String),
    Regex(Vec<String>),
    Integer(i64),
    Float(f64),
    Bool(bool),
    None,
}

impl std::fmt::Display for InstructionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionResult::String(s) => write!(f, "{}", s),
            InstructionResult::Regex(s) => write!(f, "{:?}", s),
            InstructionResult::Integer(i) => write!(f, "{}", i),
            InstructionResult::Float(i) => write!(f, "{}", i),
            InstructionResult::Bool(b) => write!(f, "{}", b),
            InstructionResult::None => write!(f, "()"),
        }
    }
}

struct Test {
    name: String,
    instruction: Instruction,
    environment: Environment,
    process: Session,
    passed: bool,
}

impl Test {
    fn new(name: String, command: String, instruction: Instruction) -> Self {
        let process = spawn(command).unwrap();

        Self {
            name,

            instruction,
            environment: Environment::new(),
            process,
            passed: true,
        }
    }

    fn run(&mut self) {
        let instruction = self.instruction.clone();
        match self.interpret_instruction(instruction) {
            Ok(_) => (),
            Err(e) => {
                e.print();
                return;
            }
        }

        match self.process.get_process().status() {
            Ok(WaitStatus::Exited(_, value)) => {
                if value != 0 {
                    self.passed = false;
                    self.fail(&format!("Non zero exit code {}", value));
                }
            }
            Ok(_) => {
                self.passed = false;
                self.fail("Process did not exit");
            }
            Err(e) => {
                eprintln!("Failed to kill process: {}", e);
            }
        }

        match self.passed {
            false => (),
            true => self.pass(),
        }
    }

    fn pass(&self) {
        println!("Test passed: {}", self.name);
    }

    fn fail(&self, message: &str) {
        eprintln!("{} failed: {}", self.name, message);
    }

    fn interpret_unary_operation(
        &mut self,
        operator: UnaryOperator,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        match operator {
            UnaryOperator::Not => self.interpret_not(instruction),
            UnaryOperator::Negation => self.interpret_negation(instruction),
        }
    }

    fn interpret_not(
        &mut self,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let value = self.interpret_instruction(instruction)?;
        match value {
            InstructionResult::Bool(value) => Ok(InstructionResult::Bool(!value)),
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_negation(
        &mut self,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let value = self.interpret_instruction(instruction)?;
        match value {
            InstructionResult::Integer(value) => Ok(InstructionResult::Integer(-value)),
            InstructionResult::Float(value) => Ok(InstructionResult::Float(-value)),
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_binary_operation(
        &mut self,
        operator: BinaryOperator,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        match operator {
            BinaryOperator::Addition => self.interpret_addition(left, right),
            BinaryOperator::Subtraction => self.interpret_subtraction(left, right),
            BinaryOperator::Multiplication => self.interpret_multiplication(left, right),
            BinaryOperator::Division => self.interpret_division(left, right),
            BinaryOperator::Modulo => self.interpret_modulo(left, right),

            BinaryOperator::Equal => self.interpret_equal(left, right),
            BinaryOperator::NotEqual => self.interpret_not_equal(left, right),
            BinaryOperator::GreaterThan => self.interpret_greater_than(left, right),
            BinaryOperator::GreaterThanOrEqual => self.interpret_greater_than_or_equal(left, right),
            BinaryOperator::LessThan => self.interpret_less_than(left, right),
            BinaryOperator::LessThanOrEqual => self.interpret_less_than_or_equal(left, right),

            BinaryOperator::And => self.interpret_and(left, right),
            BinaryOperator::Or => self.interpret_or(left, right),
        }
    }

    fn interpret_addition(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::String(left), InstructionResult::String(right)) => {
                Ok(InstructionResult::String(format!("{}{}", left, right)))
            }
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Integer(left + right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left + right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_subtraction(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Integer(left - right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left - right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_multiplication(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Integer(left * right))
            }
            (InstructionResult::String(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::String(left.repeat(right as usize)))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left * right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_division(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Integer(left / right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Float(left / right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_modulo(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Integer(left % right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_equal(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        Ok(InstructionResult::Bool(left == right))
    }

    fn interpret_not_equal(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        Ok(InstructionResult::Bool(left != right))
    }

    fn interpret_greater_than(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Bool(left > right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left > right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_greater_than_or_equal(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Bool(left >= right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left >= right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_less_than(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Bool(left < right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left < right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_less_than_or_equal(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Integer(left), InstructionResult::Integer(right)) => {
                Ok(InstructionResult::Bool(left <= right))
            }
            (InstructionResult::Float(left), InstructionResult::Float(right)) => {
                Ok(InstructionResult::Bool(left <= right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_and(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                Ok(InstructionResult::Bool(left && right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_or(
        &mut self,
        left: Instruction,
        right: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let left = self.interpret_instruction(left)?;
        let right = self.interpret_instruction(right)?;
        match (left.clone(), right.clone()) {
            (InstructionResult::Bool(left), InstructionResult::Bool(right)) => {
                Ok(InstructionResult::Bool(left || right))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_builtin(
        &mut self,
        builtin: BuiltIn,
    ) -> Result<InstructionResult, InterpreterError> {
        let value = match builtin.clone() {
            BuiltIn::Input(value) => self.interpret_instruction(*value)?,
            BuiltIn::Output(value) => self.interpret_instruction(*value)?,
            BuiltIn::Print(value) => self.interpret_instruction(*value)?,
            BuiltIn::Println(value) => self.interpret_instruction(*value)?,
        };
        let value = match value {
            InstructionResult::String(value) => value,
            _ => {
                unreachable!()
            }
        };
        match builtin {
            BuiltIn::Input(_) => {
                self.process
                    .send_line(&value)
                    .expect("Failed to send input to process");
            }
            BuiltIn::Output(_) => match self.process.expect(expectrl::Regex(".")) {
                Ok(v) => {
                    let output = String::from_utf8(v.as_bytes().to_vec())
                        .unwrap()
                        .replace("\r", "");
                    if output.trim() != value {
                        self.passed = false;
                        self.fail(&format!(
                            "Output mismatch: expected \"{:?}\", got \"{:?}\"",
                            value,
                            output.trim(),
                        ));
                    }
                }
                Err(_) => {
                    self.passed = false;
                    self.fail("Got no output");
                }
            },
            BuiltIn::Print(_) => print!("{}", value),
            BuiltIn::Println(_) => println!("{}", value),
        }
        Ok(InstructionResult::None)
    }

    fn interpret_assignment(
        &mut self,
        var: Variable,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let value = self.interpret_instruction(instruction)?;
        self.environment.insert(var.name.clone(), value.clone());
        Ok(value)
    }

    fn interpret_block(
        &mut self,
        instructions: Vec<Instruction>,
    ) -> Result<InstructionResult, InterpreterError> {
        let mut result = InstructionResult::None;
        self.environment.add_scope();
        for instruction in instructions {
            result = self.interpret_instruction(instruction)?;
        }
        self.environment.remove_scope();
        Ok(result)
    }

    fn interpret_conditional(
        &mut self,
        condition: Instruction,
        instruction: Instruction,
        r#else: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let condition = self.interpret_instruction(condition)?;
        match condition {
            InstructionResult::Bool(true) => self.interpret_instruction(instruction),
            InstructionResult::Bool(false) => self.interpret_instruction(r#else),
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_for(
        &mut self,
        assignment: Instruction,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        let mut result = InstructionResult::None;
        let (assignment_var, assignment_values) = match assignment.r#type {
            InstructionType::IterableAssignment(var, instruction) => {
                (var, self.interpret_instruction(*instruction)?)
            }
            _ => {
                unreachable!()
            }
        };
        match assignment_values {
            InstructionResult::Regex(values) => {
                for value in values {
                    self.environment.insert(
                        assignment_var.name.clone(),
                        InstructionResult::String(value),
                    );
                    result = self.interpret_instruction(instruction.clone())?;
                }
            }
            _ => {
                unreachable!()
            }
        }
        Ok(result)
    }

    fn interpret_variable(&self, var: Variable) -> Result<InstructionResult, InterpreterError> {
        match self.environment.get(&var.name) {
            Some(value) => Ok(value.clone()),
            None => unreachable!(),
        }
    }

    fn interpret_type_cast(
        &mut self,
        instruction: Instruction,
        r#type: Type,
    ) -> Result<InstructionResult, InterpreterError> {
        let value = self.interpret_instruction(instruction)?;
        match (value.clone(), r#type) {
            (InstructionResult::String(value), Type::Int) => {
                Ok(InstructionResult::Integer(match value.parse() {
                    Ok(value) => value,
                    Err(_) => {
                        return Err(InterpreterError::new(
                            InterpreterErrorType::TypeCastError {
                                result: InstructionResult::String(value),
                                from: Type::String,
                                to: Type::Int,
                            },
                            "Failed to cast string to int",
                        ));
                    }
                }))
            }
            (InstructionResult::Integer(value), Type::String) => {
                Ok(InstructionResult::String(value.to_string()))
            }
            (InstructionResult::String(value), Type::Regex) => {
                Ok(InstructionResult::Regex(vec![value]))
            }

            (InstructionResult::String(value), Type::Bool) => {
                Ok(InstructionResult::Bool(value == "true"))
            }
            (InstructionResult::Bool(value), Type::String) => {
                Ok(InstructionResult::String(value.to_string()))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn interpret_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<InstructionResult, InterpreterError> {
        Ok(match instruction.r#type {
            InstructionType::StringLiteral(value) => InstructionResult::String(value),
            InstructionType::RegexLiteral(value) => InstructionResult::Regex(value),
            InstructionType::IntegerLiteral(value) => InstructionResult::Integer(value),
            InstructionType::FloatLiteral(value) => InstructionResult::Float(value),
            InstructionType::BooleanLiteral(value) => InstructionResult::Bool(value),

            InstructionType::BuiltIn(builtin) => self.interpret_builtin(builtin)?,

            InstructionType::Block(instructions) => self.interpret_block(instructions)?,
            InstructionType::Paren(instruction) => self.interpret_instruction(*instruction)?,

            InstructionType::Conditional {
                condition,
                instruction,
                r#else,
            } => self.interpret_conditional(*condition, *instruction, *r#else)?,
            InstructionType::For(assignment, instruction) => {
                self.interpret_for(*assignment, *instruction)?
            }
            InstructionType::Assignment {
                variable,
                instruction,
                ..
            } => self.interpret_assignment(variable, *instruction)?,
            InstructionType::Variable(var) => self.interpret_variable(var)?,

            InstructionType::None => InstructionResult::None,

            InstructionType::UnaryOperation {
                operator,
                instruction,
            } => self.interpret_unary_operation(operator, *instruction)?,
            InstructionType::BinaryOperation {
                operator,
                left,
                right,
            } => self.interpret_binary_operation(operator, *left, *right)?,

            InstructionType::TypeCast {
                instruction,
                r#type,
            } => self.interpret_type_cast(*instruction, r#type)?,
            _ => {
                unreachable!();
            }
        })
    }
}

pub struct Interpreter {
    program: Vec<Instruction>,
}

impl Interpreter {
    pub fn new(program: Vec<Instruction>) -> Self {
        Self { program }
    }

    fn interpret_test(&self, instruction: Instruction) {
        match instruction.r#type {
            InstructionType::Test(instruction, name, file) => {
                let mut test = Test::new(name, file, *instruction);
                test.run();
            }
            _ => panic!("Unexpected instruction {:?}", instruction),
        }
    }

    pub fn interpret(&self) {
        for test in self.program.clone().into_iter() {
            self.interpret_test(test);
        }
    }
}
