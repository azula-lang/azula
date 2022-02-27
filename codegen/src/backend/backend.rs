use std::error::Error;

use azula_ir::prelude::Module;

pub trait Backend<'a> {
    fn codegen(module: Module<'a>) -> Result<(), Box<dyn Error>>;
}
