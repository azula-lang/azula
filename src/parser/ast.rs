type Block = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Number(f64, usize, usize),
    Identifier(String, usize, usize),
    Boolean(bool, usize, usize),
    String(String, usize, usize),

    Op(Box<Expr>, Opcode, Box<Expr>, usize, usize),
    FunctionCall(String, Vec<Expr>, usize, usize),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(
        Option<String>,
        String,
        Option<Type>,
        Box<Expr>,
        usize,
        usize,
    ),
    Reassign(String, Box<Expr>, usize, usize),
    Function(
        String,
        Option<Vec<(Type, String)>>,
        Option<Type>,
        Block,
        usize,
        usize,
    ),
    Macro(
        String,
        Option<Vec<(Type, String)>>,
        Option<Type>,
        Block,
        usize,
        usize,
    ),
    Return(Option<Box<Expr>>, usize, usize),
    Expression(Box<Expr>, usize, usize),
    If(Box<Expr>, Block, usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Integer(i32),
    Float(i32),
    String,
    Boolean,
    Void,

    Generic(i32),
}

impl Type {
    pub fn from_string(typ: String) -> Option<Type> {
        match typ.as_str() {
            "int" | "int32" => Some(Type::Integer(32)),
            "int64" => Some(Type::Integer(64)),
            "float" | "float32" => Some(Type::Float(32)),
            "float64" => Some(Type::Float(64)),
            "string" => Some(Type::String),
            "bool" => Some(Type::Boolean),
            "T" => Some(Type::Generic(0)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Mul,
    Div,
    Add,
    Sub,
    Rem,

    Eq,
    NotEq,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,

    Or,
    And,
}
