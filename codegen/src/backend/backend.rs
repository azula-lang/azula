use std::error::Error;

use azula_ir::prelude::Module;

pub trait Backend<'a> {
    fn codegen(
        name: &'a str,
        destination: &'a str,
        emit: bool,
        target: Option<&String>,
        opt_level: OptimizationLevel,
        module: Module<'a>,
    ) -> Result<(), Box<dyn Error>>;
}

#[derive(PartialEq)]
pub enum OptimizationLevel {
    None,
    Default,
    Aggressive,
}
