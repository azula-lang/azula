mod value;

use azula_ir::prelude::*;
use std::collections::HashMap;
use value::Value;

pub struct VM {
    globals: Vec<Value>,
}

struct Frame {
    locals: HashMap<usize, Value>,
    named_locals: HashMap<String, Value>,
}

impl<'a> VM {
    pub fn new() -> Self {
        VM { globals: vec![] }
    }

    pub fn run(self: &mut VM, module: Module<'a>) -> Result<Value, String> {
        module.strings.iter().for_each(|s| {
            self.globals
                .push(Value::Pointer(Box::new(Value::String(s.clone()))));
        });

        let mut return_val = Value::Null;
        for (name, func) in module.functions {
            if name == "main" {
                for (_, block) in &func.blocks {
                    return_val = self.execute(block).unwrap();
                }
            }
        }

        Ok(return_val)
    }

    pub fn execute(&mut self, block: &Block) -> Result<Value, String> {
        let mut frame = Frame {
            named_locals: HashMap::new(),
            locals: HashMap::new(),
        };

        for instruction in &block.instructions {
            match instruction {
                Instruction::Load(name, dest, _) => {
                    frame.locals.insert(*dest, frame.named_locals[name].clone());
                }
                Instruction::LoadGlobal(_, _, _) => todo!(),
                Instruction::Store(name, value, _) => {
                    frame.named_locals.insert(
                        name.clone(),
                        match value {
                            azula_ir::prelude::Value::LiteralInteger(i) => Value::Integer(*i),
                            azula_ir::prelude::Value::LiteralBoolean(_) => todo!(),
                            azula_ir::prelude::Value::Local(i) => frame.locals[i].clone(),
                            azula_ir::prelude::Value::Global(g) => self.globals[g.clone()].clone(),
                        },
                    );
                }
                Instruction::LoadArg(_, _, _) => todo!(),
                Instruction::ConstInt(_, _) => todo!(),
                Instruction::ConstFloat(_, _) => todo!(),
                Instruction::ConstTrue(_) => todo!(),
                Instruction::ConstFalse(_) => todo!(),
                Instruction::ConstNull(_) => todo!(),
                Instruction::Add(_, _, _) => todo!(),
                Instruction::Sub(_, _, _) => todo!(),
                Instruction::Mul(_, _, _) => todo!(),
                Instruction::Div(_, _, _) => todo!(),
                Instruction::Mod(_, _, _) => todo!(),
                Instruction::Pow(_, _, _) => todo!(),
                Instruction::Or(_, _, _) => todo!(),
                Instruction::And(_, _, _) => todo!(),
                Instruction::Eq(_, _, _) => todo!(),
                Instruction::Neq(_, _, _) => todo!(),
                Instruction::Gt(_, _, _) => todo!(),
                Instruction::Gte(_, _, _) => todo!(),
                Instruction::Lt(_, _, _) => todo!(),
                Instruction::Lte(_, _, _) => todo!(),
                Instruction::Not(_, _) => todo!(),
                Instruction::Return(_) => {}
                Instruction::FunctionCall(name, params, dest) => {
                    let mut args = vec![];
                    for param in params {
                        args.push(match param {
                            azula_ir::prelude::Value::LiteralInteger(i) => Value::Integer(*i),
                            azula_ir::prelude::Value::LiteralBoolean(_) => todo!(),
                            azula_ir::prelude::Value::Local(i) => frame.locals[i].clone(),
                            azula_ir::prelude::Value::Global(g) => self.globals[g.clone()].clone(),
                        })
                    }

                    let mut ret_val = Value::Null;
                    match name.as_str() {
                        "printf" => {
                            println!("{}", args[1]);
                        }
                        "open" => {
                            let file = std::fs::read_to_string(match args[0].clone() {
                                Value::Pointer(p) => match *p {
                                    Value::String(s) => s,
                                    _ => todo!(),
                                },
                                _ => todo!(),
                            })
                            .unwrap();
                            ret_val = Value::Pointer(Box::new(Value::String(file)));
                        }
                        _ => todo!(),
                    }

                    frame.locals.insert(*dest, ret_val);
                }
                Instruction::Jcond(_, _, _) => todo!(),
                Instruction::Jump(_) => todo!(),
                Instruction::Pointer(_, _) => todo!(),
                Instruction::CreateArray(_, _, _) => todo!(),
                Instruction::StoreElement(_, _, _, _) => todo!(),
                Instruction::AccessElement(_, _, _, _) => todo!(),
                Instruction::CreateStruct(_, _, _) => todo!(),
                Instruction::StoreStructMember(_, _, _, _) => todo!(),
                Instruction::AccessStructMember(_, _, _, _, _) => todo!(),
                Instruction::Cast(_, _, _) => todo!(),
            }
        }

        Ok(Value::Null)
    }
}
