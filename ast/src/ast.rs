use std::rc::Rc;

use azula_type::prelude::AzulaType;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'a> {
    Root(Body<'a>),
    Block(Body<'a>),
    Function {
        name: &'a str,
        args: Vec<TypedIdentifier<'a>>,
        returns: AzulaType<'a>,
        body: Rc<Statement<'a>>,
        span: Span,
    },
    Return(Option<ExpressionNode<'a>>, Span),
    Assign(
        bool,
        String,
        Option<AzulaType<'a>>,
        ExpressionNode<'a>,
        Span,
    ),
    ExpressionStatement(ExpressionNode<'a>, Span),
    If(ExpressionNode<'a>, Body<'a>, Span),
    ExternFunction {
        name: &'a str,
        varargs: bool,
        args: Vec<AzulaType<'a>>,
        returns: AzulaType<'a>,
        span: Span,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<'a> {
    Infix(Rc<ExpressionNode<'a>>, Operator, Rc<ExpressionNode<'a>>),
    Integer(i64),
    Float(f64),
    Identifier(String),
    Boolean(bool),
    String(String),
    FunctionCall {
        function: Rc<ExpressionNode<'a>>,
        args: Vec<ExpressionNode<'a>>,
    },
    Not(Rc<ExpressionNode<'a>>),
    Pointer(Rc<ExpressionNode<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionNode<'a> {
    pub expression: Expression<'a>,
    pub typed: AzulaType<'a>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power,
    Or,
    And,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
}

pub type Body<'a> = Vec<Statement<'a>>;
pub type TypedIdentifier<'a> = (AzulaType<'a>, &'a str);

// #[derive(Debug, PartialEq, Clone)]
// pub enum Type<'a> {
//     Basic(&'a str),
//     WithArgument(&'a str, Rc<Type<'a>>),
//     Pointer(Rc<Type<'a>>),
//     Infer,
//     None,
// }

#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
