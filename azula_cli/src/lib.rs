use std::{
    fs,
    process::{exit, Command},
};

use azula_codegen::prelude::{Backend, Codegen, OptimizationLevel};
use azula_codegen_llvm::prelude::LLVMCodegen;
use azula_parser::prelude::{Lexer, Parser};
use azula_typecheck::prelude::Typechecker;
use clap::{StructOpt, Subcommand};

/// Azula command line
#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct AzulaCLI {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run {
        file: String,

        #[clap(long)]
        release: bool,

        #[clap(long)]
        print_azula_ir: bool,
    },
    Build {
        file: String,

        #[clap(long)]
        target: Option<String>,

        #[clap(long)]
        emit_llvm: bool,

        #[clap(long)]
        release: bool,

        #[clap(long)]
        print_azula_ir: bool,
    },
}

pub fn run() {
    let args = AzulaCLI::parse();

    match &args.command {
        Commands::Run {
            file,
            release,
            print_azula_ir,
        } => {
            let result = build(file, ".build/", None, false, *release, *print_azula_ir);

            Command::new(format!("./.build/{}", result))
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        Commands::Build {
            file,
            target,
            emit_llvm,
            release,
            print_azula_ir,
        } => {
            build(
                file,
                "",
                target.as_ref(),
                *emit_llvm,
                *release,
                *print_azula_ir,
            );
        }
    }
}

fn build<'a>(
    name: &'a str,
    destination: &'a str,
    target: Option<&String>,
    emit_llvm: bool,
    release: bool,
    print_azula_ir: bool,
) -> &'a str {
    let input = fs::read_to_string(name).unwrap();
    let lexer: Lexer = input.as_str().into();
    let mut parser = Parser::new(input.as_str(), lexer);
    let parsed = parser.parse();
    for error in &parser.errors {
        error.print_stdout(&input, name);
    }

    if !parser.errors.is_empty() {
        exit(1);
    }

    let mut typecheck = Typechecker::new(parsed);
    let result = typecheck.typecheck();
    for err in typecheck.errors {
        err.print_stdout(&input, name);
    }

    if result.is_err() {
        exit(1);
    }

    let root = result.unwrap();

    let name = name.trim_end_matches(".azl");

    let mut codegen = Codegen::new(name, root);
    codegen.codegen();
    codegen.insert_implicit_return();

    if print_azula_ir {
        println!("{}", codegen.module);
    }

    LLVMCodegen::codegen(
        name,
        destination,
        emit_llvm,
        target,
        if release {
            OptimizationLevel::Aggressive
        } else {
            OptimizationLevel::Default
        },
        codegen.module,
    )
    .unwrap();

    return name;
}
