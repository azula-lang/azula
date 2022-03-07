#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind<'a> {
    Identifier(&'a str),
    String(&'a str),
    Char(&'a str),
    Integer(i64),

    BracketOpen,  // (
    BracketClose, // )
    SquareOpen,   // [
    SquareClose,  // ]
    BraceOpen,    // {
    BraceClose,   // }

    Dot,       // .
    Comma,     // ,
    SemiColon, // ;
    Colon,     // :

    Plus,         // +
    Minus,        // -
    Slash,        // /
    Asterisk,     // *
    Power,        // **
    Assign,       // =
    Equal,        // ==
    NotEqual,     // !=
    Bar,          // |
    Or,           // ||
    Ampersand,    // &
    And,          // &&
    Bang,         // !
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    Modulo,       // %

    Function, // func
    Return,   // return
    Var,      // var
    Const,    // const
    True,     // true
    False,    // false
    If,       // If
    Extern,   // extern
    VarArgs,  // varargs
    While,    // while
    Struct,   // struct

    Comment,
    UnknownToken,
    EOF,
}

impl<'a> TokenKind<'a> {
    pub fn get_closing_delimiter(&self) -> Option<Self> {
        match self {
            Self::BracketOpen => Some(Self::BracketClose),
            Self::BraceOpen => Some(Self::BraceClose),
            Self::SquareOpen => Some(Self::SquareClose),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind<'a>, start: usize, end: usize) -> Self {
        Self {
            kind,
            span: Span { start, end },
        }
    }
}
