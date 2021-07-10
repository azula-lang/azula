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
    targets::{FileType, InitializationConfig, Target, TargetTriple},
    AddressSpace,
};

use crate::codegen::*;

#[macro_use]
extern crate lalrpop_util;

fn main() {
    // Read in the source file
    let args: Vec<String> = std::env::args().collect();
    let source_file =
        fs::read_to_string(Path::new(&args[1])).expect("Could not read supplied file.");

    // Generate parse tree from source
    let parse_tree = parser::parser::ProgramParser::new()
        .parse(&source_file)
        .unwrap();

    // Construct the compiler struct using LLVM constructs
    let context = Context::create();

    let module = context.create_module("main_mod");

    let builder = context.create_builder();
    let mut compiler = Compiler {
        context: &context,
        builder: &builder,
        module,
        ptrs: HashMap::new(),
        str_type: context.i8_type().ptr_type(AddressSpace::Generic),
    };

    // Add the print functions
    compiler.add_print_funcs();
    // Generate the code
    compiler.gen(parse_tree);

    // Create the temporary .build directory to store the object files
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
