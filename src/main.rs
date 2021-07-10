use core::panic;
use std::{
    collections::HashMap,
    fs::{self},
    path::Path,
    process::Command,
};

pub mod codegen;
pub mod parser;

use inkwell::{
    context::Context,
    module::Linkage,
    targets::{FileType, InitializationConfig, Target, TargetTriple},
    types::{AnyType, AnyTypeEnum, BasicType},
    values::BasicValue,
    AddressSpace,
};
use parser::ast::Statement;

use crate::codegen::*;
use crate::parser::ast::Type;

#[macro_use]
extern crate lalrpop_util;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source_file =
        fs::read_to_string(Path::new(&args[1])).expect("Could not read supplied file.");

    let parse_tree = parser::parser::ProgramParser::new()
        .parse(&source_file)
        .unwrap();

    let context = Context::create();

    let module = context.create_module("main_mod");

    let builder = context.create_builder();
    let mut compiler = Compiler {
        context: &context,
        builder: &builder,
        module,
        ptrs: HashMap::new(),
    };

    let print_int = compiler.module.add_function(
        "print_int",
        context
            .void_type()
            .fn_type(&[context.i32_type().as_basic_type_enum()], false),
        Some(Linkage::Private),
    );
    let str_type = context.i8_type().ptr_type(AddressSpace::Generic);
    let printf = compiler.module.add_function(
        "printf",
        context
            .i32_type()
            .fn_type(&[str_type.as_basic_type_enum()], true),
        Some(Linkage::External),
    );
    let main_builder = context.create_builder();
    let main_entry = context.append_basic_block(print_int, "entry");
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

    let print_float = compiler.module.add_function(
        "print_float",
        context
            .void_type()
            .fn_type(&[context.f32_type().as_basic_type_enum()], false),
        Some(Linkage::Private),
    );
    let entry = context.append_basic_block(print_float, "entry");
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

    let print_string = compiler.module.add_function(
        "print_string",
        context.void_type().fn_type(
            &[context
                .i8_type()
                .ptr_type(AddressSpace::Generic)
                .as_basic_type_enum()],
            false,
        ),
        Some(Linkage::Private),
    );
    let entry = context.append_basic_block(print_string, "entry");
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
    for statement in parse_tree {
        match *statement {
            Statement::Function(name, params, return_type, body) => {
                let llvm_params = if let Some(x) = &params {
                    x.iter()
                        .map(|(typ, _)| match *typ {
                            Type::Integer(size) => match size {
                                32 => compiler.context.i32_type().as_basic_type_enum(),
                                64 => compiler.context.i64_type().as_basic_type_enum(),
                                _ => panic!("unimplemented int size"),
                            },
                            Type::Float(size) => match size {
                                32 => compiler.context.f32_type().as_basic_type_enum(),
                                64 => compiler.context.f64_type().as_basic_type_enum(),
                                _ => panic!("unimplemented float size"),
                            },
                            Type::Boolean => compiler.context.bool_type().as_basic_type_enum(),
                            Type::String => str_type.as_basic_type_enum(),
                        })
                        .collect()
                } else {
                    vec![]
                };

                let mut linkage = Some(Linkage::Private);
                if name == "main" {
                    linkage = None;
                }

                let mut llvm_ret: AnyTypeEnum = compiler.context.void_type().as_any_type_enum();
                if let Some(ret) = return_type {
                    llvm_ret = match ret {
                        Type::Integer(size) => match size {
                            32 => compiler.context.i32_type().as_any_type_enum(),
                            64 => compiler.context.i64_type().as_any_type_enum(),
                            _ => panic!("unimplemented int size"),
                        },
                        Type::Float(size) => match size {
                            32 => compiler.context.f32_type().as_any_type_enum(),
                            64 => compiler.context.f64_type().as_any_type_enum(),
                            _ => panic!("unimplemented float size"),
                        },
                        Type::Boolean => compiler.context.bool_type().as_any_type_enum(),
                        Type::String => str_type.as_any_type_enum(),
                    };
                }

                let mut function_type = context.void_type().fn_type(&[], false);
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

                let llvm_func = compiler
                    .module
                    .add_function(name.as_str(), function_type, linkage);

                let entry = compiler.context.append_basic_block(llvm_func, "entry");
                builder.position_at_end(entry);

                if let Some(p) = &params {
                    for (index, (typ, name)) in p.iter().enumerate() {
                        let alloca = compiler.builder.build_alloca(
                            match *typ {
                                Type::Integer(size) => match size {
                                    32 => compiler.context.i32_type().as_basic_type_enum(),
                                    64 => compiler.context.i64_type().as_basic_type_enum(),
                                    _ => panic!("unimplemented int size"),
                                },
                                Type::Float(size) => match size {
                                    32 => compiler.context.f32_type().as_basic_type_enum(),
                                    64 => compiler.context.f64_type().as_basic_type_enum(),
                                    _ => panic!("unimplemented float size"),
                                },
                                Type::Boolean => compiler.context.bool_type().as_basic_type_enum(),
                                Type::String => str_type.as_basic_type_enum(),
                            },
                            "param",
                        );
                        compiler
                            .builder
                            .build_store(alloca, llvm_func.get_params()[index]);
                        compiler.ptrs.insert(name.clone(), alloca);
                    }
                }

                for stmt in &body {
                    compiler.gen_stmt(&llvm_func, *stmt.clone());
                }

                let x = body.last().unwrap();
                match *x.clone() {
                    Statement::Return(_) => continue,
                    _ => builder.build_return(None),
                };
            }
            _ => panic!("non-function at top level"),
        }
    }

    // builder.build_return(Some(&gen(&context, &builder, parse_tree)));

    let _ = fs::create_dir(".build");

    compiler
        .module
        .print_to_file(Path::new(&format!(
            ".build/{}.ll",
            args[1].strip_suffix(".azl").unwrap()
        )))
        .unwrap();

    let arm = TargetTriple::create("arm64-apple-darwin20.5.0");
    compiler.module.set_triple(&arm);
    Target::initialize_native(&InitializationConfig::default()).unwrap();
    let target = Target::from_triple(&arm).unwrap();
    let target_machine = target
        .create_target_machine(
            &arm,
            "cyclone",
            "",
            inkwell::OptimizationLevel::Default,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
        .unwrap();

    target_machine
        .write_to_file(
            &compiler.module,
            FileType::Object,
            Path::new(".build/out.o"),
        )
        .unwrap();

    Command::new("clang")
        .arg(format!("-o{}", args[1].strip_suffix(".azl").unwrap()))
        .arg(".build/out.o")
        .arg("-flto=thin")
        .output()
        .expect("Failed to link");

    let metadata = fs::metadata(Path::new(
        &args[1].strip_suffix(".azl").unwrap().to_string(),
    ))
    .expect("Could not read generated binary.");

    println!("Generated binary of {} Kilobytes.", metadata.len() / 1000);
}
