use codespan_reporting::diagnostic::Label;

use crate::parser::ast::Type;

#[derive(Debug)]
pub enum AzulaError {
    FunctionIncorrectParams {
        expected: Type,
        found: Type,
        function_l: usize,
        function_r: usize,
        l: usize,
        r: usize,
    },
    NonBooleanIfCond {
        found: Type,
        l: usize,
        r: usize,
    },
    FunctionNotFound {
        name: String,
        l: usize,
        r: usize,
    },
    VariableNotFound {
        name: String,
        l: usize,
        r: usize,
    },
    VariableWrongType {
        annotated: Type,
        found: Type,
        l: usize,
        r: usize,
    },
}

impl AzulaError {
    pub fn labels(&self, file_id: usize) -> Vec<Label<usize>> {
        match self {
            AzulaError::FunctionIncorrectParams {
                expected,
                found,
                function_l,
                function_r,
                l,
                r,
            } => vec![
                Label::primary(file_id, *l..*r).with_message(format!(
                    "Incorrect function parameter, expected {:?}, got {:?}",
                    expected, found
                )),
                Label::secondary(file_id, *function_l..*function_r),
            ],
            AzulaError::NonBooleanIfCond { found, l, r } => {
                vec![Label::primary(file_id, *l..*r)
                    .with_message(format!("Non-boolean used in conditional, got {:?}", found))]
            }
            AzulaError::FunctionNotFound { name, l, r } => {
                vec![Label::primary(file_id, *l..*r)
                    .with_message(format!("Function {} not found", name))]
            }
            AzulaError::VariableNotFound { name, l, r } => vec![Label::primary(file_id, *l..*r)
                .with_message(format!("Variable {} not found", name))],
            AzulaError::VariableWrongType {
                annotated,
                found,
                l,
                r,
            } => vec![Label::primary(file_id, *l..*r).with_message(format!(
                "Variable has wrong type, value is {:?}, variable expects {:?}",
                found, annotated
            ))],
        }
    }
}
