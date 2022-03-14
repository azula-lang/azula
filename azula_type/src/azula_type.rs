use std::{ops::Deref, rc::Rc};

#[derive(Debug, PartialEq, Clone)]
pub enum AzulaType<'a> {
    Int,
    SizedSignedInt(usize),
    SizedUnsignedInt(usize),
    Str,
    Float,
    SizedFloat(usize),
    Bool,
    Void,
    Pointer(Rc<AzulaType<'a>>),
    Infer,
    Named(String),
    UnknownType(&'a str),
    Array(Rc<AzulaType<'a>>, Option<usize>),
}

impl<'a> From<&'a str> for AzulaType<'a> {
    fn from(val: &'a str) -> Self {
        match val {
            "int" => Self::Int,
            "i8" => Self::SizedSignedInt(8),
            "i16" => Self::SizedSignedInt(16),
            "i32" => Self::SizedSignedInt(32),
            "i64" => Self::SizedSignedInt(64),
            "u8" => Self::SizedUnsignedInt(8),
            "u16" => Self::SizedUnsignedInt(16),
            "u32" => Self::SizedUnsignedInt(32),
            "u64" => Self::SizedUnsignedInt(64),
            "f32" => Self::SizedFloat(32),
            "f64" => Self::SizedFloat(64),
            "str" => Self::Str,
            "float" => Self::Float,
            "bool" => Self::Bool,
            "void" => Self::Void,
            _ => Self::Named(val.to_string()),
        }
    }
}

impl<'a> AzulaType<'a> {
    pub fn is_indexable(&self) -> bool {
        match self {
            AzulaType::Array(..) => true,
            AzulaType::Pointer(nested) => match nested.deref().clone() {
                AzulaType::Str => true,
                _ => false,
            },
            _ => false,
        }
    }
}
