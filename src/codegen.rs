use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum, PointerType},
    values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};

use crate::parser::ast::{Expr, Opcode, Statement, Type};

pub struct Compiler<'a> {
    pub context: &'a Context,
    pub builder: &'a Builder<'a>,
    pub module: Module<'a>,

    pub str_type: PointerType<'a>,
    pub ptrs: HashMap<String, PointerValue<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn add_print_funcs(self: &mut Compiler<'a>) {
        let print_int = self.module.add_function(
            "print_int",
            self.context
                .void_type()
                .fn_type(&[self.context.i32_type().as_basic_type_enum()], false),
            Some(Linkage::Private),
        );
        let printf = self.module.add_function(
            "printf",
            self.context
                .i32_type()
                .fn_type(&[self.str_type.as_basic_type_enum()], true),
            Some(Linkage::External),
        );
        let main_builder = self.context.create_builder();
        let main_entry = self.context.append_basic_block(print_int, "entry");
        main_builder.position_at_end(main_entry);
        let global = main_builder.build_global_string_ptr("%d\n", "format");
        main_builder.build_call(
            printf,
            &[
                global.as_basic_value_enum(),
                print_int.get_first_param().unwrap(),
            ],
            "print",
        );
        main_builder.build_return(None);

        let print_float = self.module.add_function(
            "print_float",
            self.context
                .void_type()
                .fn_type(&[self.context.f32_type().as_basic_type_enum()], false),
            Some(Linkage::Private),
        );
        let entry = self.context.append_basic_block(print_float, "entry");
        main_builder.position_at_end(entry);
        let global_f = main_builder.build_global_string_ptr("%f\n", "format");
        main_builder.build_call(
            printf,
            &[
                global_f.as_basic_value_enum(),
                print_float.get_first_param().unwrap(),
            ],
            "print",
        );
        main_builder.build_return(None);

        let print_string = self.module.add_function(
            "print_string",
            self.context.void_type().fn_type(
                &[self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum()],
                false,
            ),
            Some(Linkage::Private),
        );
        let entry = self.context.append_basic_block(print_string, "entry");
        main_builder.position_at_end(entry);
        let global_f = main_builder.build_global_string_ptr("%s\n", "format");
        main_builder.build_call(
            printf,
            &[
                global_f.as_basic_value_enum(),
                print_string.get_first_param().unwrap(),
            ],
            "print",
        );
        main_builder.build_return(None);
    }

    pub fn gen(self: &mut Compiler<'a>, parse_tree: Vec<Box<Statement>>) {
        for statement in parse_tree {
            match *statement {
                Statement::Function(name, params, return_type, body) => {
                    let llvm_params = if let Some(x) = &params {
                        x.iter()
                            .map(|(typ, _)| match *typ {
                                Type::Integer(size) => match size {
                                    32 => self.context.i32_type().as_basic_type_enum(),
                                    64 => self.context.i64_type().as_basic_type_enum(),
                                    _ => panic!("unimplemented int size"),
                                },
                                Type::Float(size) => match size {
                                    32 => self.context.f32_type().as_basic_type_enum(),
                                    64 => self.context.f64_type().as_basic_type_enum(),
                                    _ => panic!("unimplemented float size"),
                                },
                                Type::Boolean => self.context.bool_type().as_basic_type_enum(),
                                Type::String => self.str_type.as_basic_type_enum(),
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    let mut linkage = Some(Linkage::Private);
                    if name == "main" {
                        linkage = None;
                    }

                    let mut llvm_ret: AnyTypeEnum = self.context.void_type().as_any_type_enum();
                    if let Some(ret) = return_type {
                        llvm_ret = match ret {
                            Type::Integer(size) => match size {
                                32 => self.context.i32_type().as_any_type_enum(),
                                64 => self.context.i64_type().as_any_type_enum(),
                                _ => panic!("unimplemented int size"),
                            },
                            Type::Float(size) => match size {
                                32 => self.context.f32_type().as_any_type_enum(),
                                64 => self.context.f64_type().as_any_type_enum(),
                                _ => panic!("unimplemented float size"),
                            },
                            Type::Boolean => self.context.bool_type().as_any_type_enum(),
                            Type::String => self.str_type.as_any_type_enum(),
                        };
                    }

                    let mut function_type = self.context.void_type().fn_type(&[], false);
                    if llvm_ret.is_int_type() {
                        function_type = llvm_ret.into_int_type().fn_type(&llvm_params, false);
                    }
                    if llvm_ret.is_float_type() {
                        function_type = llvm_ret.into_float_type().fn_type(&llvm_params, false);
                    }
                    if llvm_ret.is_pointer_type() {
                        function_type = llvm_ret.into_pointer_type().fn_type(&llvm_params, false);
                    }
                    if llvm_ret.is_void_type() {
                        function_type = llvm_ret.into_void_type().fn_type(&llvm_params, false)
                    }

                    let llvm_func = self
                        .module
                        .add_function(name.as_str(), function_type, linkage);

                    let entry = self.context.append_basic_block(llvm_func, "entry");
                    self.builder.position_at_end(entry);

                    if let Some(p) = &params {
                        for (index, (typ, name)) in p.iter().enumerate() {
                            let alloca = self.builder.build_alloca(
                                match *typ {
                                    Type::Integer(size) => match size {
                                        32 => self.context.i32_type().as_basic_type_enum(),
                                        64 => self.context.i64_type().as_basic_type_enum(),
                                        _ => panic!("unimplemented int size"),
                                    },
                                    Type::Float(size) => match size {
                                        32 => self.context.f32_type().as_basic_type_enum(),
                                        64 => self.context.f64_type().as_basic_type_enum(),
                                        _ => panic!("unimplemented float size"),
                                    },
                                    Type::Boolean => self.context.bool_type().as_basic_type_enum(),
                                    Type::String => self.str_type.as_basic_type_enum(),
                                },
                                "param",
                            );
                            self.builder
                                .build_store(alloca, llvm_func.get_params()[index]);
                            self.ptrs.insert(name.clone(), alloca);
                        }
                    }

                    for stmt in &body {
                        self.gen_stmt(&llvm_func, *stmt.clone());
                    }

                    let x = body.last().unwrap();
                    match *x.clone() {
                        Statement::Return(_) => continue,
                        _ => self.builder.build_return(None),
                    };
                }
                _ => panic!("non-function at top level"),
            }
        }
    }

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
