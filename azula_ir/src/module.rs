use core::fmt;
use std::{collections::HashMap, fmt::Display, rc::Rc};

use azula_type::prelude::AzulaType;

use crate::instruction::{Instruction, Value};

pub struct Module<'a> {
    pub name: &'a str,
    pub functions: HashMap<String, Function<'a>>,
    pub extern_functions: HashMap<&'a str, ExternFunction<'a>>,
    pub strings: Vec<String>,
    pub global_values: HashMap<String, GlobalValue>,
    pub structs: HashMap<&'a str, Struct<'a>>,
    pub enums: HashMap<String, Vec<String>>,
}

impl<'a> Module<'a> {
    pub fn new(name: &'a str) -> Self {
        let mut extern_functions = HashMap::new();
        extern_functions.insert(
            "printf",
            ExternFunction {
                varargs: true,
                arguments: vec![AzulaType::Pointer(Rc::new(AzulaType::Str))],
                returns: AzulaType::Void,
            },
        );
        extern_functions.insert(
            "strlen",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str],
                returns: AzulaType::Int,
            },
        );
        extern_functions.insert(
            "strcmp",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Str],
                returns: AzulaType::Int,
            },
        );
        extern_functions.insert(
            "malloc",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Int],
                returns: AzulaType::Str,
            },
        );
        extern_functions.insert(
            "memcpy",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Str, AzulaType::Int],
                returns: AzulaType::Str,
            },
        );
        extern_functions.insert(
            "free",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str],
                returns: AzulaType::Void,
            },
        );
        extern_functions.insert(
            "realloc",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Int],
                returns: AzulaType::Str,
            },
        );
        extern_functions.insert(
            "ptr_read_int",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str],
                returns: AzulaType::Int,
            },
        );
        extern_functions.insert(
            "ptr_write_int",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Int],
                returns: AzulaType::Void,
            },
        );
        extern_functions.insert(
            "ptr_read_str",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str],
                returns: AzulaType::Str,
            },
        );
        extern_functions.insert(
            "ptr_write_str",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Str],
                returns: AzulaType::Void,
            },
        );
        extern_functions.insert(
            "ptr_add",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Int],
                returns: AzulaType::Str,
            },
        );
        extern_functions.insert(
            "fopen",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Str],
                returns: AzulaType::Str,
            },
        );
        extern_functions.insert(
            "fclose",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str],
                returns: AzulaType::Int,
            },
        );
        extern_functions.insert(
            "fseek",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Int, AzulaType::Int],
                returns: AzulaType::Int,
            },
        );
        extern_functions.insert(
            "ftell",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str],
                returns: AzulaType::Int,
            },
        );
        extern_functions.insert(
            "fread",
            ExternFunction {
                varargs: false,
                arguments: vec![AzulaType::Str, AzulaType::Int, AzulaType::Int, AzulaType::Str],
                returns: AzulaType::Int,
            },
        );
        Module {
            name,
            functions: HashMap::new(),
            extern_functions,
            strings: vec![],
            global_values: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, name: String, function: Function<'a>) {
        self.functions.insert(name, function);
    }

    pub fn add_extern_function(&mut self, name: &'a str, function: ExternFunction<'a>) {
        self.extern_functions.insert(name, function);
    }

    pub fn add_string(&mut self, val: String) -> Value {
        self.strings.push(val);

        Value::Global(self.strings.len() - 1)
    }

    pub fn add_struct(&mut self, name: &'a str, struc: Struct<'a>) {
        self.structs.insert(name, struc);
    }

    pub fn add_enum(&mut self, name: String, variants: Vec<String>) {
        self.enums.insert(name, variants);
    }
}

impl<'a> Display for Module<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Module: {}", self.name).unwrap();
        writeln!(f, "Strings:").unwrap();
        for (index, string) in self.strings.iter().enumerate() {
            writeln!(f, "\t{}: {}", index, string).unwrap();
        }

        writeln!(f, "Structs:").unwrap();
        for (name, struc) in self.structs.iter() {
            writeln!(f, "\t{}: {:?}", name, struc).unwrap();
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
    pub match_block_index: usize,

    pub current_block: String,
}

pub struct ExternFunction<'a> {
    pub varargs: bool,
    pub arguments: Vec<AzulaType<'a>>,
    pub returns: AzulaType<'a>,
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
            match_block_index: 0,
            current_block: "entry".to_string(),
        }
    }

    pub fn load(&mut self, variable: String, typ: AzulaType<'a>) -> Value {
        self.add_instruction(Instruction::Load(variable, self.tmp_var_index, typ));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn load_global(&mut self, variable: String, typ: AzulaType<'a>) -> Value {
        self.add_instruction(Instruction::LoadGlobal(variable, self.tmp_var_index, typ));

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

    pub fn const_float(&mut self, val: f64) -> Value {
        self.add_instruction(Instruction::ConstFloat(val, self.tmp_var_index));

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

    pub fn const_null(&mut self) -> Value {
        self.add_instruction(Instruction::ConstNull(self.tmp_var_index));

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

    pub fn pow(&mut self, val1: Value, val2: Value) -> Value {
        self.add_instruction(Instruction::Pow(val1, val2, self.tmp_var_index));

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

    pub fn cast(&mut self, val: Value, typ: AzulaType<'a>) -> Value {
        self.add_instruction(Instruction::Cast(val, typ, self.tmp_var_index));
        self.tmp_var_index += 1;
        Value::Local(self.tmp_var_index - 1)
    }

    pub fn ptr(&mut self, val: String) -> Value {
        self.add_instruction(Instruction::Pointer(val, self.tmp_var_index));

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

    pub fn jump(&mut self, block: String) {
        self.add_instruction(Instruction::Jump(block));
    }

    pub fn create_array(&mut self, typ: AzulaType<'a>, size: usize) -> Value {
        self.add_instruction(Instruction::CreateArray(typ, size, self.tmp_var_index));

        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn store_element(&mut self, array: Value, index: Value, value: Value, elem_type: AzulaType<'a>) {
        self.add_instruction(Instruction::StoreElement(array, index, value, elem_type));
    }

    pub fn access_element(&mut self, array: Value, index: Value, elem_type: AzulaType<'a>) -> Value {
        self.add_instruction(Instruction::AccessElement(array, index, self.tmp_var_index, elem_type));
        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn create_struct(&mut self, struc: String, values: Vec<Value>) -> Value {
        self.add_instruction(Instruction::CreateStruct(struc, values, self.tmp_var_index));
        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn access_struct_member(&mut self, struc: Value, index: usize, resolve: bool, struct_name: String) -> Value {
        self.add_instruction(Instruction::AccessStructMember(
            struc,
            index,
            self.tmp_var_index,
            resolve,
            struct_name,
        ));
        self.tmp_var_index += 1;

        Value::Local(self.tmp_var_index - 1)
    }

    pub fn store_struct_member(&mut self, struc: Value, index: usize, value: Value, struct_name: String) {
        self.add_instruction(Instruction::StoreStructMember(struc, index, value, struct_name));
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

#[derive(Clone, Debug)]
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

#[derive(Debug, Clone)]
pub enum GlobalValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(usize),
    Array(Vec<GlobalValue>),
}

#[derive(Debug, Clone)]
pub struct Struct<'a> {
    pub name: &'a str,
    pub attributes: Vec<(AzulaType<'a>, &'a str)>,
}
