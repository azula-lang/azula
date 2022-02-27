use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::path::Path;
use std::process::Command;

use azula_codegen::prelude::Backend;
use azula_ir::prelude::{Instruction, Module, Value};
use azula_type::prelude::AzulaType;
use inkwell::basic_block::BasicBlock;
use inkwell::module::{Linkage, Module as LLVMModule};
use inkwell::targets::{FileType, InitializationConfig, Target, TargetMachine};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue};
use inkwell::{builder::Builder, context::Context};
use inkwell::{AddressSpace, IntPredicate};

pub struct LLVMCodegen<'ctx> {
    context: &'ctx Context,
    module: LLVMModule<'ctx>,
    builder: Builder<'ctx>,

    strings: HashMap<usize, BasicValueEnum<'ctx>>,
}

struct FunctionLocals<'a> {
    registers: HashMap<usize, BasicValueEnum<'a>>,
    variables: HashMap<String, BasicValueEnum<'a>>,

    blocks: HashMap<String, BasicBlock<'a>>,
}

impl<'ctx> Backend<'ctx> for LLVMCodegen<'ctx> {
    fn codegen(module: Module<'ctx>) -> Result<(), Box<dyn Error>> {
        let context = Context::create();
        let llvm_module = context.create_module(module.name);
        let mut codegen = LLVMCodegen {
            context: &context,
            module: llvm_module,
            builder: context.create_builder(),
            strings: HashMap::new(),
        };

        codegen.module.add_function(
            "printf",
            codegen.context.i32_type().fn_type(
                &[codegen
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum()
                    .into()],
                true,
            ),
            Some(Linkage::External),
        );

        let mut i = 0;

        for (name, func) in &module.functions {
            let mut linkage = Some(Linkage::Private);
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
                    for (i, str) in module.strings.clone().into_iter().enumerate() {
                        let ptr = codegen
                            .builder
                            .build_global_string_ptr(str.as_str(), "string")
                            .as_basic_value_enum();

                        codegen.strings.insert(i, ptr);
                    }
                    i += 1;
                }
                for instruction in &block.instructions {
                    codegen.codegen_instruction(instruction.clone(), &function, &mut locals);
                }
            }
        }

        codegen.module.print_to_file("test.ll").unwrap();

        codegen.build_object_file(".build/out.o");

        Command::new("clang")
            .arg("-otest")
            .arg(".build/out.o")
            .arg("-flto")
            // -lSystem seems to have gone missing?
            .arg("-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib")
            .arg("-march=native")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Ok(())
    }
}

impl<'a> LLVMCodegen<'a> {
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
            Instruction::LoadArg(arg, dest, _) => {
                locals.store(dest, func.get_params()[arg]);
            }
            Instruction::Store(name, val, typ) => {
                let value = locals.load(value_to_local(val));
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
            Instruction::Add(..) => self.codegen_add(instruction, locals),
            Instruction::Sub(..) => self.codegen_sub(instruction, locals),
            Instruction::Mul(..) => self.codegen_mul(instruction, locals),
            Instruction::Div(..) => self.codegen_div(instruction, locals),
            Instruction::Mod(..) => self.codegen_mod(instruction, locals),
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
                let local1 = locals.load(value_to_local(val1)).into_int_value();
                let local2 = locals.load(value_to_local(val2)).into_int_value();
                let value = self
                    .builder
                    .build_int_compare(IntPredicate::EQ, local1, local2, "eq");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Neq(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1)).into_int_value();
                let local2 = locals.load(value_to_local(val2)).into_int_value();
                let value = self
                    .builder
                    .build_int_compare(IntPredicate::NE, local1, local2, "eq");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Jump(val) => {
                let jump_block = self.context.append_basic_block(*func, val.clone().as_str());
                locals.blocks.insert(val.clone(), jump_block);
                self.builder.build_unconditional_branch(jump_block);

                self.builder.position_at_end(jump_block);
            }
            Instruction::Gt(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1)).into_int_value();
                let local2 = locals.load(value_to_local(val2)).into_int_value();
                let value = self
                    .builder
                    .build_int_compare(IntPredicate::SGT, local1, local2, "gt");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Gte(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1)).into_int_value();
                let local2 = locals.load(value_to_local(val2)).into_int_value();
                let value =
                    self.builder
                        .build_int_compare(IntPredicate::SGE, local1, local2, "gte");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Lt(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1)).into_int_value();
                let local2 = locals.load(value_to_local(val2)).into_int_value();
                let value = self
                    .builder
                    .build_int_compare(IntPredicate::SLT, local1, local2, "lt");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Lte(val1, val2, dest) => {
                let local1 = locals.load(value_to_local(val1)).into_int_value();
                let local2 = locals.load(value_to_local(val2)).into_int_value();
                let value = self
                    .builder
                    .build_int_compare(IntPredicate::SLE, local1, local2, "lt");

                locals.store(dest, value.as_basic_value_enum());
            }
            Instruction::Not(val, dest) => {
                let local = locals.load(value_to_local(val)).into_int_value();
                let value = self.builder.build_not(local, "not");

                locals.store(dest, value.as_basic_value_enum());
            }
        };
    }

    fn codegen_add(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Add(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1)).into_int_value();
            let local2 = locals.load(value_to_local(val2)).into_int_value();

            let value = self.builder.build_int_add(local1, local2, "add");

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn codegen_sub(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Sub(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1)).into_int_value();
            let local2 = locals.load(value_to_local(val2)).into_int_value();

            let value = self.builder.build_int_sub(local1, local2, "sub");

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn codegen_mul(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Mul(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1)).into_int_value();
            let local2 = locals.load(value_to_local(val2)).into_int_value();

            let value = self.builder.build_int_mul(local1, local2, "mul");

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn codegen_div(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Div(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1)).into_int_value();
            let local2 = locals.load(value_to_local(val2)).into_int_value();

            let value = self.builder.build_int_signed_div(local1, local2, "div");

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn codegen_mod(&self, instruction: Instruction<'a>, locals: &mut FunctionLocals<'a>) {
        if let Instruction::Mod(val1, val2, dest) = instruction {
            let local1 = locals.load(value_to_local(val1)).into_int_value();
            let local2 = locals.load(value_to_local(val2)).into_int_value();

            let value = self.builder.build_int_signed_rem(local1, local2, "mod");

            locals.store(dest, value.as_basic_value_enum());
        }
    }

    fn build_object_file(&self, dest: &'a str) {
        let triple = TargetMachine::get_default_triple();
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        self.module.set_triple(&triple);
        Target::initialize_native(&InitializationConfig::default()).unwrap();
        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &triple,
                &cpu,
                &features,
                inkwell::OptimizationLevel::Default,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .unwrap();

        target_machine
            .write_to_file(&self.module, FileType::Object, Path::new(dest))
            .unwrap();
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

fn value_to_local(value: Value) -> usize {
    match value {
        Value::LiteralInteger(_) => unreachable!(),
        Value::LiteralBoolean(_) => unreachable!(),
        Value::Local(val) => val,
        Value::Global(_) => unreachable!(),
    }
}
