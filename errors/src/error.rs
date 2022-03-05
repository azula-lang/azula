use ansi_term::Colour::*;
use line_col::LineColLookup;

#[derive(Debug)]
pub enum ErrorType {
    // Lex & Parse
    UnknownToken,
    ExpectedToken(String, Option<String>),
    ExpectedExpression(String),
    ExpectedStatement(String),
    UnexpectedEOF,
    InvalidEscape,

    // Typecheck
    UnknownVariable(String),
    MismatchedTypes(String, String),
    NonOperatorType(String, String),
    IncorrectFunctionReturn(String, String),
    NonBoolCondition(String),
    FunctionNotFound(String),
    MismatchedAssignTypes(String, String),
    NonGlobalConstant,
    NonIntIndex(String),
    NonArrayInIndex(String),
}

impl<'a> ErrorType {
    fn error_message(&self) -> String {
        match self {
            ErrorType::UnknownToken => "Unknown token found".to_string(),
            ErrorType::ExpectedToken(expected, got) => match got {
                Some(got) => format!("Expected token: {}, got {}", expected, got),
                None => format!("Expected token: {}", expected),
            },
            ErrorType::ExpectedExpression(got) => format!("Expected an expression, got {:?}", got),
            ErrorType::ExpectedStatement(got) => format!("Expected a statement, got {:?}", got),
            ErrorType::UnexpectedEOF => "Unexpected EOF".to_string(),
            ErrorType::InvalidEscape => "Invalid use of escape".to_string(),
            ErrorType::UnknownVariable(name) => format!("Unknown variable {:?}", name),
            ErrorType::MismatchedTypes(left, right) => {
                format!("Mismatched types: {:?} and {:?}", left, right)
            }
            ErrorType::NonOperatorType(typ, operator) => {
                format!("Type {:?} cannot be used with operator {:?}", typ, operator)
            }
            ErrorType::IncorrectFunctionReturn(wanted, got) => {
                format!(
                    "Function returns wrong value, wanted {:?}, got {:?}",
                    wanted, got
                )
            }
            ErrorType::NonBoolCondition(got) => {
                format!("If condition must be a boolean, got {}", got)
            }
            ErrorType::FunctionNotFound(func) => {
                format!("Function {} not found", func)
            }
            ErrorType::MismatchedAssignTypes(wanted, got) => {
                format!(
                    "Mismatched types in assign, expected {}, got {}",
                    wanted, got
                )
            }
            ErrorType::NonGlobalConstant => {
                "Only constant assigns are allowed at top-level".to_string()
            }
            ErrorType::NonIntIndex(got) => {
                format!("Index must be integer, got {}", got)
            }
            ErrorType::NonArrayInIndex(got) => {
                format!("Cannot index non-array, got {}", got)
            }
        }
    }
}

#[derive(Debug)]
pub struct AzulaError {
    pub error_type: ErrorType,
    pub start: usize,
    pub end: usize,
}

impl AzulaError {
    pub fn new(error_type: ErrorType, start: usize, end: usize) -> Self {
        Self {
            error_type,
            start,
            end,
        }
    }

    pub fn print_stdout(&self, source: &str, filename: &str) {
        let lookup = LineColLookup::new(source);
        println!(
            "{}: {}",
            Red.paint("ERROR"),
            self.error_type.error_message()
        );
        let show_start = self.read_back_until_new_line(source, self.start - 1);
        let show_end = self.read_forward_until_new_line(source, self.end - 1);
        let (line_number, col) = lookup.get(self.start);
        // println!(
        //     "{}",
        //     Red.paint(format!("Line: {} Column: {}", line_number, col))
        // );
        println!(
            "{}",
            Red.paint(format!("-> {}:{}:{}", filename, line_number, col))
        );
        print!("{}", &source[show_start + 1..self.start]);
        print!("{}", White.paint(&source[self.start..self.end]));
        print!("{}\n", &source[self.end..show_end]);
        println!(
            "{}{}",
            " ".repeat(col - 1),
            Red.paint("^".repeat(self.end - self.start))
        );
    }

    fn read_back_until_new_line(&self, source: &str, mut point: usize) -> usize {
        let mut char = source.as_bytes()[point] as char;
        while char != '\n' {
            point -= 1;
            if point == 0 {
                return point;
            }
            char = source.as_bytes()[point] as char;
        }
        return point;
    }

    fn read_forward_until_new_line(&self, source: &str, mut point: usize) -> usize {
        let mut char = source.as_bytes()[point] as char;
        while char != '\n' {
            point += 1;
            if point >= source.len() {
                return point;
            }
            char = source.as_bytes()[point] as char;
        }
        return point;
    }
}
