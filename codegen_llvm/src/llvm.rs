use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::path::Path;
use std::process::Command;

use azula_codegen::prelude::Backend;
use azula_ir::prelude::{GlobalValue, Instruction, Module, Value};
use azula_type::prelude::AzulaType;
use inkwell::basic_block::BasicBlock;
use inkwell::module::{Linkage, Module as LLVMModule};
use inkwell::targets::{FileType, InitializationConfig, Target, TargetMachine, TargetTriple};
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

    target: Option<String>,
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
            target,
        };

        for (name, extern_func) in &module.extern_functions {
            let args: Vec<_> = extern_func
                .arguments
                .iter()
                .map(|arg| azula_type_to_llvm_basic_type(&context, arg.clone()).into())
                .collect();
            codegen.module.add_function(
                name,
                azula_type_to_function_llvm_type_with_varargs(
                    &context,
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
            // let mut linkage = Some(Linkage::Private);
            let mut linkage = None;
            if *name == "main" {
                linkage = None;
            }
            codegen.module.add_function(
                name,
                azula_type_to_function_llvm_type(
                    &codegen.context,
                    func.returns.clone(),
                    &func
                        .arguments
                        .iter()
                        .map(|(_, typ)| {
                            azula_type_to_llvm_basic_type(&codegen.context, typ.clone()).into()
                        })
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
                .print_to_file(format!(".build/{}.o", name))
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
            };

            self.globals.insert(name.clone(), ptr);
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
                        let alloca = self.builder.build_alloca(
                            azula_type_to_llvm_basic_type(self.context, typ),
                            "alloca",
                        );
                        self.builder.build_store(
                            alloca,
                            self.strings.get(&y).unwrap().as_basic_value_enum(),
                        );
                        locals.variables.insert(name, alloca.as_basic_value_enum());
                        return;
                    }
                };
                let alloca = self
                    .builder
                    .build_alloca(azula_type_to_llvm_basic_type(self.context, typ), "alloca");
                self.builder.build_store(alloca, value);

                locals.variables.insert(name, alloca.as_basic_value_enum());
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
                let jump_block = self.context.append_basic_block(*func, val.clone().as_str());
                locals.blocks.insert(val.clone(), jump_block);
                self.builder.build_unconditional_branch(jump_block);

                self.builder.position_at_end(jump_block);
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
                    _ => unreachable!(),
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
            return target.create_target_machine(
                &triple,
                "",
                "",
                inkwell::OptimizationLevel::Default,
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
        target.create_target_machine(
            &triple,
            &cpu,
            &features,
            inkwell::OptimizationLevel::Default,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
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

fn azula_type_to_llvm_basic_type<'ctx>(
    context: &'ctx Context,
    t: AzulaType,
) -> BasicTypeEnum<'ctx> {
    match t {
        AzulaType::Int => context.i64_type().as_basic_type_enum(),
        AzulaType::Str => context.i8_type().as_basic_type_enum(),
        AzulaType::Float => context.f64_type().as_basic_type_enum(),
        AzulaType::Bool => context.bool_type().as_basic_type_enum(),
        AzulaType::Void => todo!(),
        AzulaType::Pointer(nested) => {
            let typ = azula_type_to_llvm_basic_type(context, nested.deref().clone());

            typ.ptr_type(AddressSpace::Generic).as_basic_type_enum()
        }
        AzulaType::Infer => unreachable!(),
        AzulaType::Named(_) => todo!(),
        AzulaType::UnknownType(_) => todo!(),
    }
}

fn azula_type_to_function_llvm_type<'ctx>(
    context: &'ctx Context,
    t: AzulaType,
    args: &[BasicMetadataTypeEnum<'ctx>],
) -> FunctionType<'ctx> {
    match t {
        AzulaType::Int => context.i64_type().fn_type(args, false),
        AzulaType::Str => todo!(),
        AzulaType::Float => context.f64_type().fn_type(args, false),
        AzulaType::Bool => context.bool_type().fn_type(args, false),
        AzulaType::Void => context.void_type().fn_type(args, false),
        AzulaType::Pointer(nested) => {
            let typ = azula_type_to_llvm_basic_type(context, nested.deref().clone());

            typ.ptr_type(AddressSpace::Generic).fn_type(args, false)
        }
        AzulaType::Infer => todo!(),
        AzulaType::Named(_) => todo!(),
        AzulaType::UnknownType(_) => todo!(),
    }
}

fn azula_type_to_function_llvm_type_with_varargs<'ctx>(
    context: &'ctx Context,
    t: AzulaType,
    args: &[BasicMetadataTypeEnum<'ctx>],
    varargs: bool,
) -> FunctionType<'ctx> {
    match t {
        AzulaType::Int => context.i64_type().fn_type(args, varargs),
        AzulaType::Str => todo!(),
        AzulaType::Float => context.f64_type().fn_type(args, varargs),
        AzulaType::Bool => context.bool_type().fn_type(args, varargs),
        AzulaType::Void => context.void_type().fn_type(args, varargs),
        AzulaType::Pointer(nested) => {
            let typ = azula_type_to_llvm_basic_type(context, nested.deref().clone());

            typ.ptr_type(AddressSpace::Generic).fn_type(args, varargs)
        }
        AzulaType::Infer => todo!(),
        AzulaType::Named(_) => todo!(),
        AzulaType::UnknownType(_) => todo!(),
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
