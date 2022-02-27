use core::fmt;
use std::{collections::HashMap, fmt::Display};

use azula_type::prelude::AzulaType;

use crate::instruction::{Instruction, Value};

pub struct Module<'a> {
    pub name: &'a str,
    pub functions: HashMap<&'a str, Function<'a>>,
    pub strings: Vec<String>,
}

impl<'a> Module<'a> {
    pub fn new(name: &'a str) -> Self {
        Module {
            name,
            functions: HashMap::new(),
            strings: vec![],
        }
    }

    pub fn add_function(&mut self, name: &'a str, function: Function<'a>) {
        self.functions.insert(name, function);
    }

    pub fn add_string(&mut self, val: String) -> Value {
        self.strings.push(val);

        Value::Global(self.strings.len() - 1)
    }
}

impl<'a> Display for Module<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Module: {}", self.name).unwrap();
        writeln!(f, "Strings:").unwrap();
        for (index, string) in self.strings.iter().enumerate() {
            writeln!(f, "\t{}: {}", index, string).unwrap();
        }

        writeln!(f).unwrap();

        for (name, func) in &self.functions {
            writeln!(f, "func {}:", name).unwrap();
            writeln!(f, "\tArguments:").unwrap();
            for (var, typ) in &func.arguments {
                writeln!(f, "\t\t{} {:?}", var, typ).unwrap();
            }
            writeln!(f, "\tReturns: {:?}", func.returns).unwrap();
            writeln!(f, "\tVariables:").unwrap();
            for (var, typ) in &func.variables {
                writeln!(f, "\t\t{} {:?}", var, typ).unwrap();
            }

            for (name, block) in &func.blocks {
                writeln!(f, "\t{}:", name).unwrap();
                for instruction in &block.instructions {
                    writeln!(f, "\t\t{}", instruction).unwrap();
                }
            }
        }

        Ok(())
    }
}

pub struct Function<'a> {
    pub blocks: Vec<(String, Block<'a>)>,
    pub variables: HashMap<String, AzulaType<'a>>,
    pub arguments: Vec<(String, AzulaType<'a>)>,
    pub returns: AzulaType<'a>,

    tmp_var_index: usize,
    pub if_block_index: usize,

    pub current_block: String,
}

impl<'a> Function<'a> {
    pub fn new() -> Self {
        let mut blocks = vec![];
        blocks.push((
            "entry".to_string(),
            Block {
                instructions: vec![],
            },
        ));
        Self {
            blocks,
            variables: HashMap::new(),
            arguments: vec![],
            returns: AzulaType::Void,
            tmp_var_index: 0,
            if_block_index: 0,
            current_block: "entry".to_string(),
        }
    }

    pub fn load(&mut self, variable: String, typ: AzulaType<'a>) -> Value {
        self.add_instruction(Instruction::Load(variable, self.tmp_var_index, typ));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn load_arg(&mut self, arg: usize, typ: AzulaType<'a>) -> Value {
        self.add_instruction(Instruction::LoadArg(arg, self.tmp_var_index, typ));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn store(&mut self, variable: String, val: Value, typ: AzulaType<'a>) {
        self.add_instruction(Instruction::Store(variable, val, typ));
    }

    pub fn const_int(&mut self, val: i64) -> Value {
        self.add_instruction(Instruction::ConstInt(val, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn const_true(&mut self) -> Value {
        self.add_instruction(Instruction::ConstTrue(self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn const_false(&mut self) -> Value {
        self.add_instruction(Instruction::ConstFalse(self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn add(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Add(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn sub(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Sub(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn mul(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Mul(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn div(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Div(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn modulus(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Mod(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn or(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Or(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn and(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::And(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn eq(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Eq(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn neq(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Neq(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn gt(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Gt(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn gte(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Gte(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn lt(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Lt(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn lte(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Lte(val1, val2, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn not(&mut self, val: Value) -> Value {
        self.add_instruction(Instruction::Not(val, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn ret(&mut self, val: Option<Value>) {
        self.add_instruction(Instruction::Return(val));
    }

    pub fn function_call(&mut self, name: String, args: Vec<Value>) -> Value {
        self.add_instruction(Instruction::FunctionCall(name, args, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn jcond(&mut self, cond: Value, true_block: String, end_block: String) {
        self.add_instruction(Instruction::Jcond(cond, true_block, end_block));
    }

    fn add_instruction(&mut self, instruction: Instruction<'a>) {
        self.blocks
            .iter_mut()
            .find(|(x, _)| x.clone() == self.current_block)
            .unwrap()
            .1
            .instructions
            .push(instruction);
    }
}

#[derive(Clone)]
pub struct Block<'a> {
    pub instructions: Vec<Instruction<'a>>,
}

impl<'a> Block<'a> {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
        }
    }
}
