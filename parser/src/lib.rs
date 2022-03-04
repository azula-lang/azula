mod lexer;
mod parser;
mod token;

pub mod prelude {
    pub use crate::lexer::Lexer;
    pub use crate::parser::Parser;
    pub use crate::token::{Span, Token, TokenKind};
}
