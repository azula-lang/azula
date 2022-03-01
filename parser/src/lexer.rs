use std::{iter::Peekable, str::Chars};

use crate::token::*;

/// Lexer transforms a &str into Tokens we can parse
pub struct Lexer<'a> {
    pub input: &'a str,
    pub peekable: Peekable<Chars<'a>>,

    pub index: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new Lexer from a given input str
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            peekable: input.chars().peekable(),
            index: 0,
        }
    }

    fn next(&mut self) -> Option<char> {
        self.index += 1;
        self.peekable.next()
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        self.skip_whitespace();
        let start = self.index;
        if let Some(char) = self.next() {
            return Some(match char {
                '(' => Token::new(TokenKind::BracketOpen, start, self.index),
                ')' => Token::new(TokenKind::BracketClose, start, self.index),
                '[' => Token::new(TokenKind::SquareOpen, start, self.index),
                ']' => Token::new(TokenKind::SquareClose, start, self.index),
                '{' => Token::new(TokenKind::BraceOpen, start, self.index),
                '}' => Token::new(TokenKind::BraceClose, start, self.index),
                '.' => Token::new(TokenKind::Dot, start, self.index),
                ',' => Token::new(TokenKind::Comma, start, self.index),
                ';' => Token::new(TokenKind::SemiColon, start, self.index),
                ':' => Token::new(TokenKind::Colon, start, self.index),
                '+' => Token::new(TokenKind::Plus, start, self.index),
                '-' => Token::new(TokenKind::Minus, start, self.index),
                '/' => match self.peekable.peek() {
                    Some('/') => {
                        // We found a comment - skip for now (maybe use for docs later)
                        while let Some(val) = self.peekable.peek() {
                            match val {
                                '\n' => break,
                                _ => self.next(),
                            };
                        }

                        Token::new(TokenKind::Comment, start, self.index)
                    }
                    _ => Token::new(TokenKind::Slash, start, self.index),
                },
                '*' => match self.peekable.peek() {
                    Some('*') => {
                        self.next();
                        Token::new(TokenKind::Power, start, self.index)
                    }
                    _ => Token::new(TokenKind::Asterisk, start, self.index),
                },
                '=' => match self.peekable.peek() {
                    Some('=') => {
                        self.next();
                        Token::new(TokenKind::Equal, start, self.index)
                    }
                    _ => Token::new(TokenKind::Assign, start, self.index),
                },
                '!' => match self.peekable.peek() {
                    Some('=') => {
                        self.next();
                        Token::new(TokenKind::NotEqual, start, self.index)
                    }
                    _ => Token::new(TokenKind::Bang, start, self.index),
                },
                '|' => match self.peekable.peek() {
                    Some('|') => {
                        self.next();
                        Token::new(TokenKind::Or, start, self.index)
                    }
                    _ => Token::new(TokenKind::Bar, start, self.index),
                },
                '&' => match self.peekable.peek() {
                    Some('&') => {
                        self.next();
                        Token::new(TokenKind::And, start, self.index)
                    }
                    _ => Token::new(TokenKind::Ampersand, start, self.index),
                },
                '<' => match self.peekable.peek() {
                    Some('=') => {
                        self.next();
                        Token::new(TokenKind::LessEqual, start, self.index)
                    }
                    _ => Token::new(TokenKind::Less, start, self.index),
                },
                '>' => match self.peekable.peek() {
                    Some('=') => {
                        self.next();
                        Token::new(TokenKind::GreaterEqual, start, self.index)
                    }
                    _ => Token::new(TokenKind::Greater, start, self.index),
                },
                '%' => Token::new(TokenKind::Modulo, start, self.index),
                '"' => {
                    while let Some(val) = self.peekable.peek() {
                        match val {
                            _ if *val == char => break,
                            _ => self.next(),
                        };
                    }

                    self.next();

                    let str = &self.input[start + 1..self.index - 1];
                    Token::new(TokenKind::String(str), start, self.index)
                }
                '\'' => {
                    while let Some(val) = self.peekable.peek() {
                        match val {
                            _ if *val == char => break,
                            _ => self.next(),
                        };
                    }

                    self.next();

                    let str = &self.input[start + 1..self.index - 1];
                    Token::new(TokenKind::Char(str), start, self.index)
                }
                '0'..='9' => {
                    while let Some(val) = self.peekable.peek() {
                        match val {
                            '0'..='9' => self.next(),
                            _ => break,
                        };
                    }

                    let identifier = &self.input[start..self.index];
                    Token::new(
                        TokenKind::Integer(identifier.parse().unwrap()),
                        start,
                        self.index,
                    )
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    while let Some(val) = self.peekable.peek() {
                        match val {
                            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => self.next(),
                            _ => break,
                        };
                    }

                    let identifier = &self.input[start..self.index];
                    self.tokenize_identifier(identifier, start)
                }
                _ => Token::new(TokenKind::UnknownToken, start, self.index),
            });
        }

        None
    }

    fn skip_whitespace(&mut self) {
        while let Some(val) = self.peekable.peek() {
            match val {
                ' ' | '\n' => self.next(),
                _ => break,
            };
        }
    }

    fn tokenize_identifier(&mut self, value: &'a str, start: usize) -> Token<'a> {
        match value {
            "func" => Token::new(TokenKind::Function, start, self.index),
            "return" => Token::new(TokenKind::Return, start, self.index),
            "var" => Token::new(TokenKind::Var, start, self.index),
            "const" => Token::new(TokenKind::Const, start, self.index),
            "true" => Token::new(TokenKind::True, start, self.index),
            "false" => Token::new(TokenKind::False, start, self.index),
            "if" => Token::new(TokenKind::If, start, self.index),
            "extern" => Token::new(TokenKind::Extern, start, self.index),
            "varargs" => Token::new(TokenKind::VarArgs, start, self.index),
            _ => Token::new(TokenKind::Identifier(value), start, self.index),
        }
    }
}

// Allows us to call .into() on a &str
impl<'a> From<&'a str> for Lexer<'a> {
    fn from(source: &'a str) -> Self {
        Self {
            input: source,
            peekable: source.chars().peekable(),
            index: 0,
        }
    }
}

// The core of our Lexer, allowing us to provide iterator methods
impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! lexer_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let lexer: Lexer = input.into();
                    assert_eq!(lexer.collect::<Vec<Token>>(), expected);
                }
            )*
        }
    }

    lexer_tests! {
        skip_whitespace: (
            "   (    )    ",
            vec![
                Token::new(TokenKind::BracketOpen, 3, 4),
                Token::new(TokenKind::BracketClose, 8, 9),
            ],
        ),
        bracket_open: (
            "(",
            vec![Token::new(TokenKind::BracketOpen, 0, 1)],
        ),
        bracket_close: (
            ")",
            vec![Token::new(TokenKind::BracketClose, 0, 1)],
        ),
        dot: (
            ".",
            vec![Token::new(TokenKind::Dot, 0, 1)],
        ),
        comma: (
            ",",
            vec![Token::new(TokenKind::Comma, 0, 1)],
        ),
        semicolon: (
            ";",
            vec![Token::new(TokenKind::SemiColon, 0, 1)],
        ),
        colon: (
            ":",
            vec![Token::new(TokenKind::Colon, 0, 1)],
        ),
        plus: (
            "+",
            vec![Token::new(TokenKind::Plus, 0, 1)],
        ),
        minus: (
            "-",
            vec![Token::new(TokenKind::Minus, 0, 1)],
        ),
        slash: (
            "/",
            vec![Token::new(TokenKind::Slash, 0, 1)],
        ),
        asterisk: (
            "*",
            vec![Token::new(TokenKind::Asterisk, 0, 1)],
        ),
        power: (
            "**",
            vec![Token::new(TokenKind::Power, 0, 2)],
        ),
        assign: (
            "=",
            vec![Token::new(TokenKind::Assign, 0, 1)],
        ),
        equal: (
            "==",
            vec![Token::new(TokenKind::Equal, 0, 2)],
        ),
        not_equal: (
            "!=",
            vec![Token::new(TokenKind::NotEqual, 0, 2)],
        ),
        bang: (
            "!",
            vec![Token::new(TokenKind::Bang, 0, 1)],
        ),
        bar: (
            "|",
            vec![Token::new(TokenKind::Bar, 0, 1)],
        ),
        or: (
            "||",
            vec![Token::new(TokenKind::Or, 0, 2)],
        ),
        ampersand: (
            "&",
            vec![Token::new(TokenKind::Ampersand, 0, 1)],
        ),
        less: (
            "<",
            vec![Token::new(TokenKind::Less, 0, 1)],
        ),
        less_equal: (
            "<=",
            vec![Token::new(TokenKind::LessEqual, 0, 2)],
        ),
        greater: (
            ">",
            vec![Token::new(TokenKind::Greater, 0, 1)],
        ),
        greater_equal: (
            ">=",
            vec![Token::new(TokenKind::GreaterEqual, 0, 2)],
        ),
        and: (
            "&&",
            vec![Token::new(TokenKind::And, 0, 2)],
        ),
        modulo: (
            "%",
            vec![Token::new(TokenKind::Modulo, 0, 1)],
        ),
        string: (
            "\"test\" \"another_test$$%\"",
            vec![Token::new(TokenKind::String("test"), 0, 6), Token::new(TokenKind::String("another_test$$%"), 7, 24)],
        ),
        char: (
            "'a' '$'",
            vec![Token::new(TokenKind::Char("a"), 0, 3), Token::new(TokenKind::Char("$"), 4, 7)],
        ),
        number: (
            "49102",
            vec![Token::new(TokenKind::Integer(49102), 0, 5)],
        ),
        comment: (
            "
            // a comment
            test
            ",
            vec![Token::new(TokenKind::Comment, 13, 25), Token::new(TokenKind::Identifier("test"), 38, 42)],
        ),
        identifier: (
            "identifier_test",
            vec![Token::new(TokenKind::Identifier("identifier_test"), 0, 15)],
        ),
        function: (
            "func test",
            vec![Token::new(TokenKind::Function, 0, 4), Token::new(TokenKind::Identifier("test"), 5, 9)],
        ),
        returns: (
            "return",
            vec![Token::new(TokenKind::Return, 0, 6)],
        ),
        var: (
            "var",
            vec![Token::new(TokenKind::Var, 0, 3)],
        ),
        const_stmt: (
            "const",
            vec![Token::new(TokenKind::Const, 0, 5)],
        ),
        true_stmt: (
            "true",
            vec![Token::new(TokenKind::True, 0, 4)],
        ),
        false_stmt: (
            "false",
            vec![Token::new(TokenKind::False, 0, 5)],
        ),
        if_stmt: (
            "if",
            vec![Token::new(TokenKind::If, 0, 2)],
        ),
    }
}
