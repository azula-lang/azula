mod instruction;
mod module;

pub mod prelude {
    pub use crate::instruction::{Instruction, Value};
    pub use crate::module::{Block, ExternFunction, Function, GlobalValue, Module, Struct};
}
