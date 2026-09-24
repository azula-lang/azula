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

impl<'a> ToString for AzulaType<'a> {
    fn to_string(&self) -> String {
        match self {
            AzulaType::Int => "int".to_string(),
            AzulaType::SizedSignedInt(size) => match size {
                8 => "i8",
                16 => "i16",
                32 => "i32",
                64 => "i64",
                _ => "unknown int size",
            }
            .to_string(),
            AzulaType::SizedUnsignedInt(size) => match size {
                8 => "u8",
                16 => "u16",
                32 => "u32",
                64 => "u64",
                _ => "unknown int size",
            }
            .to_string(),
            AzulaType::Str => "str".to_string(),
            AzulaType::Float => "float".to_string(),
            AzulaType::SizedFloat(size) => match size {
                8 => "f8",
                16 => "f16",
                32 => "f32",
                64 => "f64",
                _ => "unknown float size",
            }
            .to_string(),
            AzulaType::Bool => "bool".to_string(),
            AzulaType::Void => "void".to_string(),
            AzulaType::Pointer(ptr) => format!("{}", ptr.to_string()),
            AzulaType::Infer => "infer".to_string(),
            AzulaType::Named(s) => s.clone(),
            AzulaType::UnknownType(_) => "unknown".to_string(),
            AzulaType::Array(typ, size) => match size {
                Some(s) => format!("[{:?}; {:?}]", typ.to_string(), s),
                None => format!("[{:?}]", typ.to_string()),
            },
        }
    }
}

impl<'a> AzulaType<'a> {
    pub fn is_indexable(&self) -> bool {
        match self {
            AzulaType::Array(..) => true,
            AzulaType::Pointer(..) => true,
            AzulaType::Str => true,
            _ => false,
        }
    }
}
