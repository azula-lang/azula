use std::{fs, process::exit};

use azula_codegen::prelude::{Backend, Codegen};
use azula_codegen_llvm::prelude::LLVMCodegen;
use azula_parser::prelude::*;
use azula_typecheck::prelude::Typechecker;

fn main() {
    let input = fs::read_to_string("test.azl").unwrap();
    let lexer: Lexer = input.as_str().into();
    let mut parser = Parser::new(input.as_str(), lexer);
    let parsed = parser.parse();
    for error in &parser.errors {
        error.print_stdout(&input, "test.azl");
    }

    if !parser.errors.is_empty() {
        exit(1);
    }

    let mut typecheck = Typechecker::new(parsed);
    let result = typecheck.typecheck();
    for err in typecheck.errors {
        err.print_stdout(&input, "test.azl");
    }

    if result.is_err() {
        exit(1);
    }

    let root = result.unwrap();

    let mut codegen = Codegen::new("main", root);
    codegen.codegen();
    codegen.insert_implicit_return();

    println!("{}", codegen.module);

    LLVMCodegen::codegen(codegen.module).unwrap();
}
