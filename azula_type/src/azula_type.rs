use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum AzulaType<'a> {
    Int,
    Str,
    Bool,
    Void,
    Pointer(Rc<AzulaType<'a>>),
    Infer,
    Named(&'a str),
    UnknownType(&'a str),
}

impl<'a> From<&'a str> for AzulaType<'a> {
    fn from(val: &'a str) -> Self {
        match val {
            "int" => Self::Int,
            "str" => Self::Str,
            "bool" => Self::Bool,
            "void" => Self::Void,
            _ => Self::Named(val),
        }
    }
}
