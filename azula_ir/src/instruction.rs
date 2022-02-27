use std::fmt::{self, Display, Formatter};

use azula_type::prelude::AzulaType;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction<'a> {
    Load(String, usize, AzulaType<'a>),
    Store(String, Value, AzulaType<'a>),
    LoadArg(usize, usize, AzulaType<'a>),
    ConstInt(i64, usize),
    ConstTrue(usize),
    ConstFalse(usize),
    Add(Value, Value, usize),
    Sub(Value, Value, usize),
    Mul(Value, Value, usize),
    Div(Value, Value, usize),
    Mod(Value, Value, usize),
    Or(Value, Value, usize),
    And(Value, Value, usize),
    Eq(Value, Value, usize),
    Neq(Value, Value, usize),
    Gt(Value, Value, usize),
    Gte(Value, Value, usize),
    Lt(Value, Value, usize),
    Lte(Value, Value, usize),
    Not(Value, usize),
    Return(Option<Value>),
    FunctionCall(String, Vec<Value>, usize),
    Jcond(Value, String, String),
    Jump(String),
}

impl<'a> Display for Instruction<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Load(name, dest, typ) => write!(f, "%{}: load @{} {:?}", dest, name, typ),
            Instruction::Store(name, value, typ) => {
                write!(f, "store @{} {} {:?}", name, value, typ)
            }
            Instruction::LoadArg(arg, dest, typ) => {
                write!(f, "%{}: load_arg %{} {:?}", dest, arg, typ)
            }
            Instruction::ConstInt(val, dest) => write!(f, "%{}: const_int {}", dest, val),
            Instruction::ConstTrue(dest) => write!(f, "%{}: const_true", dest),
            Instruction::ConstFalse(dest) => write!(f, "%{}: const_false", dest),
            Instruction::Add(val1, val2, dest) => write!(f, "%{}: add {} {}", dest, val1, val2),
            Instruction::Sub(val1, val2, dest) => write!(f, "%{}: sub {} {}", dest, val1, val2),
            Instruction::Mul(val1, val2, dest) => write!(f, "%{}: mul {} {}", dest, val1, val2),
            Instruction::Div(val1, val2, dest) => write!(f, "%{}: div {} {}", dest, val1, val2),
            Instruction::Mod(val1, val2, dest) => write!(f, "%{}: mod {} {}", dest, val1, val2),
            Instruction::Or(val1, val2, dest) => write!(f, "%{}: or {} {}", dest, val1, val2),
            Instruction::And(val1, val2, dest) => write!(f, "%{}: and {} {}", dest, val1, val2),
            Instruction::Eq(val1, val2, dest) => write!(f, "%{}: eq {} {}", dest, val1, val2),
            Instruction::Neq(val1, val2, dest) => write!(f, "%{}: neq {} {}", dest, val1, val2),
            Instruction::Gt(val1, val2, dest) => write!(f, "%{}: gt {} {}", dest, val1, val2),
            Instruction::Gte(val1, val2, dest) => write!(f, "%{}: gte {} {}", dest, val1, val2),
            Instruction::Lt(val1, val2, dest) => write!(f, "%{}: lt {} {}", dest, val1, val2),
            Instruction::Lte(val1, val2, dest) => write!(f, "%{}: lte {} {}", dest, val1, val2),
            Instruction::Not(val, dest) => write!(f, "%{}: not {}", dest, val),
            Instruction::Return(val) => write!(
                f,
                "ret {}",
                match val {
                    Some(val) => format!("{}", val),
                    None => "".to_string(),
                }
            ),
            Instruction::FunctionCall(name, args, dest) => {
                write!(f, "%{}: function_call @{} {:?}", dest, name, args)
            }
            Instruction::Jcond(cond, true_block, false_block) => {
                write!(f, "jcond {} {} {}", cond, true_block, false_block)
            }
            Instruction::Jump(block) => {
                write!(f, "jump {}", block)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    LiteralInteger(i64),
    LiteralBoolean(bool),
    Local(usize),
    Global(usize),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::LiteralInteger(val) => write!(f, "{}", val),
            Value::LiteralBoolean(val) => write!(f, "{}", val),
            Value::Local(val) => write!(f, "%{}", val),
            Value::Global(val) => write!(f, "${}", val),
        }
    }
}
