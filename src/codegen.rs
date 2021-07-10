use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue},
};

use crate::parser::ast::{Expr, Opcode, Statement};

pub struct Compiler<'a> {
    pub context: &'a Context,
    pub builder: &'a Builder<'a>,
    pub module: Module<'a>,

    pub ptrs: HashMap<String, PointerValue<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn gen_stmt(self: &mut Compiler<'a>, current_func: &FunctionValue<'a>, stmt: Statement) {
        match stmt {
            Statement::Return(expr) => {
                if expr.is_none() {
                    self.builder.build_return(None);
                    return;
                }

                self.builder
                    .build_return(Some(&*self.gen_expr(current_func, &expr.unwrap()).unwrap()));
            }
            Statement::Let(_mutability, name, value) => {
                let value = *self.gen_expr(current_func, &value).unwrap();
                // Need to typecheck the value
                let ptr = self
                    .builder
                    .build_alloca(value.as_basic_value_enum().get_type(), &name);
                self.builder.build_store(ptr, value);

                self.ptrs.insert(name, ptr);
            }
            Statement::Expression(expr) => {
                self.gen_expr(current_func, &expr);
            }
            Statement::If(cond, stmts) => {
                let val = self
                    .gen_expr(current_func, &cond)
                    .unwrap()
                    .as_basic_value_enum()
                    .into_int_value();
                let end = self.context.append_basic_block(*current_func, "end");
                let block = self.context.append_basic_block(*current_func, "btrue");
                self.builder.build_conditional_branch(val, block, end);
                self.builder.position_at_end(block);
                for stmt in stmts.clone() {
                    self.gen_stmt(current_func, *stmt.clone());
                }

                let x = stmts.last().unwrap();
                match *x.clone() {
                    Statement::Return(_) => (),
                    _ => {
                        self.builder.build_unconditional_branch(end);
                    }
                };

                self.builder.position_at_end(end);
            }
            _ => panic!("uh oh"),
        }
    }

    fn gen_expr(
        self: &mut Compiler<'a>,
        current_func: &FunctionValue<'a>,
        expr: &Expr,
    ) -> Option<Box<BasicValueEnum<'a>>> {
        match expr {
            Expr::Number(i) => Some(Box::new(if (*i - i.round()).abs() < 0.01 {
                self.context
                    .i32_type()
                    .const_int(*i as u64, false)
                    .as_basic_value_enum()
            } else {
                self.context
                    .f32_type()
                    .const_float(*i as f64)
                    .as_basic_value_enum()
            })),
            Expr::String(s) => {
                return Some(Box::new(
                    self.builder
                        .build_global_string_ptr(s, "string")
                        .as_basic_value_enum(),
                ));
            }
            Expr::Op(expr1, opcode, expr2) => {
                let result1 = *self.gen_expr(current_func, expr1).unwrap();
                let result2 = *self.gen_expr(current_func, expr2).unwrap();

                if result1.as_basic_value_enum().get_type().is_int_type() {
                    let result1 = result1.into_int_value();
                    let result2 = result2.into_int_value();
                    return match opcode {
                        Opcode::Add => Some(Box::new(
                            self.builder
                                .build_int_add(result1, result2, "add")
                                .as_basic_value_enum(),
                        )),
                        Opcode::Sub => Some(Box::new(
                            self.builder
                                .build_int_sub(result1, result2, "sub")
                                .as_basic_value_enum(),
                        )),
                        Opcode::Mul => Some(Box::new(
                            self.builder
                                .build_int_mul(result1, result2, "mul")
                                .as_basic_value_enum(),
                        )),
                        Opcode::Div => Some(Box::new(
                            self.builder
                                .build_int_signed_div(result1, result2, "div")
                                .as_basic_value_enum(),
                        )),
                        Opcode::Rem => Some(Box::new(
                            self.builder
                                .build_int_signed_rem(result1, result2, "rem")
                                .as_basic_value_enum(),
                        )),
                        Opcode::Eq => Some(Box::new(
                            self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::EQ,
                                    result1,
                                    result2,
                                    "eq",
                                )
                                .as_basic_value_enum(),
                        )),
                        Opcode::NotEq => Some(Box::new(
                            self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::NE,
                                    result1,
                                    result2,
                                    "noteq",
                                )
                                .as_basic_value_enum(),
                        )),
                        Opcode::LessThan => Some(Box::new(
                            self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::SLT,
                                    result1,
                                    result2,
                                    "lessthan",
                                )
                                .as_basic_value_enum(),
                        )),
                        Opcode::GreaterThan => Some(Box::new(
                            self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::SGT,
                                    result1,
                                    result2,
                                    "lessthan",
                                )
                                .as_basic_value_enum(),
                        )),
                        Opcode::LessEqual => Some(Box::new(
                            self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::SLE,
                                    result1,
                                    result2,
                                    "lessthan",
                                )
                                .as_basic_value_enum(),
                        )),
                        Opcode::GreaterEqual => Some(Box::new(
                            self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::SGE,
                                    result1,
                                    result2,
                                    "lessthan",
                                )
                                .as_basic_value_enum(),
                        )),
                        Opcode::Or => Some(Box::new(
                            self.builder
                                .build_or(result1, result2, "or")
                                .as_basic_value_enum(),
                        )),
                        Opcode::And => Some(Box::new(
                            self.builder
                                .build_and(result1, result2, "and")
                                .as_basic_value_enum(),
                        )),
                    };
                } else if result1.as_basic_value_enum().get_type().is_float_type() {
                    let result1 = result1.into_float_value();
                    let result2 = result2.into_float_value();
                    return match opcode {
                        Opcode::Add => Some(Box::new(
                            self.builder
                                .build_float_add(result1, result2, "add")
                                .as_basic_value_enum(),
                        )),
                        Opcode::Sub => Some(Box::new(
                            self.builder
                                .build_float_sub(result1, result2, "sub")
                                .as_basic_value_enum(),
                        )),
                        Opcode::Mul => Some(Box::new(
                            self.builder
                                .build_float_mul(result1, result2, "mul")
                                .as_basic_value_enum(),
                        )),
                        Opcode::Div => Some(Box::new(
                            self.builder
                                .build_float_div(result1, result2, "div")
                                .as_basic_value_enum(),
                        )),
                        _ => panic!("unimplemented float opcode"),
                    };
                }

                None
            }
            Expr::Identifier(name) => Some(Box::new(
                self.builder
                    .build_load(*self.ptrs.get(name).unwrap(), "load")
                    .as_basic_value_enum(),
            )),
            Expr::FunctionCall(name, values) => {
                let llvm_vals = values
                    .iter()
                    .map(|expr| {
                        self.gen_expr(current_func, expr)
                            .unwrap()
                            .as_basic_value_enum()
                    })
                    .collect::<Vec<_>>();

                let mut func = self.module.get_function(&name);
                if name == "print" {
                    func = match llvm_vals.first().unwrap().as_basic_value_enum().get_type() {
                        BasicTypeEnum::IntType(_) => {
                            // let size = llvm_vals
                            //     .first()
                            //     .unwrap()
                            //     .as_basic_value_enum()
                            //     .get_type()
                            //     .into_int_type()
                            //     .get_bit_width();
                            self.module.get_function("print_int")
                        }
                        BasicTypeEnum::FloatType(_) => self.module.get_function("print_float"),
                        BasicTypeEnum::PointerType(s) => {
                            if s.get_element_type().is_int_type() {
                                self.module.get_function("print_string")
                            } else {
                                panic!("can't print this type")
                            }
                        }
                        _ => panic!("can't print this type"),
                    };
                }

                self.builder
                    .build_call(func.unwrap(), &llvm_vals, "call")
                    .try_as_basic_value()
                    .left()
                    .map(Box::new)
            }
            Expr::Boolean(b) => Some(Box::new(
                self.context
                    .bool_type()
                    .const_int(*b as u64, false)
                    .as_basic_value_enum(),
            )),
        }
    }
}
