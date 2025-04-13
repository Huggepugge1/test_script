use crate::cli::Args;
use crate::environment::ParseEnvironment;
use crate::error::{ParseError, ParseErrorType, ParseWarning, ParseWarningType};
use crate::instruction::assignment::iterable_assignment::IterableAssignment;
use crate::instruction::assignment::Assignment;
use crate::instruction::binary_operation::{BinaryOperation, BinaryOperator};
use crate::instruction::block::Block;
use crate::instruction::builtin::BuiltIn;
use crate::instruction::conditional::Conditional;
use crate::instruction::function::Function;
use crate::instruction::function_call::FunctionCall;
use crate::instruction::r#for::For;
use crate::instruction::test::TestInstruction;
use crate::instruction::type_cast::TypeCast;
use crate::instruction::unary_operation::{UnaryOperation, UnaryOperator};
use crate::instruction::{Instruction, InstructionType};
use crate::r#type::Type;
use crate::token::Token;

pub struct TypeChecker {
    program: Vec<Instruction>,
    environment: ParseEnvironment,
    success: bool,
    args: Args,
}

impl TypeChecker {
    pub fn new(program: Vec<Instruction>, args: Args) -> Self {
        Self {
            program,
            environment: ParseEnvironment::new(args.clone()),
            success: true,
            args,
        }
    }

    pub fn check(&mut self) -> Result<(), ParseError> {
        for instruction in self.program.clone() {
            match instruction.r#type {
                InstructionType::Test(TestInstruction { body, .. }) => {
                    match self.check_instruction(&body) {
                        Ok(_) => (),
                        Err(e) => {
                            e.print();
                            self.success = false;
                        }
                    }
                }
                InstructionType::Function(_) => match self.check_instruction(&instruction) {
                    Ok(_) => (),
                    Err(e) => {
                        e.print();
                        self.success = false;
                    }
                },

                InstructionType::Assignment(assignment) => {
                    match self.check_instruction(&assignment.body) {
                        Ok(_) => (),
                        Err(e) => {
                            e.print();
                            self.success = false;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        match self.success {
            true => Ok(()),
            false => Err(ParseError::none()),
        }
    }

    fn check_instruction(&mut self, instruction: &Instruction) -> Result<Type, ParseError> {
        let token = instruction.token.clone();
        match &instruction.r#type {
            InstructionType::StringLiteral(_) => Ok(Type::String),
            InstructionType::RegexLiteral(_) => Ok(Type::Regex),
            InstructionType::IntegerLiteral(_) => Ok(Type::Int),
            InstructionType::FloatLiteral(_) => Ok(Type::Float),
            InstructionType::BooleanLiteral(_) => Ok(Type::Bool),

            InstructionType::BuiltIn(instruction) => self.check_builtin(instruction, &token),

            InstructionType::Block(block) => self.check_block(block),

            InstructionType::Paren(paren) => self.check_instruction(&paren.expression),

            InstructionType::Conditional(conditional) => self.check_conditional(conditional),

            InstructionType::Function(function) => self.check_function(function),

            InstructionType::For(For { assignment, body }) => {
                self.environment.add_scope();
                self.check_iterable_assignment(assignment)?;
                let result = self.check_instruction(body)?;
                self.environment.remove_scope();
                Ok(result)
            }

            InstructionType::Variable(variable) => {
                let variable = match self.environment.get(&variable.name) {
                    Some(v) => {
                        v.read = true;
                        v
                    }
                    None => variable,
                };
                Ok(variable.r#type.clone())
            }

            InstructionType::FunctionCall(FunctionCall { name, arguments }) => {
                self.check_function_call(name, arguments)
            }

            InstructionType::Assignment(assignment) => self.check_assignment(assignment),

            InstructionType::IterableAssignment(assignment) => {
                self.check_iterable_assignment(assignment)
            }

            InstructionType::UnaryOperation(UnaryOperation {
                operator,
                instruction,
            }) => self.check_unary(operator, instruction),
            InstructionType::BinaryOperation(BinaryOperation {
                operator,
                left,
                right,
            }) => self.check_binary(operator, left, right),

            InstructionType::TypeCast(TypeCast {
                from: left_instruction,
                to,
            }) => self.check_type_cast(left_instruction, instruction, to),

            InstructionType::None => {
                ParseWarning::new(
                    ParseWarningType::TrailingSemicolon,
                    instruction.token.clone(),
                )
                .print(self.args.disable_warnings);
                Ok(Type::None)
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn check_builtin(&mut self, built_in: &BuiltIn, token: &Token) -> Result<Type, ParseError> {
        match built_in.name.as_str() {
            "print" | "println" => {
                for argument in &built_in.arguments {
                    self.check_instruction(argument)?;
                }
                Ok(Type::None)
            }
            "input" | "output" => {
                if built_in.arguments.len() != 1 {
                    return Err(ParseError::new(
                        ParseErrorType::MismatchedArguments {
                            expected: 1,
                            actual: built_in.arguments.len(),
                        },
                        token.clone(),
                    ));
                }
                let r#type = self.check_instruction(&built_in.arguments[0])?;
                match r#type {
                    Type::String => Ok(Type::None),
                    _ => Err(ParseError::new(
                        ParseErrorType::MismatchedType {
                            expected: vec![Type::String],
                            actual: r#type,
                        },
                        token.clone(),
                    )),
                }
            }
            _ => unreachable!(),
        }
    }

    fn check_block(&mut self, block: &Block) -> Result<Type, ParseError> {
        self.environment.add_scope();
        if block.statements.is_empty() {
            return Ok(Type::None);
        }
        for instruction in &block.statements[..block.statements.len() - 1] {
            match self.check_instruction(instruction) {
                Ok(t) => match t {
                    Type::None => (),
                    _ => {
                        ParseWarning::new(
                            ParseWarningType::UnusedValue,
                            instruction.inner_most().token.clone(),
                        )
                        .print(self.args.disable_warnings);
                    }
                },
                Err(e) => {
                    e.print();
                    self.success = false;
                }
            }
        }
        let result = self.check_instruction(&block.statements[block.statements.len() - 1])?;
        self.environment.remove_scope();
        Ok(result)
    }

    fn check_assignment(&mut self, assignment: &Assignment) -> Result<Type, ParseError> {
        let variable_type = assignment.variable.r#type.clone();

        let instruction_type = self.check_instruction(&assignment.body)?;

        if variable_type != Type::Any && variable_type != instruction_type {
            return Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![variable_type],
                    actual: instruction_type,
                },
                assignment.variable.declaration_token.clone(),
            ));
        }

        let mut variable = match self.environment.get(&assignment.variable.name) {
            Some(v) => v.clone(),
            None => assignment.variable.clone(),
        };
        variable.read = false;
        variable.last_assignment_token = assignment.token.clone();

        variable.assigned = true;

        self.environment.insert(variable);
        Ok(Type::None)
    }

    fn check_iterable_assignment(
        &mut self,
        assignment: &IterableAssignment,
    ) -> Result<Type, ParseError> {
        let variable_type = assignment.variable.r#type.clone();
        let t = self.check_instruction(&assignment.body)?;
        if t.is_iterable() && t.get_iterable_inner_type() == variable_type {
            self.environment.insert(assignment.variable.clone());
            if let Some(v) = self.environment.get(&assignment.variable.name) {
                v.assigned = true;
            }
            Ok(variable_type)
        } else {
            Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Iterable(Box::new(variable_type))],
                    actual: t,
                },
                assignment.variable.last_assignment_token.clone(),
            ))
        }
    }

    fn check_unary(
        &mut self,
        operator: &UnaryOperator,
        instruction: &Instruction,
    ) -> Result<Type, ParseError> {
        let instruction_type = self.check_instruction(instruction)?;
        match operator {
            UnaryOperator::Not => match instruction_type {
                Type::Bool => Ok(Type::Bool),
                t => Err(ParseError::new(
                    ParseErrorType::MismatchedType {
                        expected: vec![Type::Bool],
                        actual: t,
                    },
                    instruction.token.clone(),
                )),
            },
            UnaryOperator::Negation => match instruction_type {
                Type::Int => Ok(Type::Int),
                t => Err(ParseError::new(
                    ParseErrorType::MismatchedType {
                        expected: vec![Type::Int],
                        actual: t,
                    },
                    instruction.token.clone(),
                )),
            },
        }
    }

    fn check_binary(
        &mut self,
        operator: &BinaryOperator,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        match operator {
            BinaryOperator::Addition => self.check_addition(left, right),
            BinaryOperator::Subtraction => self.check_subtraction(left, right),
            BinaryOperator::Multiplication => self.check_multiplication(left, right),
            BinaryOperator::Division => self.check_division(left, right),
            BinaryOperator::Modulo => self.check_modulo(left, right),

            BinaryOperator::Equal => self.check_comparison(operator, left, right),
            BinaryOperator::NotEqual => self.check_comparison(operator, left, right),
            BinaryOperator::GreaterThan => self.check_comparison(operator, left, right),
            BinaryOperator::GreaterThanOrEqual => self.check_comparison(operator, left, right),
            BinaryOperator::LessThan => self.check_comparison(operator, left, right),
            BinaryOperator::LessThanOrEqual => self.check_comparison(operator, left, right),

            BinaryOperator::And => self.check_logical(left, right),
            BinaryOperator::Or => self.check_logical(left, right),
        }
    }

    fn check_addition(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::String, Type::String) => Ok(Type::String),
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::String, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::String],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::String, Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_subtraction(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (t1, _) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_multiplication(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::String, Type::Int) => Ok(Type::String),
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::String, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (t1, _) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::String, Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_division(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::Float, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Float],
                    actual: t2,
                },
                right.token.clone(),
            )),

            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_modulo(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_comparison(
        &mut self,
        operator: &BinaryOperator,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Int, Type::Int) => Ok(Type::Bool),
            (Type::Int, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::Float, Type::Float) => Ok(Type::Bool),
            (Type::Float, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Float],
                    actual: t2,
                },
                right.token.clone(),
            )),
            (Type::String, Type::String) | (Type::Bool, Type::Bool) => match operator {
                BinaryOperator::Equal | BinaryOperator::NotEqual => Ok(Type::Bool),
                _ => Err(ParseError::new(
                    ParseErrorType::MismatchedType {
                        expected: vec![Type::Int],
                        actual: Type::Int,
                    },
                    left.token.clone(),
                )),
            },

            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Int, Type::Float],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_logical(
        &mut self,
        left: &Instruction,
        right: &Instruction,
    ) -> Result<Type, ParseError> {
        let left_type = self.check_instruction(left)?;
        let right_type = self.check_instruction(right)?;

        match (left_type, right_type) {
            (Type::Bool, Type::Bool) => Ok(Type::Bool),

            (t1, _t2) => Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Bool],
                    actual: t1,
                },
                left.token.clone(),
            )),
        }
    }

    fn check_type_cast(
        &mut self,
        left_instruction: &Instruction,
        instruction: &Instruction,
        r#type: &Type,
    ) -> Result<Type, ParseError> {
        let instruction_type = self.check_instruction(left_instruction)?;
        match match r#type {
            Type::String => match instruction_type {
                Type::Int => Ok(Type::String),
                Type::Float => Ok(Type::String),
                Type::Bool => Ok(Type::String),
                _ => Err(()),
            },
            Type::Int => match instruction_type {
                Type::String => Ok(Type::Int),
                Type::Float => Ok(Type::Int),
                Type::Bool => Ok(Type::Int),
                _ => Err(()),
            },
            Type::Float => match instruction_type {
                Type::String => Ok(Type::Float),
                Type::Int => Ok(Type::Float),
                Type::Bool => Ok(Type::Float),
                _ => Err(()),
            },
            Type::Bool => match instruction_type {
                Type::String => Ok(Type::Bool),
                Type::Int => Ok(Type::Bool),
                Type::Float => Ok(Type::Bool),
                _ => Err(()),
            },
            _ => Err(()),
        } {
            Ok(t) => Ok(t),
            Err(()) => Err(ParseError::new(
                ParseErrorType::TypeCast {
                    from: instruction_type,
                    to: r#type.clone(),
                },
                instruction.token.clone(),
            )),
        }
    }

    fn check_function(&mut self, function: &Function) -> Result<Type, ParseError> {
        self.environment.add_function(function.clone());

        self.environment.add_scope();
        for parameter in &function.parameters {
            self.environment.insert(parameter.clone());
        }
        let result = self.check_instruction(&function.body);
        self.environment.remove_scope();
        result
    }

    fn check_function_call(
        &mut self,
        name: &str,
        arguments: &[Instruction],
    ) -> Result<Type, ParseError> {
        match &self.environment.functions.get(name).cloned() {
            Some(function) => {
                if function.parameters.len() != arguments.len() {
                    return Err(ParseError::new(
                        ParseErrorType::MismatchedArguments {
                            expected: function.parameters.len(),
                            actual: arguments.len(),
                        },
                        arguments[arguments.len() - 1].token.clone(),
                    ));
                }

                for (parameter, argument) in function.parameters.iter().zip(arguments.iter()) {
                    let argument_type = self.check_instruction(argument)?;
                    if parameter.r#type != argument_type {
                        return Err(ParseError::new(
                            ParseErrorType::MismatchedType {
                                expected: vec![parameter.r#type.clone()],
                                actual: argument_type,
                            },
                            argument.token.clone(),
                        ));
                    }
                }
                Ok(function.return_type.clone())
            }
            None => unreachable!(),
        }
    }

    fn check_conditional(&mut self, conditional: &Conditional) -> Result<Type, ParseError> {
        let condition_type = self.check_instruction(&conditional.condition)?;
        if condition_type != Type::Bool {
            return Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![Type::Bool],
                    actual: condition_type,
                },
                conditional.condition.token.clone(),
            ));
        }
        let result = self.check_instruction(&conditional.r#if)?;
        let result_else = if *conditional.r#else != Instruction::NONE {
            self.check_instruction(&conditional.r#else)?
        } else {
            Type::None
        };

        if result == Type::None || result == result_else {
            Ok(result)
        } else {
            Err(ParseError::new(
                ParseErrorType::MismatchedType {
                    expected: vec![result],
                    actual: result_else,
                },
                conditional.r#else.inner_most().token.clone(),
            ))
        }
    }
}
