use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum AzulaType<'a> {
    Int,
    Str,
    Float,
    Bool,
    Void,
    Pointer(Rc<AzulaType<'a>>),
    Infer,
    Named(&'a str),
    UnknownType(&'a str),
    Array(Rc<AzulaType<'a>>, Option<usize>),
}

impl<'a> From<&'a str> for AzulaType<'a> {
    fn from(val: &'a str) -> Self {
        match val {
            "int" => Self::Int,
            "str" => Self::Str,
            "float" => Self::Float,
            "bool" => Self::Bool,
            "void" => Self::Void,
            _ => Self::Named(val),
        }
    }
}
