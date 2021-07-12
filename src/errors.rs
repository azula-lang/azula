use codespan_reporting::diagnostic::Label;

use crate::parser::ast::Type;

#[derive(Debug, Clone)]
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
    InvalidToken {
        l: usize,
        r: usize,
    },
    UnexpectedEOF {
        l: usize,
        r: usize,
    },
}

impl AzulaError {
    pub fn labels(&self, file_id: usize) -> Vec<Label<usize>> {
        match self {
            AzulaError::FunctionIncorrectParams {
                function_l,
                function_r,
                l,
                r,
                ..
            } => vec![
                Label::primary(file_id, *l..*r).with_message(self.message()),
                Label::secondary(file_id, *function_l..*function_r).with_message("as defined here"),
            ],
            AzulaError::NonBooleanIfCond { l, r, .. } => {
                vec![Label::primary(file_id, *l..*r).with_message(self.message())]
            }
            AzulaError::FunctionNotFound { l, r, .. } => {
                vec![Label::primary(file_id, *l..*r).with_message(self.message())]
            }
            AzulaError::VariableNotFound { l, r, .. } => {
                vec![Label::primary(file_id, *l..*r).with_message(self.message())]
            }
            AzulaError::VariableWrongType { l, r, .. } => {
                vec![Label::primary(file_id, *l..*r).with_message(self.message())]
            }
            AzulaError::InvalidToken { l, r, .. } => {
                vec![Label::primary(file_id, *l..*r).with_message(self.message())]
            }
            AzulaError::UnexpectedEOF { l, r, .. } => {
                vec![Label::primary(file_id, *l..*r).with_message(self.message())]
            }
        }
    }

    pub fn message(&self) -> String {
        match self {
            AzulaError::FunctionIncorrectParams {
                expected, found, ..
            } => format!(
                "Incorrect function parameter, expected {:?}, got {:?}",
                expected, found
            ),
            AzulaError::NonBooleanIfCond { found, .. } => {
                format!("Non-boolean used in conditional, got {:?}", found)
            }
            AzulaError::FunctionNotFound { name, .. } => format!("Function {} not found", name),
            AzulaError::VariableNotFound { name, .. } => format!("Variable {} not found", name),
            AzulaError::VariableWrongType {
                annotated, found, ..
            } => format!(
                "Variable has wrong type, value is {:?}, variable expects {:?}",
                found, annotated
            ),
            AzulaError::InvalidToken { .. } => "Invalid token found".to_string(),
            AzulaError::UnexpectedEOF { .. } => "Unexpected EOF".to_string(),
        }
    }

    pub fn error_code(&self) -> i32 {
        match self {
            AzulaError::FunctionIncorrectParams { .. } => 0,
            AzulaError::NonBooleanIfCond { .. } => 1,
            AzulaError::FunctionNotFound { .. } => 2,
            AzulaError::VariableNotFound { .. } => 3,
            AzulaError::VariableWrongType { .. } => 4,
            AzulaError::InvalidToken { .. } => 5,
            AzulaError::UnexpectedEOF { .. } => 6,
        }
    }
}
