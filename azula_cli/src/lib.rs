use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};

use azula_codegen::prelude::{Backend, Codegen, OptimizationLevel};
use azula_codegen_llvm::prelude::LLVMCodegen;
use azula_parser::prelude::{Lexer, Parser};
use azula_typecheck::prelude::Typechecker;
// use azula_vm::VM;
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
        files: Vec<String>,

        #[clap(long)]
        release: bool,

        #[clap(long)]
        print_azula_ir: bool,
    },
    Build {
        files: Vec<String>,

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
            files,
            release,
            print_azula_ir,
        } => {
            let result = build(files, ".build/", None, false, *release, *print_azula_ir);

            Command::new(format!("./.build/{}", result))
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        Commands::Build {
            files,
            target,
            emit_llvm,
            release,
            print_azula_ir,
        } => {
            build(
                files,
                "",
                target.as_ref(),
                *emit_llvm,
                *release,
                *print_azula_ir,
            );
        }
    }
}

const STDLIB_STRING: &str = include_str!("../../stdlib/string.azl");
const STDLIB_VEC: &str = include_str!("../../stdlib/vec.azl");
const STDLIB_STRMAP: &str = include_str!("../../stdlib/strmap.azl");

/// Recursively read `path` and all its `import "..."` dependencies, returning
/// a single concatenated source string. `seen` prevents duplicate inclusion.
fn resolve_imports(path: &Path, seen: &mut HashSet<PathBuf>) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if seen.contains(&canonical) {
        return String::new();
    }
    seen.insert(canonical.clone());

    let src = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Could not read file: {}", path.display()));

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut result = String::new();

    for line in src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("import \"") {
            if let Some(import_path) = rest.strip_suffix('"') {
                let dep = dir.join(import_path);
                result.push_str(&resolve_imports(&dep, seen));
                result.push('\n');
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}

fn build(
    files: &[String],
    destination: &str,
    target: Option<&String>,
    emit_llvm: bool,
    release: bool,
    print_azula_ir: bool,
) -> String {
    let mut seen = HashSet::new();
    let user_source: String = files
        .iter()
        .map(|f| resolve_imports(Path::new(f), &mut seen))
        .collect::<Vec<_>>()
        .join("\n");

    let input = format!("{}\n{}\n{}\n{}", STDLIB_STRING, STDLIB_VEC, STDLIB_STRMAP, user_source);

    let primary = files.last().unwrap();

    let lexer: Lexer = input.as_str().into();
    let mut parser = Parser::new(input.as_str(), lexer);
    let parsed = parser.parse();
    for error in &parser.errors {
        error.print_stdout(&input, primary.as_str());
    }

    if !parser.errors.is_empty() {
        exit(1);
    }

    let mut typecheck = Typechecker::new(parsed);
    let result = typecheck.typecheck();
    for err in typecheck.errors {
        err.print_stdout(&input, primary.as_str());
    }

    if result.is_err() {
        exit(1);
    }

    let root = result.unwrap();

    let name = primary.trim_end_matches(".azl").to_string();

    let mut codegen = Codegen::new(&name, root);
    codegen.codegen();
    codegen.insert_implicit_return();

    if print_azula_ir {
        println!("{}", codegen.module);
    }

    LLVMCodegen::codegen(
        &name,
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

    // println!("{:?}", VM::new().run(codegen.module));

    return name;
}
