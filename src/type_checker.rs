use crate::cli::Args;
use crate::environment::ParseEnvironment;
use crate::error::{ParseError, ParseErrorType, ParseWarning, ParseWarningType};
use crate::instruction::{BuiltIn, Instruction, InstructionType};
use crate::r#type::Type;
use crate::token::Token;
use crate::variable::Variable;

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
            environment: ParseEnvironment::new(),
            success: true,
            args,
        }
    }

    pub fn check(&mut self) -> Result<(), ParseError> {
        for instruction in self.program.clone() {
            match instruction.r#type {
                InstructionType::Test(instruction, _name, _command) => {
                    match self.check_instruction(&instruction) {
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
        match &instruction.r#type {
            InstructionType::StringLiteral(_) => Ok(Type::String),
            InstructionType::RegexLiteral(_) => Ok(Type::Regex),

            InstructionType::BuiltIn(instruction) => self.check_builtin(instruction),

            InstructionType::Block(instructions) => {
                let mut result = Type::None;
                self.environment.add_scope();
                for instruction in instructions {
                    result = self.check_instruction(&instruction)?;
                }
                self.environment.remove_scope();
                Ok(result)
            }

            InstructionType::For(assignment, statement) => {
                self.environment.add_scope();
                self.check_instruction(&assignment)?;
                let result = self.check_instruction(&statement)?;
                self.environment.remove_scope();
                Ok(result)
            }

            InstructionType::Variable(variable) => {
                self.environment.insert(variable.clone());
                Ok(variable.r#type)
            }

            InstructionType::Assignment(variable, instruction) => {
                self.check_assignment(&variable, &instruction)
            }

            InstructionType::IterableAssignment(variable, instruction) => {
                self.check_iterable_assignment(&variable, &instruction)
            }

            InstructionType::Addition { left, right } => {
                self.check_addition(left, right, &instruction.token)
            }

            InstructionType::None => {
                ParseWarning::new(
                    ParseWarningType::TrailingSemicolon,
                    instruction.token.clone(),
                    "Remove the trailing semicolon",
                )
                .print(self.args.disable_warnings);
                Ok(Type::None)
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn check_builtin(&mut self, built_in: &BuiltIn) -> Result<Type, ParseError> {
        match built_in {
            BuiltIn::Input(instruction) => self.check_instruction(&instruction),
            BuiltIn::Output(instruction) => self.check_instruction(&instruction),
            BuiltIn::Print(instruction) => self.check_instruction(&instruction),
            BuiltIn::Println(instruction) => self.check_instruction(&instruction),
        }
    }

    fn check_assignment(
        &mut self,
        variable: &Variable,
        instruction: &Instruction,
    ) -> Result<Type, ParseError> {
        let variable_name = &variable.name;
        let variable_type = variable.r#type;

        let instruction_type = self.check_instruction(&instruction.clone())?;

        if variable_type != instruction_type {
            return Err(ParseError::new(
                ParseErrorType::MismatchedType{expected: variable_type, actual: instruction_type},
                instruction.token.clone(),
                format!(
                    "Expected expression of type {:?} because of \"{variable_name}\" type but found {:?}",
                    variable_type, instruction_type
                ),
            ));
        }

        self.environment.insert(variable.clone());
        Ok(variable_type)
    }

    fn check_iterable_assignment(
        &mut self,
        variable: &Variable,
        instruction: &Instruction,
    ) -> Result<Type, ParseError> {
        let variable_type = variable.r#type;
        match self.check_instruction(&instruction) {
            Ok(Type::Regex) => match variable_type {
                Type::String => {
                    self.environment.insert(variable.clone());
                    Ok(variable_type)
                }
                _ => Err(ParseError::new(
                ParseErrorType::MismatchedType{expected: Type::Regex, actual: variable_type},
                    instruction.token.clone(),
                    format!(
                        "Expected expression of type {:?} because of the variable type but found {:?}",
                        variable_type, instruction.r#type
                    ),
                )),
            },
            Ok(t) => Err(ParseError::new(
                ParseErrorType::MismatchedType{expected: Type::Iterable, actual: t},
                instruction.token.clone(),
                format!("Expected an iterable type but found a {t:?}"),
            )),
            Err(e) => Err(e),
        }
    }

    fn check_addition(
        &mut self,
        left: &Instruction,
        right: &Instruction,
        token: &Token,
    ) -> Result<Type, ParseError> {
        let left = self.check_instruction(left)?;
        let right = self.check_instruction(right)?;

        match (left, right) {
            (Type::String, Type::String) => Ok(Type::String),
            (t1, t2) => Err(ParseError::new(
                ParseErrorType::MismatchedTypeBinary {
                    expected_left: Type::String,
                    actual_left: t1,
                    expected_right: Type::String,
                    actual_right: t2,
                },
                token.clone(),
                format!("Addition is not supported between `{}` and `{}`", t1, t2),
            )),
        }
    }
}
