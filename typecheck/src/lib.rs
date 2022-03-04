#[macro_use]
extern crate maplit;

mod typecheck;

pub mod prelude {
    pub use crate::typecheck::Typechecker;
}
