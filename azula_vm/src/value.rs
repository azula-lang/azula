use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Integer(i64),
    String(String),
    Pointer(Box<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{}", s),
            Value::Pointer(p) => match *p.clone() {
                Value::Null => write!(f, "null"),
                Value::Integer(i) => write!(f, "*{}", i),
                Value::String(s) => write!(f, "{}", s),
                Value::Pointer(_) => write!(f, "pointer"),
            },
        }
    }
}
