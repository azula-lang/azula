use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::path::Path;
use std::process::Command;

use azula_codegen::prelude::Backend;
use azula_codegen::prelude::OptimizationLevel;
use azula_ir::prelude::{GlobalValue, Instruction, Module, Value};
use azula_type::prelude::AzulaType;
use inkwell::basic_block::BasicBlock;
use inkwell::module::{Linkage, Module as LLVMModule};
use inkwell::targets::{FileType, InitializationConfig, Target, TargetMachine, TargetTriple};
use inkwell::types::StructType;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue};
use inkwell::{builder::Builder, context::Context};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};

pub struct LLVMCodegen<'ctx> {
    context: &'ctx Context,
    module: LLVMModule<'ctx>,
    builder: Builder<'ctx>,

    strings: HashMap<usize, BasicValueEnum<'ctx>>,
    string_size: HashMap<usize, usize>,
    globals: HashMap<String, BasicValueEnum<'ctx>>,
    structs: HashMap<String, StructType<'ctx>>,

    target: Option<String>,
    opt_level: OptimizationLevel,
}

struct FunctionLocals<'a> {
    registers: HashMap<usize, BasicValueEnum<'a>>,
    variables: HashMap<String, BasicValueEnum<'a>>,

    blocks: HashMap<String, BasicBlock<'a>>,
}

impl<'ctx> Backend<'ctx> for LLVMCodegen<'ctx> {
    fn codegen(
        name: &'ctx str,
        destination: &'ctx str,
        emit: bool,
        target: Option<&String>,
        opt_level: OptimizationLevel,
        module: Module<'ctx>,
    ) -> Result<(), Box<dyn Error>> {
        let context = Context::create();
        let llvm_module = context.create_module(module.name);
        let target = if let Some(val) = target {
            Some(val.clone())
        } else {
            None
        };
        let mut codegen = LLVMCodegen {
            context: &context,
            module: llvm_module,
            builder: context.create_builder(),
            strings: HashMap::new(),
            string_size: HashMap::new(),
            globals: HashMap::new(),
            structs: HashMap::new(),
            target,
            opt_level,
        };

        codegen.generate_structs(&module);

        for (name, extern_func) in &module.extern_functions {
            let args: Vec<_> = extern_func
                .arguments
                .iter()
                .map(|arg| codegen.azula_type_to_llvm_basic_type(arg.clone()).into())
                .collect();
            codegen.module.add_function(
                name,
                codegen.azula_type_to_function_llvm_type_with_varargs(
                    extern_func.returns.clone(),
                    &args,
                    extern_func.varargs,
                ),
                Some(Linkage::External),
            );
        }

        codegen.module.add_function(
            "pow",
            codegen.context.f64_type().fn_type(
                &[
                    codegen.context.f64_type().as_basic_type_enum().into(),
                    codegen.context.f64_type().as_basic_type_enum().into(),
                ],
                false,
            ),
            Some(Linkage::External),
        );
        let mut i = 0;

        for (name, func) in &module.functions {
            let mut linkage = Some(Linkage::Private);
            // let mut linkage = None;
            if *name == "main" {
                linkage = None;
            }
            codegen.module.add_function(
                name,
                codegen.azula_type_to_function_llvm_type(
                    func.returns.clone(),
                    &func
                        .arguments
                        .iter()
                        .map(|(_, typ)| codegen.azula_type_to_llvm_basic_type(typ.clone()).into())
                        .collect::<Vec<_>>(),
                ),
                linkage,
            );
        }

        for (name, func) in &module.functions {
            let mut locals = FunctionLocals::new();
            let function = codegen.module.get_function(name).unwrap();

            for (name, block) in &func.blocks {
                let basic = if locals.blocks.contains_key(name) {
                    *locals.blocks.get(name).unwrap()
                } else {
                    codegen.context.append_basic_block(function, &name)
                };
                codegen.builder.position_at_end(basic);
                if i == 0 {
                    codegen.store_globals(&module);
                    i += 1;
                }
                for instruction in &block.instructions {
                    codegen.codegen_instruction(instruction.clone(), &function, &mut locals);
                }
            }
        }

        if emit {
            codegen
                .module
                .print_to_file(format!("{}.ll", name))
                .unwrap();
        }

        let object_file = format!(".build/{}.o", name);
        codegen.build_object_file(object_file.clone());

        if let Some(target) = codegen.target {
            Command::new("zig")
                .arg("cc")
                .arg(format!("-o{}{}", destination, name))
                .arg(object_file)
                .arg("-target")
                .arg(target)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        } else {
            Command::new("zig")
                .arg("cc")
                .arg(format!("-o{}{}", destination, name))
                .arg(object_file)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }

        Ok(())
    }
}

impl<'a> LLVMCodegen<'a> {
    fn store_globals(&mut self, module: &Module<'a>) {
        for (i, str) in module.strings.clone().into_iter().enumerate() {
            let ptr = self
                .builder
                .build_global_string_ptr(str.as_str(), "string")
                .as_basic_value_enum();

            self.strings.insert(i, ptr);
            self.string_size.insert(i, str.len());
        }

        for (name, val) in &module.global_values {
            let ptr = match val {
                GlobalValue::Int(i) => {
                    let val = self.module.add_global(
                        self.context.i64_type(),
                        Some(AddressSpace::Global),
                        &name,
                    );

                    val.set_initializer(
                        &self
                            .context
                            .i64_type()
                            .const_int((*i).try_into().unwrap(), false),
                    );

                    val.as_basic_value_enum()
                }
                GlobalValue::Float(f) => {
                    let val = self.module.add_global(
                        self.context.f64_type(),
                        Some(AddressSpace::Global),
                        &name,
                    );

                    val.set_initializer(
                        &self
                            .context
                            .f64_type()
                            .const_float((*f).try_into().unwrap()),
                    );

                    val.as_basic_value_enum()
                }
                GlobalValue::Bool(b) => {
                    let val = self.module.add_global(
                        self.context.f64_type(),
                        Some(AddressSpace::Global),
                        &name,
                    );

                    val.set_initializer(
                        &self
                            .context
                            .bool_type()
                            .const_int((*b).try_into().unwrap(), false),
                    );

                    val.as_basic_value_enum()
                }
                GlobalValue::String(s) => *self.strings.get(&s).unwrap(),
                GlobalValue::Array(_) => todo!(),
            };

            self.globals.insert(name.clone(), ptr);
        }
    }

    fn generate_structs(&mut self, module: &Module<'a>) {
        // Generate structs first so they can refer to each other
        for (i, _) in &module.structs {
            let struc = self.context.opaque_struct_type(i);
            self.structs.insert(i.to_string(), struc);
        }

        // Set the body of the structs
        for (i, str) in &module.structs {
            let args: Vec<_> = str
                .attributes
                .iter()
                .map(|(arg, _)| self.azula_type_to_llvm_basic_type(arg.clone()))
                .collect();

            self.structs
                .get(&i.to_string())
                .unwrap()
                .set_body(&args, false);
        }
    }

    fn codegen_instruction(
        &self,
        instruction: Instruction<'a>,
        func: &FunctionValue<'a>,
        locals: &mut FunctionLocals<'a>,
    ) {
        match instruction {
            Instruction::Load(name, dest, _) => {
                let alloca = locals.variables.get(&name).unwrap();
                let value = self.builder.build_load(alloca.into_pointer_value(), "load");

                locals.store(dest, value);
            }
            Instruction::LoadGlobal(name, dest, _) => {
                let alloca = self.globals.get(&name).unwrap();
                let value = self.builder.build_load(alloca.into_pointer_value(), "load");

                locals.store(dest, value);
            }
            Instruction::LoadArg(arg, dest, _) => {
                locals.store(dest, func.get_params()[arg]);
            }
            Instruction::Store(name, val, typ) => {
                let value = match val {
                    Value::Local(val) => locals.load(val),
                    Value::LiteralInteger(_) => todo!(),
                    Value::LiteralBoolean(_) => todo!(),
                    Value::Global(y) => {
                        // let alloca = self.builder.build_alloca(
                        //     azula_type_to_llvm_basic_type(self.context, typ),
                        //     "alloca",
                        // );
                        let alloca = if locals.variables.contains_key(&name) {
                            locals.variables.get(&name).unwrap().into_pointer_value()
                        } else {
                            let alloca = self
                                .builder
                                .build_alloca(self.azula_type_to_llvm_basic_type(typ), "alloca");
                            locals.variables.insert(name, alloca.as_basic_value_enum());

                            alloca
                        };
                        self.builder.build_store(
                            alloca,
                            self.strings.get(&y).unwrap().as_basic_value_enum(),
                        );
                        return;
                    }
                };

                let alloca = if locals.variables.contains_key(&name) {
                    locals.variables.get(&name).unwrap().into_pointer_value()
                } else {
                    let alloca = self
                        .builder
                        .build_alloca(self.azula_type_to_llvm_basic_type(typ), "alloca");
                    locals.variables.insert(name, alloca.as_basic_value_enum());

                    alloca
                };
                self.builder.build_store(alloca, value);
            }
            Instruction::ConstInt(val, dest) => {
                locals.registers.insert(
                    dest,
                    self.context
                        .i64_type()
                        .const_int(val as u64, false)
                        .as_basic_value_enum(),
                );
            }
            Instruction::ConstFloat(val, dest) => {
                locals.registers.insert(
                    dest,
                    self.context
                        .f64_type()
                        .const_float(val)
                        .as_basic_value_enum(),
                );
            }
            Instruction::ConstTrue(dest) => {
                locals.registers.insert(
                    dest,
                    self.context
                        .bool_type()
                        .const_int(1 as u64, false)
                        .as_basic_value_enum(),
                );
            }
            Instruction::ConstFalse(dest) => {
                locals.registers.insert(
                    dest,
                    self.context
                        .bool_type()
                        .const_int(0 as u64, false)
                        .as_basic_value_enum(),
                );
            }
            Instruction::ConstNull(dest) => {
                locals.registers.insert(
                    dest,
                    self.context
                        .i8_type()
                        .ptr_type(AddressSpace::Generic)
                        .const_zero()
                        .as_basic_value_enum(),
                );
            }
            Instruction::Add(..) => self.codegen_add(instruction, locals),
            Instruction::Sub(..) => self.codegen_sub(instruction, locals),
            Instruction::Mul(..) => self.codegen_mul(instruction, locals),
            Instruction::Div(..) => self.codegen_div(instruction, locals),
            Instruction::Mod(..) => self.codegen_mod(instruction, locals),
            Instruction::Pow(..) => self.codegen_pow(instruction, locals),
            Instruction::Return(val) => match val {
                None => {
                    self.builder.build_return(None);
                }
                Some(Value::Local(val)) => {
                    self.builder
                        .build_return(Some(locals.registers.get(&val).clone().unwrap()));
                }
                Some(Value::Global(val)) => {
                    self.builder
                        .build_return(Some(self.strings.get(&val).clone().unwrap()));
                }
                _ => unreachable!(),
            },
            Instruction::FunctionCall(name, args, dest) => {
                let converted_args: Vec<BasicMetadataValueEnum> = args
                    .iter()
                    .map(|arg| match arg {
                        Value::Global(i) => {
                            self.strings.get(i).unwrap().as_basic_value_enum().into()
                        }
                        Value::Local(..) => locals.load(value_to_local(arg.clone())).into(),
                        Value::LiteralInteger(_) => todo!(),
                        Value::LiteralBoolean(_) => todo!(),
                    })
                    .collect();

                let result = self.builder.build_call(
                    self.module.get_function(&name).unwrap(),
                    &converted_args,
                    "call",
                );

                let result = result.try_as_basic_value();

                if result.is_left() {
                    locals.store(dest, result.unwrap_left().as_basic_value_enum());
                }
            }
            Instruction::Jcond(cond, true_block_name, end_block_name) => {
                let local = locals.load(value_to_local(cond)).into_int_value();

                let true_block = self
                    .context
                    .append_basic_block(*func, true_block_name.clone().as_str());
                let end_block = self
                    .context
                    .append_basic_block(*func, end_block_name.clone().as_str());

                locals.blocks.insert(true_block_name.clone(), true_block);
                locals.blocks.insert(end_block_name.clone(), end_block);

                self.builder
                    .build_conditional_branch(local, true_block, end_block);

                self.builder.position_at_end(true_block);
                self.builder.position_at_end(end_block);
            }
            Instruction::Or(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1)).into_int_value();
                let local2 = locals.load(value_to_local(val2)).into_int_value();
                let value = self.builder.build_or(local1, local2, "or");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::And(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1)).into_int_value();
                let local2 = locals.load(value_to_local(val2)).into_int_value();
                let value = self.builder.build_and(local1, local2, "and");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Eq(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1));
                let local2 = locals.load(value_to_local(val2));

                let value = match local1.get_type() {
                    BasicTypeEnum::FloatType(_) => self
                        .builder
                        .build_float_compare(
                            FloatPredicate::OEQ,
                            local1.into_float_value(),
                            local2.into_float_value(),
                            "add",
                        )
                        .as_basic_value_enum(),
                    BasicTypeEnum::IntType(_) => self
                        .builder
                        .build_int_compare(
                            IntPredicate::EQ,
                            local1.into_int_value(),
                            local2.into_int_value(),
                            "add",
                        )
                        .as_basic_value_enum(),
                    _ => unreachable!(),
                };

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Neq(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1));
                let local2 = locals.load(value_to_local(val2));

                let value = match local1.get_type() {
                    BasicTypeEnum::FloatType(_) => self
                        .builder
                        .build_float_compare(
                            FloatPredicate::ONE,
                            local1.into_float_value(),
                            local2.into_float_value(),
                            "sub",
                        )
                        .as_basic_value_enum(),
                    BasicTypeEnum::IntType(_) => self
                        .builder
                        .build_int_compare(
                            IntPredicate::NE,
                            local1.into_int_value(),
                            local2.into_int_value(),
                            "sub",
                        )
                        .as_basic_value_enum(),
                    _ => unreachable!(),
                };

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Jump(val) => {
                if let Some(block) = locals.blocks.get(&val) {
                    self.builder.build_unconditional_branch(*block);
                } else {
                    let jump_block = self.context.append_basic_block(*func, val.clone().as_str());
                    locals.blocks.insert(val.clone(), jump_block);
                    self.builder.build_unconditional_branch(jump_block);

                    self.builder.position_at_end(jump_block);
                }
            }
            Instruction::Gt(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1));
                let local2 = locals.load(value_to_local(val2));

                let value = match local1.get_type() {
                    BasicTypeEnum::FloatType(_) => self
                        .builder
                        .build_float_compare(
                            FloatPredicate::OGT,
                            local1.into_float_value(),
                            local2.into_float_value(),
                            "gt",
                        )
                        .as_basic_value_enum(),
                    BasicTypeEnum::IntType(_) => self
                        .builder
                        .build_int_compare(
                            IntPredicate::SGT,
                            local1.into_int_value(),
                            local2.into_int_value(),
                            "gt",
                        )
                        .as_basic_value_enum(),
                    _ => unreachable!(),
                };

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Gte(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1));
                let local2 = locals.load(value_to_local(val2));

                let value = match local1.get_type() {
                    BasicTypeEnum::FloatType(_) => self
                        .builder
                        .build_float_compare(
                            FloatPredicate::OGE,
                            local1.into_float_value(),
                            local2.into_float_value(),
                            "gte",
                        )
                        .as_basic_value_enum(),
                    BasicTypeEnum::IntType(_) => self
                        .builder
                        .build_int_compare(
                            IntPredicate::SGE,
                            local1.into_int_value(),
                            local2.into_int_value(),
                            "gte",
                        )
                        .as_basic_value_enum(),
                    _ => unreachable!(),
                };

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Lt(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1));
                let local2 = locals.load(value_to_local(val2));

                let value = match local1.get_type() {
                    BasicTypeEnum::FloatType(_) => self
                        .builder
                        .build_float_compare(
                            FloatPredicate::OLT,
                            local1.into_float_value(),
                            local2.into_float_value(),
                            "lt",
                        )
                        .as_basic_value_enum(),
                    BasicTypeEnum::IntType(_) => self
                        .builder
                        .build_int_compare(
                            IntPredicate::SLT,
                            local1.into_int_value(),
                            local2.into_int_value(),
                            "lt",
                        )
                        .as_basic_value_enum(),
                    _ => unreachable!("{:?}", local1),
                };

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Lte(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1));
                let local2 = locals.load(value_to_local(val2));

                let value = match local1.get_type() {
                    BasicTypeEnum::FloatType(_) => self
                        .builder
                        .build_float_compare(
                            FloatPredicate::OLE,
                            local1.into_float_value(),
                            local2.into_float_value(),
                            "add",
                        )
                        .as_basic_value_enum(),
                    BasicTypeEnum::IntType(_) => self
                        .builder
                        .build_int_compare(
                            IntPredicate::SLE,
                            local1.into_int_value(),
                            local2.into_int_value(),
                            "add",
                        )
                        .as_basic_value_enum(),
                    _ => unreachable!(),
                };

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Not(val, dest) => {
                let local = locals.load(value_to_local(val)).into_int_value();
                let value = self.builder.build_not(local, "not");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Pointer(val, dest) => {
                let alloca = locals.variables.get(&val).unwrap().clone();

                locals.store(dest, alloca.as_basic_value_enum());
            }
            Instruction::CreateArray(typ, size, dest) => {
                // let alloca = self.builder.build_alloca(
                //     self.azula_type_to_llvm_basic_type(typ)
                //         .array_type(size as u32),
                //     "array",
                // );

                let array = self
                    .builder
                    .build_array_malloc(
                        self.azula_type_to_llvm_basic_type(typ.clone()),
                        self.context.i32_type().const_int(size as u64, false),
                        "array",
                    )
                    .unwrap();

                locals.store(dest, array.as_basic_value_enum());
            }
            Instruction::StoreElement(array, index, value) => {
                let array = locals.load(value_to_local(array));

                let index = locals.load(value_to_local(index)).into_int_value();

                let val = match value {
                    Value::Local(..) => locals.load(value_to_local(value)),
                    Value::Global(pos) => self.strings.get(&pos).unwrap().as_basic_value_enum(),
                    _ => unreachable!(),
                };

                // let array_ptr = self.builder.build_bitcast(
                //     array,
                //     array
                //         .get_type()
                //         .into_array_type()
                //         .ptr_type(AddressSpace::Generic),
                //     "cast",
                // );

                let ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(array.into_pointer_value(), &[index], "gep")
                };

                self.builder.build_store(ptr, val);
            }
            Instruction::AccessElement(array, index, dest) => {
                let array = locals.load(value_to_local(array));

                let index = locals.load(value_to_local(index)).into_int_value();

                // let array_ptr = self.builder.build_bitcast(
                //     array,
                //     array
                //         .get_type()
                //         .into_array_type()
                //         .ptr_type(AddressSpace::Generic),
                //     "cast",
                // );

                let ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(array.into_pointer_value(), &[index], "gep")
                };

                let result = self.builder.build_load(ptr, "access");

                locals.store(dest, result.as_basic_value_enum());
            }
            Instruction::StoreStructMember(struc, index, val) => {
                let val = match val {
                    Value::Local(ptr) => locals.load(ptr),
                    Value::LiteralInteger(_) => todo!(),
                    Value::LiteralBoolean(_) => todo!(),
                    Value::Global(v) => *self.strings.get(&v).unwrap(),
                };

                let struc_val = locals.load(value_to_local(struc.clone()));

                if struc_val.is_struct_value() {
                    let val = self
                        .builder
                        .build_insert_value(struc_val.into_struct_value(), val, index as u32, "val")
                        .unwrap();

                    locals.store(value_to_local(struc), val.as_basic_value_enum());
                } else if struc_val.is_pointer_value() {
                    let ptr = unsafe {
                        self.builder.build_in_bounds_gep(
                            struc_val.into_pointer_value(),
                            &[
                                self.context.i32_type().const_int(0, false),
                                self.context.i32_type().const_int(index as u64, false),
                            ],
                            "gep",
                        )
                    };

                    self.builder.build_store(ptr, val);
                }
            }
            Instruction::CreateStruct(struc, values, dest) => {
                let struc = self.structs.get(&struc).unwrap();

                let vals: Vec<_> = values
                    .iter()
                    .map(|val| match val {
                        Value::Local(ptr) => locals.load(*ptr),
                        Value::LiteralInteger(_) => todo!(),
                        Value::LiteralBoolean(_) => todo!(),
                        Value::Global(v) => *self.strings.get(&v).unwrap(),
                    })
                    .collect();

                // self.builder.build_insert_value(agg, value, index, name)

                let val = struc.const_named_struct(&[]);

                let mut agg = val.as_basic_value_enum().into_struct_value();
                for (index, arg) in vals.iter().enumerate() {
                    agg = self
                        .builder
                        .build_insert_value(agg, arg.as_basic_value_enum(), index as u32, "insert")
                        .unwrap()
                        .as_basic_value_enum()
                        .into_struct_value();
                }

                locals.store(dest, agg.as_basic_value_enum());
            }
            Instruction::AccessStructMember(struc, index, dest, resolve) => {
                let struc = locals.load(value_to_local(struc));

                if struc.is_struct_value() {
                    let val = self
                        .builder
                        .build_extract_value(struc.into_struct_value(), index as u32, "val")
                        .unwrap();

                    locals.store(dest, val.as_basic_value_enum());
                } else if struc.is_pointer_value() {
                    let ptr = unsafe {
                        self.builder.build_in_bounds_gep(
                            struc.into_pointer_value(),
                            &[
                                self.context.i32_type().const_int(0, false),
                                self.context.i32_type().const_int(index as u64, false),
                            ],
                            "gep",
                        )
                    };

                    if resolve {
                        let val = self.builder.build_load(ptr, "load");
                        locals.store(dest, val.as_basic_value_enum());
                    } else {
                        locals.store(dest, ptr.as_basic_value_enum());
                    }
                }
            }
        };
    }

    fn codegen_add(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Add(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1));
            let local2 = locals.load(value_to_local(val2));

            let value = match local1.get_type() {
                BasicTypeEnum::FloatType(_) => self
                    .builder
                    .build_float_add(local1.into_float_value(), local2.into_float_value(), "add")
                    .as_basic_value_enum(),
                BasicTypeEnum::IntType(_) => self
                    .builder
                    .build_int_add(local1.into_int_value(), local2.into_int_value(), "add")
                    .as_basic_value_enum(),
                _ => unreachable!(),
            };

            locals.store(dest, value);
        }
    }

    fn codegen_sub(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Sub(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1));
            let local2 = locals.load(value_to_local(val2));

            let value = match local1.get_type() {
                BasicTypeEnum::FloatType(_) => self
                    .builder
                    .build_float_sub(local1.into_float_value(), local2.into_float_value(), "add")
                    .as_basic_value_enum(),
                BasicTypeEnum::IntType(_) => self
                    .builder
                    .build_int_sub(local1.into_int_value(), local2.into_int_value(), "add")
                    .as_basic_value_enum(),
                _ => unreachable!(),
            };

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn codegen_mul(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Mul(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1));
            let local2 = locals.load(value_to_local(val2));

            let value = match local1.get_type() {
                BasicTypeEnum::FloatType(_) => self
                    .builder
                    .build_float_mul(local1.into_float_value(), local2.into_float_value(), "add")
                    .as_basic_value_enum(),
                BasicTypeEnum::IntType(_) => self
                    .builder
                    .build_int_mul(local1.into_int_value(), local2.into_int_value(), "add")
                    .as_basic_value_enum(),
                _ => unreachable!(),
            };

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn codegen_div(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Div(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1));
            let local2 = locals.load(value_to_local(val2));

            let value = match local1.get_type() {
                BasicTypeEnum::FloatType(_) => self
                    .builder
                    .build_float_div(local1.into_float_value(), local2.into_float_value(), "div")
                    .as_basic_value_enum(),
                BasicTypeEnum::IntType(_) => self
                    .builder
                    .build_int_signed_div(local1.into_int_value(), local2.into_int_value(), "div")
                    .as_basic_value_enum(),
                _ => unreachable!(),
            };

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn codegen_mod(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Mod(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1));
            let local2 = locals.load(value_to_local(val2));

            let value = match local1.get_type() {
                BasicTypeEnum::FloatType(_) => self
                    .builder
                    .build_float_rem(local1.into_float_value(), local2.into_float_value(), "mod")
                    .as_basic_value_enum(),
                BasicTypeEnum::IntType(_) => self
                    .builder
                    .build_int_signed_rem(local1.into_int_value(), local2.into_int_value(), "mod")
                    .as_basic_value_enum(),
                _ => unreachable!(),
            };

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn codegen_pow(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Pow(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1));
            let local2 = locals.load(value_to_local(val2));

            let result = self.builder.build_call(
                self.module.get_function("pow").unwrap(),
                &[local1.into(), local2.into()],
                "power",
            );

            locals.store(dest, result.try_as_basic_value().unwrap_left());
        }
    }

    fn build_object_file(&self, dest: String) {
        let target_machine = self.create_machine(self.target.clone()).unwrap();

        target_machine
            .write_to_file(&self.module, FileType::Object, Path::new(&dest))
            .unwrap();
    }

    fn create_machine(&self, name: Option<String>) -> Option<TargetMachine> {
        if let Some(target) = name {
            let triple = TargetTriple::create(&target);

            self.module.set_triple(&triple);
            Target::initialize_all(&InitializationConfig::default());
            let target = Target::from_triple(&triple).unwrap();
            let mut opt_level = inkwell::OptimizationLevel::Default;
            if self.opt_level == OptimizationLevel::Aggressive {
                opt_level = inkwell::OptimizationLevel::Aggressive;
            }
            return target.create_target_machine(
                &triple,
                "",
                "",
                opt_level,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            );
        }

        let triple = TargetMachine::get_default_triple();
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        self.module.set_triple(&triple);
        Target::initialize_native(&InitializationConfig::default()).unwrap();
        let target = Target::from_triple(&triple).unwrap();
        let mut opt_level = inkwell::OptimizationLevel::Default;
        if self.opt_level == OptimizationLevel::Aggressive {
            opt_level = inkwell::OptimizationLevel::Aggressive;
        }
        target.create_target_machine(
            &triple,
            &cpu,
            &features,
            opt_level,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
    }

    fn azula_type_to_llvm_basic_type(&self, t: AzulaType<'a>) -> BasicTypeEnum<'a> {
        match t {
            AzulaType::Int => self.context.i64_type().as_basic_type_enum(),
            AzulaType::SizedSignedInt(size) => match size {
                8 => self.context.i8_type().as_basic_type_enum(),
                16 => self.context.i16_type().as_basic_type_enum(),
                32 => self.context.i32_type().as_basic_type_enum(),
                64 => self.context.i64_type().as_basic_type_enum(),
                _ => unreachable!(),
            },
            AzulaType::SizedUnsignedInt(size) => match size {
                8 => self.context.i8_type().as_basic_type_enum(),
                16 => self.context.i16_type().as_basic_type_enum(),
                32 => self.context.i32_type().as_basic_type_enum(),
                64 => self.context.i64_type().as_basic_type_enum(),
                _ => unreachable!(),
            },
            AzulaType::Str => self.context.i8_type().as_basic_type_enum(),
            AzulaType::Float => self.context.f64_type().as_basic_type_enum(),
            AzulaType::SizedFloat(size) => match size {
                16 => self.context.f16_type().as_basic_type_enum(),
                32 => self.context.f32_type().as_basic_type_enum(),
                64 => self.context.f64_type().as_basic_type_enum(),
                _ => unreachable!(),
            },
            AzulaType::Bool => self.context.bool_type().as_basic_type_enum(),
            AzulaType::Void => todo!(),
            AzulaType::Pointer(nested) => {
                let typ = self.azula_type_to_llvm_basic_type(nested.deref().clone());

                typ.ptr_type(AddressSpace::Generic).as_basic_type_enum()
            }
            AzulaType::Infer => unreachable!(),
            AzulaType::Named(name) => self
                .structs
                .get(&name.to_string())
                .unwrap()
                .as_basic_type_enum(),
            AzulaType::UnknownType(_) => todo!(),
            AzulaType::Array(typ, _) => {
                let typ = self.azula_type_to_llvm_basic_type(typ.deref().clone());

                // typ.array_type(size.unwrap() as u32).as_basic_type_enum()
                typ.ptr_type(AddressSpace::Generic).as_basic_type_enum()
            }
        }
    }

    fn azula_type_to_function_llvm_type(
        &self,
        t: AzulaType<'a>,
        args: &[BasicMetadataTypeEnum<'a>],
    ) -> FunctionType<'a> {
        match t {
            AzulaType::Int => self.context.i64_type().fn_type(args, false),
            AzulaType::SizedSignedInt(size) => match size {
                8 => self.context.i8_type().as_basic_type_enum(),
                16 => self.context.i16_type().as_basic_type_enum(),
                32 => self.context.i32_type().as_basic_type_enum(),
                64 => self.context.i64_type().as_basic_type_enum(),
                _ => unreachable!(),
            }
            .fn_type(args, false),
            AzulaType::SizedUnsignedInt(size) => match size {
                8 => self.context.i8_type().as_basic_type_enum(),
                16 => self.context.i16_type().as_basic_type_enum(),
                32 => self.context.i32_type().as_basic_type_enum(),
                64 => self.context.i64_type().as_basic_type_enum(),
                _ => unreachable!(),
            }
            .fn_type(args, false),
            AzulaType::Str => todo!(),
            AzulaType::Float => self.context.f64_type().fn_type(args, false),
            AzulaType::SizedFloat(size) => match size {
                16 => self.context.f16_type().as_basic_type_enum(),
                32 => self.context.f32_type().as_basic_type_enum(),
                64 => self.context.f64_type().as_basic_type_enum(),
                _ => unreachable!(),
            }
            .fn_type(args, false),
            AzulaType::Bool => self.context.bool_type().fn_type(args, false),
            AzulaType::Void => self.context.void_type().fn_type(args, false),
            AzulaType::Pointer(nested) => {
                let typ = self.azula_type_to_llvm_basic_type(nested.deref().clone());

                typ.ptr_type(AddressSpace::Generic).fn_type(args, false)
            }
            AzulaType::Infer => todo!(),
            AzulaType::Named(name) => self
                .structs
                .get(&name.to_string())
                .unwrap()
                .fn_type(args, false),
            AzulaType::UnknownType(_) => todo!(),
            AzulaType::Array(typ, _) => {
                let typ = self.azula_type_to_llvm_basic_type(typ.deref().clone());

                // typ.array_type(size.unwrap() as u32).fn_type(args, false)
                typ.ptr_type(AddressSpace::Generic).fn_type(args, false)
            }
        }
    }

    fn azula_type_to_function_llvm_type_with_varargs(
        &self,
        t: AzulaType<'a>,
        args: &[BasicMetadataTypeEnum<'a>],
        varargs: bool,
    ) -> FunctionType<'a> {
        match t {
            AzulaType::Int => self.context.i64_type().fn_type(args, varargs),
            AzulaType::SizedSignedInt(size) => match size {
                8 => self.context.i8_type().as_basic_type_enum(),
                16 => self.context.i16_type().as_basic_type_enum(),
                32 => self.context.i32_type().as_basic_type_enum(),
                64 => self.context.i64_type().as_basic_type_enum(),
                _ => unreachable!(),
            }
            .fn_type(args, true),
            AzulaType::SizedUnsignedInt(size) => match size {
                8 => self.context.i8_type().as_basic_type_enum(),
                16 => self.context.i16_type().as_basic_type_enum(),
                32 => self.context.i32_type().as_basic_type_enum(),
                64 => self.context.i64_type().as_basic_type_enum(),
                _ => unreachable!(),
            }
            .fn_type(args, true),
            AzulaType::Str => todo!(),
            AzulaType::Float => self.context.f64_type().fn_type(args, varargs),
            AzulaType::SizedFloat(size) => match size {
                16 => self.context.f16_type().as_basic_type_enum(),
                32 => self.context.f32_type().as_basic_type_enum(),
                64 => self.context.f64_type().as_basic_type_enum(),
                _ => unreachable!(),
            }
            .fn_type(args, false),
            AzulaType::Bool => self.context.bool_type().fn_type(args, varargs),
            AzulaType::Void => self.context.void_type().fn_type(args, varargs),
            AzulaType::Pointer(nested) => {
                let typ = self.azula_type_to_llvm_basic_type(nested.deref().clone());

                typ.ptr_type(AddressSpace::Generic).fn_type(args, varargs)
            }
            AzulaType::Infer => todo!(),
            AzulaType::Named(name) => self
                .structs
                .get(&name.to_string())
                .unwrap()
                .fn_type(args, varargs),
            AzulaType::UnknownType(_) => todo!(),
            AzulaType::Array(typ, _) => {
                let typ = self.azula_type_to_llvm_basic_type(typ.deref().clone());

                // typ.array_type(size.unwrap() as u32).fn_type(args, false)
                typ.ptr_type(AddressSpace::Generic).fn_type(args, varargs)
            }
        }
    }
}

impl<'ctx> FunctionLocals<'ctx> {
    pub fn new() -> Self {
        Self {
            registers: HashMap::new(),
            variables: HashMap::new(),
            blocks: HashMap::new(),
        }
    }

    pub fn store(&mut self, dest: usize, value: BasicValueEnum<'ctx>) {
        self.registers.insert(dest, value);
    }

    pub fn load(&self, dest: usize) -> BasicValueEnum<'ctx> {
        self.registers.get(&dest).unwrap().clone()
    }
}

fn value_to_local(value: Value) -> usize {
    match value {
        Value::LiteralInteger(_) => unreachable!(),
        Value::LiteralBoolean(_) => unreachable!(),
        Value::Local(val) => val,
        Value::Global(_) => unreachable!(),
    }
}
