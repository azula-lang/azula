use std::{iter::Peekable, rc::Rc};

use azula_ast::prelude::*;
use azula_error::prelude::*;
use azula_type::prelude::AzulaType;

use crate::{
    prelude::Lexer,
    token::{Token, TokenKind},
};

type OperatorPrecedence = u8;

const LOWEST: OperatorPrecedence = 0;
const COMPARISON: OperatorPrecedence = 1;
const EQUALS: OperatorPrecedence = 2;
const LESS_GREATER: OperatorPrecedence = 3;
const SUM: OperatorPrecedence = 4;
const PRODUCT: OperatorPrecedence = 5;
const PREFIX: OperatorPrecedence = 6;
const CALL: OperatorPrecedence = 7;

pub struct Parser<'a> {
    source: &'a str,
    lexer: Peekable<Lexer<'a>>,

    pub errors: Vec<AzulaError>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, lexer: Lexer<'a>) -> Self {
        Self {
            source,
            lexer: lexer.peekable(),
            errors: vec![],
        }
    }

    pub fn parse(&mut self) -> Statement<'a> {
        Statement::Root(self.parse_block(TokenKind::EOF))
    }

    fn parse_statement(&mut self) -> Option<Statement<'a>> {
        let token = self.lexer.peek();
        if token.is_none() {
            return None;
        }
        let token = token.unwrap();

        match token.kind {
            TokenKind::Function => self.parse_function(),
            TokenKind::Extern => self.parse_extern_function(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Var => self.parse_assign(true),
            TokenKind::Const => self.parse_assign(false),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::SemiColon => {
                self.lexer.next();
                None
            }
            TokenKind::Comment => {
                self.lexer.next();
                None
            }
            _ => {
                let expr = match self.parse_expression(LOWEST) {
                    Some(node) => node,
                    None => return None,
                };

                if self.lexer.peek().unwrap().kind == TokenKind::Assign {
                    return self.parse_reassign(expr.clone());
                }

                if !self.expect_peek(TokenKind::SemiColon) {
                    return None;
                }

                Some(Statement::ExpressionStatement(expr.clone(), expr.span))
            }
        }
    }

    fn parse_block(&mut self, ending: TokenKind) -> Vec<Statement<'a>> {
        let mut tok = self.lexer.peek().is_some();
        let mut statements = vec![];
        while tok {
            if self.lexer.peek().unwrap().kind == ending {
                break;
            }
            let stmt = self.parse_statement();
            tok = self.lexer.peek().is_some();
            if stmt.is_none() {
                // This could cause issues - not sure how it would occur
                continue;
            }
            statements.push(stmt.unwrap());
        }
        statements
    }

    fn parse_function(&mut self) -> Option<Statement<'a>> {
        // func
        let start_token = self.lexer.next().unwrap();

        // Parse name of the function
        let tok = self.lexer.next();
        let ident = match tok {
            Some(v) if matches!(v.kind, TokenKind::Identifier(_)) => {
                if let TokenKind::Identifier(val) = v.kind {
                    val
                } else {
                    "anon"
                }
            }
            _ => return None,
        };

        // Parse function arguments
        let mut args = vec![];
        if let Some(tok) = self.lexer.peek() {
            if tok.kind == TokenKind::BracketOpen {
                args = self.parse_typed_identifier_list(TokenKind::BracketOpen);
            }
        }

        // Parse return type
        let mut returns = AzulaType::Void;
        if let Some(tok) = self.lexer.peek() {
            // If it's a colon, we have a type
            if tok.kind == TokenKind::Colon {
                self.lexer.next();

                returns = self.parse_type();
            }
        }

        if !self.expect_peek(TokenKind::BraceOpen) {
            return None;
        }
        self.lexer.next();

        let body = self.parse_block(TokenKind::BraceClose);

        if !self.expect_peek(TokenKind::BraceClose) {
            return None;
        }

        let end_token = self.lexer.next().unwrap();

        Some(Statement::Function {
            name: ident,
            args,
            returns,
            body: Rc::new(Statement::Block(body)),
            span: Span {
                start: start_token.span.start,
                end: end_token.span.end,
            },
        })
    }

    fn parse_extern_function(&mut self) -> Option<Statement<'a>> {
        // extern
        let start_token = self.lexer.next().unwrap();

        let next = self.lexer.next().unwrap();

        let varargs = match next.kind {
            TokenKind::VarArgs => {
                self.lexer.next();
                true
            }
            _ => false,
        };

        // Parse name of the function
        let tok = self.lexer.next();
        let ident = match tok {
            Some(v) if matches!(v.kind, TokenKind::Identifier(_)) => {
                if let TokenKind::Identifier(val) = v.kind {
                    val
                } else {
                    "anon"
                }
            }
            _ => return None,
        };

        // Parse function arguments
        let mut args = vec![];
        if let Some(tok) = self.lexer.peek() {
            if tok.kind == TokenKind::BracketOpen {
                args = self.parse_type_list(TokenKind::BracketOpen);
            }
        }

        // Parse return type
        let mut returns = AzulaType::Void;
        if let Some(tok) = self.lexer.peek() {
            // If it's a colon, we have a type
            if tok.kind == TokenKind::Colon {
                self.lexer.next();

                returns = self.parse_type();
            }
        }
        if !self.expect_peek(TokenKind::SemiColon) {
            return None;
        }

        let end_token = self.lexer.next().unwrap();

        Some(Statement::ExternFunction {
            name: ident,
            varargs,
            args,
            returns,
            span: Span {
                start: start_token.span.start,
                end: end_token.span.end,
            },
        })
    }

    fn parse_return(&mut self) -> Option<Statement<'a>> {
        // return
        let start_token = self.lexer.next().unwrap();

        if let Some(v) = self.lexer.peek() {
            if v.kind == TokenKind::SemiColon {
                self.lexer.next();
                Some(Statement::Return(
                    None,
                    Span {
                        start: start_token.span.start,
                        end: start_token.span.end,
                    },
                ))
            } else {
                let expr = if let Some(expr) = self.parse_expression(LOWEST) {
                    expr
                } else {
                    return None;
                };
                if !self.expect_peek(TokenKind::SemiColon) {
                    return None;
                }

                let end_token = self.lexer.next().unwrap();
                Some(Statement::Return(
                    Some(expr.clone()),
                    Span {
                        start: start_token.span.start,
                        end: end_token.span.end,
                    },
                ))
            }
        } else {
            Some(Statement::Return(
                None,
                Span {
                    start: start_token.span.start,
                    end: start_token.span.end,
                },
            ))
        }
    }

    fn parse_assign(&mut self, mutable: bool) -> Option<Statement<'a>> {
        // var
        let start_token = self.lexer.next().unwrap();

        let tok = self.lexer.next();
        let ident = match tok {
            Some(v) if matches!(v.kind, TokenKind::Identifier(_)) => {
                if let TokenKind::Identifier(val) = v.kind {
                    val
                } else {
                    self.errors.push(AzulaError::new(
                        ErrorType::ExpectedToken(
                            format!("{:?}", TokenKind::Identifier("")),
                            Some(format!("{:?}", v.kind)),
                        ),
                        v.span.start,
                        v.span.end,
                    ));

                    return None;
                }
            }
            _ => return None,
        };

        let mut type_annotation = None;
        if self.lexer.peek().unwrap().kind == TokenKind::Colon {
            self.lexer.next();

            let typ = self.parse_type();
            type_annotation = Some(typ);
        }

        if !self.expect_peek(TokenKind::Assign) {
            return None;
        }

        self.lexer.next();

        let expr = if let Some(expr) = self.parse_expression(LOWEST) {
            expr
        } else {
            return None;
        };

        if !self.expect_peek(TokenKind::SemiColon) {
            return None;
        }

        let end_token = self.lexer.next().unwrap();

        Some(Statement::Assign(
            mutable,
            ident.to_string(),
            type_annotation,
            expr,
            Span {
                start: start_token.span.start,
                end: end_token.span.end,
            },
        ))
    }

    fn parse_reassign(&mut self, ident: ExpressionNode<'a>) -> Option<Statement<'a>> {
        self.lexer.next();

        let expr = if let Some(expr) = self.parse_expression(LOWEST) {
            expr
        } else {
            return None;
        };

        if !self.expect_peek(TokenKind::SemiColon) {
            return None;
        }

        let end_token = self.lexer.next().unwrap();

        Some(Statement::Reassign(
            ident.clone(),
            expr,
            Span {
                start: ident.span.start,
                end: end_token.span.end,
            },
        ))
    }

    fn parse_if(&mut self) -> Option<Statement<'a>> {
        // if
        let start_token = self.lexer.next().unwrap();

        let expr = self.parse_expression(LOWEST);

        if !self.expect_peek(TokenKind::BraceOpen) {
            return None;
        }

        self.lexer.next();

        let body = self.parse_block(TokenKind::BraceClose);

        let end_token = self.lexer.next().unwrap();

        Some(Statement::If(
            expr.unwrap(),
            body,
            Span {
                start: start_token.span.start,
                end: end_token.span.end,
            },
        ))
    }

    fn parse_while(&mut self) -> Option<Statement<'a>> {
        // while
        let start_token = self.lexer.next().unwrap();

        let expr = self.parse_expression(LOWEST);

        if !self.expect_peek(TokenKind::BraceOpen) {
            return None;
        }

        self.lexer.next();

        let body = self.parse_block(TokenKind::BraceClose);

        let end_token = self.lexer.next().unwrap();

        Some(Statement::While(
            expr.unwrap(),
            body,
            Span {
                start: start_token.span.start,
                end: end_token.span.end,
            },
        ))
    }

    fn parse_type(&mut self) -> AzulaType<'a> {
        if let Some(tok) = self.lexer.next() {
            if let TokenKind::Identifier(ident) = tok.kind {
                return ident.into();
            }

            if let TokenKind::Ampersand = tok.kind {
                return AzulaType::Pointer(Rc::new(self.parse_type()));
            }

            if let TokenKind::SquareOpen = tok.kind {
                let internal_type = self.parse_type();
                let mut size = None;
                if self.lexer.peek().unwrap().kind == TokenKind::SemiColon {
                    self.lexer.next();

                    if let TokenKind::Integer(i) = self.lexer.next().unwrap().kind {
                        size = Some(i as usize);
                    }
                }
                if !self.expect_peek(TokenKind::SquareClose) {
                    return AzulaType::Void;
                }

                self.lexer.next();

                return AzulaType::Array(Rc::new(internal_type), size);
            }

            // Bit of a hack to allow for double &&
            if let TokenKind::And = tok.kind {
                return AzulaType::Pointer(Rc::new(AzulaType::Pointer(Rc::new(self.parse_type()))));
            }
        }

        AzulaType::Void
    }

    fn parse_typed_identifier(&mut self) -> Option<TypedIdentifier<'a>> {
        let name = if let Some(tok) = self.lexer.next() {
            if let TokenKind::Identifier(ident) = tok.kind {
                ident
            } else {
                self.errors.push(AzulaError::new(
                    ErrorType::ExpectedToken(
                        format!("{:?}", TokenKind::Identifier("")),
                        Some(format!("{:?}", tok.kind)),
                    ),
                    tok.span.start,
                    tok.span.end,
                ));
                return None;
            }
        } else {
            self.errors.push(AzulaError::new(
                ErrorType::UnexpectedEOF,
                self.source.len() - 2,
                self.source.len() - 1,
            ));
            return None;
        };

        if !self.expect_peek(TokenKind::Colon) {
            return None;
        }

        self.lexer.next();

        let ident_type = self.parse_type();

        Some((ident_type, name))
    }

    fn parse_typed_identifier_list(
        &mut self,
        opening_delimiter: TokenKind,
    ) -> Vec<TypedIdentifier<'a>> {
        let closing_delimiter = opening_delimiter.get_closing_delimiter().unwrap();

        self.lexer.next();

        if let Some(peek) = self.lexer.peek() {
            if peek.kind == closing_delimiter {
                self.lexer.next();
                return vec![];
            }
        } else {
            self.errors.push(AzulaError::new(
                ErrorType::UnexpectedEOF,
                self.source.len() - 2,
                self.source.len() - 1,
            ));
            return vec![];
        }

        let mut identifiers = vec![];

        if let Some((typ, name)) = self.parse_typed_identifier() {
            identifiers.push((typ, name));
        }

        let mut peek = self.lexer.peek().unwrap().kind.clone();
        while peek == TokenKind::Comma {
            self.lexer.next();
            if let Some((typ, name)) = self.parse_typed_identifier() {
                identifiers.push((typ, name));
            }
            peek = self.lexer.peek().unwrap().kind.clone();
        }

        self.expect_peek(closing_delimiter);

        self.lexer.next();

        identifiers
    }

    fn parse_type_list(&mut self, opening_delimiter: TokenKind) -> Vec<AzulaType<'a>> {
        let closing_delimiter = opening_delimiter.get_closing_delimiter().unwrap();

        if let Some(peek) = self.lexer.peek() {
            if peek.kind == closing_delimiter {
                return vec![];
            }
        } else {
            self.errors.push(AzulaError::new(
                ErrorType::UnexpectedEOF,
                self.source.len() - 2,
                self.source.len() - 1,
            ));
            return vec![];
        }

        self.lexer.next();

        let mut types = vec![];

        types.push(self.parse_type());

        let mut peek = self.lexer.peek().unwrap().kind.clone();
        while peek == TokenKind::Comma {
            self.lexer.next();
            types.push(self.parse_type());
            peek = self.lexer.peek().unwrap().kind.clone();
        }

        self.expect_peek(closing_delimiter);

        self.lexer.next();

        types
    }

    fn expect_peek(&mut self, token_kind: TokenKind) -> bool {
        if let Some(tok) = self.lexer.peek() {
            return match tok.kind {
                _ if tok.kind == token_kind => true,
                _ => {
                    let span = tok.span.clone();
                    self.errors.push(AzulaError::new(
                        ErrorType::ExpectedToken(
                            format!("{:?}", token_kind),
                            Some(format!("{:?}", tok.kind)),
                        ),
                        span.start,
                        span.end,
                    ));
                    false
                }
            };
        }

        self.errors.push(AzulaError::new(
            ErrorType::UnexpectedEOF,
            self.source.len() - 1,
            self.source.len(),
        ));
        false
    }

    fn parse_expression(&mut self, precedence: OperatorPrecedence) -> Option<ExpressionNode<'a>> {
        let tok = if let Some(tok) = self.lexer.next() {
            tok
        } else {
            self.errors.push(AzulaError::new(
                ErrorType::UnexpectedEOF,
                self.source.len() - 2,
                self.source.len() - 1,
            ));
            return None;
        };

        let mut left = match tok.kind {
            TokenKind::Integer(i) => {
                if let Some(peek) = self.lexer.peek() {
                    if peek.kind == TokenKind::Dot {
                        self.lexer.next();
                        let second_number = self.parse_expression(CALL).unwrap();
                        match second_number.expression {
                            Expression::Integer(y) => Some(ExpressionNode {
                                expression: Expression::Float(
                                    format!("{}.{}", i, y).parse().unwrap(),
                                ),
                                typed: AzulaType::Float,
                                span: Span {
                                    start: tok.span.start,
                                    end: second_number.span.end,
                                },
                            }),
                            _ => panic!(),
                        }
                    } else {
                        Some(ExpressionNode {
                            expression: Expression::Integer(i),
                            typed: AzulaType::Int,
                            span: Span {
                                start: tok.span.start,
                                end: tok.span.end,
                            },
                        })
                    }
                } else {
                    Some(ExpressionNode {
                        expression: Expression::Integer(i),
                        typed: AzulaType::Int,
                        span: Span {
                            start: tok.span.start,
                            end: tok.span.end,
                        },
                    })
                }
            }
            TokenKind::True => Some(ExpressionNode {
                expression: Expression::Boolean(true),
                typed: AzulaType::Bool,
                span: Span {
                    start: tok.span.start,
                    end: tok.span.end,
                },
            }),
            TokenKind::False => Some(ExpressionNode {
                expression: Expression::Boolean(false),
                typed: AzulaType::Bool,
                span: Span {
                    start: tok.span.start,
                    end: tok.span.end,
                },
            }),
            TokenKind::String(val) => {
                let transformed = match string_transform(val) {
                    Ok(str) => str,
                    Err(index) => {
                        self.errors.push(AzulaError::new(
                            ErrorType::InvalidEscape,
                            tok.span.start + index,
                            tok.span.start + index + 1,
                        ));
                        "".to_string()
                    }
                };
                Some(ExpressionNode {
                    expression: Expression::String(transformed),
                    typed: AzulaType::Pointer(Rc::new(AzulaType::Str)),
                    span: Span {
                        start: tok.span.start,
                        end: tok.span.end,
                    },
                })
            }
            TokenKind::Identifier(x) => Some(ExpressionNode {
                expression: Expression::Identifier(x.to_string()),
                typed: AzulaType::Infer,
                span: Span {
                    start: tok.span.start,
                    end: tok.span.end,
                },
            }),
            TokenKind::BracketOpen => {
                let expr = self.parse_expression(LOWEST).unwrap();

                self.expect_peek(TokenKind::BracketClose);

                self.lexer.next();

                Some(expr)
            }
            TokenKind::Bang => {
                let expr = self.parse_expression(PREFIX).unwrap();

                Some(ExpressionNode {
                    expression: Expression::Not(Rc::new(expr.clone())),
                    typed: AzulaType::Bool,
                    span: Span {
                        start: tok.span.start,
                        end: expr.span.end,
                    },
                })
            }
            TokenKind::Ampersand => {
                let expr = self.parse_expression(PREFIX).unwrap();

                Some(ExpressionNode {
                    expression: Expression::Pointer(Rc::new(expr.clone())),
                    typed: AzulaType::Infer,
                    span: Span {
                        start: tok.span.start,
                        end: expr.span.end,
                    },
                })
            }
            TokenKind::SquareOpen => self.parse_array(tok),
            _ => {
                self.errors.push(AzulaError::new(
                    ErrorType::ExpectedExpression(format!("{:?}", tok.kind)),
                    tok.span.start,
                    tok.span.end,
                ));
                None
            }
        };

        let mut peek_token = &if let Some(tok) = self.lexer.peek() {
            tok
        } else {
            return left;
        }
        .kind;

        // Keep creating infix expressions until precedence is higher or semicolon found
        while peek_token.clone() != TokenKind::SemiColon
            && precedence < operator_precedence(peek_token.clone())
        {
            if left.is_none() {
                return None;
            }
            left = self.parse_infix(left.unwrap());
            peek_token = if let Some(v) = self.lexer.peek() {
                &v.kind
            } else {
                return left;
            };
        }

        left
    }

    fn parse_expression_list(&mut self, opening_delimiter: TokenKind) -> Vec<ExpressionNode<'a>> {
        let closing_delimiter = opening_delimiter.get_closing_delimiter().unwrap();

        self.lexer.next();

        if let Some(peek) = self.lexer.peek() {
            if peek.kind == closing_delimiter {
                return vec![];
            }
        } else {
            self.errors.push(AzulaError::new(
                ErrorType::UnexpectedEOF,
                self.source.len() - 2,
                self.source.len() - 1,
            ));
            return vec![];
        }

        let mut expressions = vec![];

        if let Some(expr) = self.parse_expression(LOWEST) {
            expressions.push(expr);
        }

        let mut peek = self.lexer.peek().unwrap().kind.clone();
        while peek == TokenKind::Comma {
            self.lexer.next();
            if let Some(expr) = self.parse_expression(LOWEST) {
                expressions.push(expr);
            }
            peek = self.lexer.peek().unwrap().kind.clone();
        }

        self.expect_peek(closing_delimiter);

        expressions
    }

    fn parse_infix(&mut self, left: ExpressionNode<'a>) -> Option<ExpressionNode<'a>> {
        let operator = self.lexer.peek().unwrap();
        let kind = operator.kind.clone();

        match kind {
            TokenKind::BracketOpen => self.parse_function_call(left),
            TokenKind::SquareOpen => self.parse_array_access(left),
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Modulo
            | TokenKind::Power
            | TokenKind::Or
            | TokenKind::And
            | TokenKind::Equal
            | TokenKind::NotEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => {
                let precedence = operator_precedence(operator.kind.clone());

                self.lexer.next();

                let right = if let Some(expr) = self.parse_expression(precedence) {
                    expr
                } else {
                    return None;
                };
                let operator = operator_from_token(kind).unwrap();
                return Some(ExpressionNode {
                    expression: Expression::Infix(
                        Rc::new(left.clone()),
                        operator,
                        Rc::new(right.clone()),
                    ),
                    typed: AzulaType::Infer,
                    span: Span {
                        start: left.span.start,
                        end: right.span.end,
                    },
                });
            }
            _ => {
                println!("unknown operator");
                return None;
            }
        }
    }

    fn parse_function_call(&mut self, left: ExpressionNode<'a>) -> Option<ExpressionNode<'a>> {
        let exprs = self.parse_expression_list(TokenKind::BracketOpen);

        let end_token = self.lexer.next().unwrap();

        Some(ExpressionNode {
            expression: Expression::FunctionCall {
                function: Rc::new(left.clone()),
                args: exprs,
            },
            typed: AzulaType::Infer,
            span: Span {
                start: left.span.start,
                end: end_token.span.end,
            },
        })
    }

    fn parse_array(&mut self, tok: Token) -> Option<ExpressionNode<'a>> {
        if let Some(peek) = self.lexer.peek() {
            if peek.kind == TokenKind::SquareClose {
                let close = self.lexer.next().unwrap();
                return Some(ExpressionNode {
                    expression: Expression::Array(vec![]),
                    typed: AzulaType::Array(Rc::new(AzulaType::Infer), Some(0)),
                    span: Span {
                        start: tok.span.start,
                        end: close.span.end,
                    },
                });
            }
        } else {
            self.errors.push(AzulaError::new(
                ErrorType::UnexpectedEOF,
                self.source.len() - 2,
                self.source.len() - 1,
            ));
            return Some(ExpressionNode {
                expression: Expression::Array(vec![]),
                typed: AzulaType::Array(Rc::new(AzulaType::Infer), Some(0)),
                span: Span {
                    start: tok.span.start,
                    end: 0,
                },
            });
        }

        let mut expressions = vec![];

        if let Some(expr) = self.parse_expression(LOWEST) {
            expressions.push(expr);
        }

        let mut peek = self.lexer.peek().unwrap().kind.clone();
        // Parse initialiser like [0; 20]
        if peek == TokenKind::SemiColon {
            self.lexer.next();

            if let TokenKind::Integer(v) = self.lexer.next().unwrap().kind {
                for _ in 0..v - 1 {
                    expressions.push(expressions[0].clone());
                }
                self.expect_peek(TokenKind::SquareClose);

                let close = self.lexer.next().unwrap();
                return Some(ExpressionNode {
                    expression: Expression::Array(expressions.clone()),
                    typed: AzulaType::Array(Rc::new(AzulaType::Infer), Some(expressions.len())),
                    span: Span {
                        start: tok.span.start,
                        end: close.span.end,
                    },
                });
            } else {
                self.expect_peek(TokenKind::SquareClose);

                let close = self.lexer.next().unwrap();
                self.errors.push(AzulaError::new(
                    ErrorType::ArrayInitialiserSizeNonConstant,
                    tok.span.start,
                    close.span.end,
                ));
                return None;
            }
        }
        while peek == TokenKind::Comma {
            self.lexer.next();
            if let Some(expr) = self.parse_expression(LOWEST) {
                expressions.push(expr);
            }
            peek = self.lexer.peek().unwrap().kind.clone();
        }

        self.expect_peek(TokenKind::SquareClose);

        let close = self.lexer.next().unwrap();

        Some(ExpressionNode {
            expression: Expression::Array(expressions.clone()),
            typed: AzulaType::Array(Rc::new(AzulaType::Infer), Some(expressions.len())),
            span: Span {
                start: tok.span.start,
                end: close.span.end,
            },
        })
    }

    fn parse_array_access(&mut self, left: ExpressionNode<'a>) -> Option<ExpressionNode<'a>> {
        self.lexer.next();
        let index = match self.parse_expression(LOWEST) {
            Some(expr) => expr,
            None => return None,
        };

        if !self.expect_peek(TokenKind::SquareClose) {
            return None;
        }

        let end_token = self.lexer.next().unwrap();

        Some(ExpressionNode {
            expression: Expression::ArrayAccess(Rc::new(left.clone()), Rc::new(index)),
            typed: AzulaType::Infer,
            span: Span {
                start: left.span.start,
                end: end_token.span.end,
            },
        })
    }
}

fn string_transform(str: &str) -> Result<String, usize> {
    let mut result = String::new();

    let mut chars = str.chars().enumerate().peekable();

    loop {
        let char = chars.next();
        if char.is_none() {
            break;
        }
        let (index, char) = char.unwrap();
        if char == '\\' {
            match chars.peek() {
                Some((_, '\\')) => {
                    chars.next();
                    result.push('\\');
                }
                Some((_, 'n')) => {
                    chars.next();
                    result.push('\n');
                }
                Some((_, 't')) => {
                    chars.next();
                    result.push('\t');
                }
                Some((_, 'r')) => {
                    chars.next();
                    result.push('\r');
                }
                Some((_, '0')) => {
                    chars.next();
                    result.push('\0');
                }
                Some((_, 'x')) => {
                    chars.next();
                    let (_, char1) = chars.next().unwrap();
                    let (_, char2) = chars.next().unwrap();
                    result.push(
                        char::from_u32(
                            i64::from_str_radix(&format!("{}{}", char1, char2), 16).unwrap() as u32,
                        )
                        .unwrap(),
                    );
                }
                _ => return Err(index + 1),
            }
        } else {
            result.push(char);
        }
    }

    Ok(result)
}

fn operator_from_token(tok: TokenKind) -> Option<Operator> {
    match tok {
        TokenKind::Plus => Some(Operator::Add),
        TokenKind::Minus => Some(Operator::Sub),
        TokenKind::Asterisk => Some(Operator::Mul),
        TokenKind::Slash => Some(Operator::Div),
        TokenKind::Modulo => Some(Operator::Mod),
        TokenKind::Power => Some(Operator::Power),
        TokenKind::Or => Some(Operator::Or),
        TokenKind::And => Some(Operator::And),
        TokenKind::Equal => Some(Operator::Eq),
        TokenKind::NotEqual => Some(Operator::Neq),
        TokenKind::Less => Some(Operator::Lt),
        TokenKind::LessEqual => Some(Operator::Lte),
        TokenKind::Greater => Some(Operator::Gt),
        TokenKind::GreaterEqual => Some(Operator::Gte),
        _ => None,
    }
}

fn operator_precedence(tok: TokenKind) -> OperatorPrecedence {
    match tok {
        TokenKind::Or | TokenKind::And => COMPARISON,
        TokenKind::Equal | TokenKind::NotEqual => EQUALS,
        TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => {
            LESS_GREATER
        }
        TokenKind::Plus | TokenKind::Minus => SUM,
        TokenKind::Slash | TokenKind::Asterisk | TokenKind::Power | TokenKind::Modulo => PRODUCT,
        TokenKind::BracketOpen | TokenKind::SquareOpen => CALL,
        _ => LOWEST,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parser_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let lexer: Lexer = input.into();
                    let mut parser = Parser::new(input, lexer);
                    if let Statement::Root(body) = parser.parse() {
                        for (i, statement) in body.into_iter().enumerate() {
                            assert_eq!(statement, expected[i])
                        }
                    };
                }
            )*
        }
    }

    parser_tests!(
        function: (
            "func test(x: bool): int { return 5; } func test2(x: int): int { return x; }",
            vec![Statement::Function {
                name: "test",
                args: vec![(AzulaType::Bool, "x")],
                returns: AzulaType::Int,
                body: Rc::new(Statement::Block(vec![Statement::Return(Some(ExpressionNode {
                    expression: Expression::Integer(5),
                    typed: AzulaType::Int,
                    span: Span { start: 33, end: 34},
                }), Span { start: 26, end: 35})])),
                span: Span { start: 0, end: 37},
            }, Statement::Function {
                name: "test2",
                args: vec![(AzulaType::Int, "x")],
                returns: AzulaType::Int,
                body: Rc::new(Statement::Block(vec![Statement::Return(Some(ExpressionNode {
                    expression: Expression::Identifier("x".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 71, end: 72},
                }), Span { start: 64, end: 73},)])),
                span: Span { start: 38, end: 75},
            }],
        ),
        return_int: (
            "return 5",
            vec![Statement::Return(Some(ExpressionNode {
                expression: Expression::Integer(5),
                typed: AzulaType::Int,
                span: Span { start: 7, end: 8},
            }), Span{start: 0, end: 8})],
        ),
        return_none: (
            "return;",
            vec![Statement::Return(None, Span{start: 0, end: 6})],
        ),
    );

    #[test]
    fn test_parse_function() {
        // No args, no return
        let input = "func main { }";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_function().unwrap();
        assert!(matches!(func, Statement::Function { .. }));
        if let Statement::Function {
            name,
            args,
            returns,
            body,
            ..
        } = func
        {
            assert_eq!(name, "main");
            assert_eq!(args, vec![]);
            assert_eq!(returns, AzulaType::Void);
            assert_eq!(body, Rc::new(Statement::Block(vec![])));
        }

        // No args, no return
        let input = "func main() { }";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_function();
        println!("{:?}", parser.errors);

        let func = func.unwrap();
        assert!(matches!(func, Statement::Function { .. }));
        if let Statement::Function {
            name,
            args,
            returns,
            body,
            ..
        } = func
        {
            assert_eq!(name, "main");
            assert_eq!(args, vec![]);
            assert_eq!(returns, AzulaType::Void);
            assert_eq!(body, Rc::new(Statement::Block(vec![])));
        }

        // Return type
        let input = "func test: bool { }";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_function().unwrap();
        assert!(matches!(func, Statement::Function { .. }));
        if let Statement::Function {
            name,
            args,
            returns,
            body,
            ..
        } = func
        {
            assert_eq!(name, "test");
            assert_eq!(args, vec![]);
            assert_eq!(returns, AzulaType::Bool);
            assert_eq!(body, Rc::new(Statement::Block(vec![])));
        }

        // Args & return type
        let input = "func test(x: int, y: bool): bool { }";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_function().unwrap();
        assert!(matches!(func, Statement::Function { .. }));
        if let Statement::Function {
            name,
            args,
            returns,
            body,
            ..
        } = func
        {
            assert_eq!(name, "test");
            assert_eq!(args, vec![(AzulaType::Int, "x"), (AzulaType::Bool, "y")]);
            assert_eq!(returns, AzulaType::Bool);
            assert_eq!(body, Rc::new(Statement::Block(vec![])));
        }
    }

    #[test]
    fn test_parse_extern_function() {
        let input = "extern varargs func test(int): bool;";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_statement().unwrap();
        println!("{:?}", func);
        assert!(matches!(func, Statement::ExternFunction { .. }));
        if let Statement::ExternFunction {
            name,
            varargs,
            args,
            returns,
            ..
        } = func
        {
            assert_eq!(name, "test");
            assert_eq!(varargs, true);
            assert_eq!(args, vec![AzulaType::Int]);
            assert_eq!(returns, AzulaType::Bool);
        }
    }

    #[test]
    fn test_parse_assign() {
        let input = "var test = 5;";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_statement().unwrap();
        assert!(matches!(func, Statement::Assign(..)));

        let input = "const test = 5;";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_statement().unwrap();
        assert!(matches!(func, Statement::Assign(..)));

        // Type annotation
        let input = "var test: int = 5;";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let assign = parser.parse_assign(true).unwrap();
        assert!(matches!(func, Statement::Assign(..)));
        if let Statement::Assign(mutable, name, type_annotation, value, ..) = assign {
            assert_eq!(mutable, true);
            assert_eq!(name, "test".to_string());
            assert_eq!(type_annotation, Some(AzulaType::Int));
            assert_eq!(
                value,
                ExpressionNode {
                    expression: Expression::Integer(5),
                    typed: AzulaType::Int,
                    span: Span { start: 16, end: 17 }
                }
            )
        }
    }

    #[test]
    fn test_parse_if() {
        let input = "if x { return 5; }";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_if().unwrap();
        assert!(matches!(func, Statement::If(..)));
        if let Statement::If(expr, body, ..) = func {
            assert_eq!(expr.expression, Expression::Identifier("x".to_string()));
            assert_eq!(
                body[0],
                Statement::Return(
                    Some(ExpressionNode {
                        expression: Expression::Integer(5),
                        typed: AzulaType::Int,
                        span: Span { start: 14, end: 15 }
                    }),
                    Span { start: 7, end: 16 }
                )
            );
        }

        let input = "if x || y { return 5; }";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let func = parser.parse_if().unwrap();
        assert!(matches!(func, Statement::If(..)));
        if let Statement::If(expr, body, ..) = func {
            assert_eq!(
                expr.expression,
                Expression::Infix(
                    Rc::new(ExpressionNode {
                        expression: Expression::Identifier("x".to_string()),
                        typed: AzulaType::Infer,
                        span: Span { start: 3, end: 4 },
                    }),
                    Operator::Or,
                    Rc::new(ExpressionNode {
                        expression: Expression::Identifier("y".to_string()),
                        typed: AzulaType::Infer,
                        span: Span { start: 8, end: 9 },
                    })
                )
            );
            assert_eq!(
                body[0],
                Statement::Return(
                    Some(ExpressionNode {
                        expression: Expression::Integer(5),
                        typed: AzulaType::Int,
                        span: Span { start: 19, end: 20 }
                    }),
                    Span { start: 12, end: 21 }
                )
            );
        }
    }

    #[test]
    fn test_parse_type() {
        // Basic type
        let input = "int";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let typ = parser.parse_type();
        assert_eq!(typ, AzulaType::Int);

        // Pointer
        let input = "&str";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let typ = parser.parse_type();
        assert_eq!(typ, AzulaType::Pointer(Rc::new(AzulaType::Str)));

        // Nested Pointers
        let input = "&&&str";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let typ = parser.parse_type();
        assert_eq!(
            typ,
            AzulaType::Pointer(Rc::new(AzulaType::Pointer(Rc::new(AzulaType::Pointer(
                Rc::new(AzulaType::Str)
            )))))
        );

        // Array
        let input = "[int]";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let typ = parser.parse_type();
        assert_eq!(typ, AzulaType::Array(Rc::new(AzulaType::Int), None));

        // Nested Array
        let input = "[[int]]";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let typ = parser.parse_type();
        assert_eq!(
            typ,
            AzulaType::Array(
                Rc::new(AzulaType::Array(Rc::new(AzulaType::Int), None)),
                None
            )
        );

        // Sized Array
        let input = "[int; 20]";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let typ = parser.parse_type();
        assert_eq!(typ, AzulaType::Array(Rc::new(AzulaType::Int), Some(20)));
    }

    #[test]
    fn test_parse_reassign() {
        // Basic int
        let input = "x = 5;";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let stmt = parser.parse_statement().unwrap();
        assert_eq!(
            stmt,
            Statement::Reassign(
                ExpressionNode {
                    expression: Expression::Identifier("x".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 1 }
                },
                ExpressionNode {
                    expression: Expression::Integer(5),
                    typed: AzulaType::Int,
                    span: Span { start: 4, end: 5 }
                },
                Span { start: 0, end: 6 },
            )
        );
    }

    #[test]
    fn test_parse_while() {
        // Basic int
        let input = "while x { i = 1; }";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let stmt = parser.parse_statement().unwrap();
        assert_eq!(
            stmt,
            Statement::While(
                ExpressionNode {
                    expression: Expression::Identifier("x".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 6, end: 7 }
                },
                vec![Statement::Reassign(
                    ExpressionNode {
                        expression: Expression::Identifier("i".to_string()),
                        typed: AzulaType::Infer,
                        span: Span { start: 10, end: 11 },
                    },
                    ExpressionNode {
                        expression: Expression::Integer(1),
                        typed: AzulaType::Int,
                        span: Span { start: 14, end: 15 },
                    },
                    Span { start: 10, end: 16 }
                )],
                Span { start: 0, end: 18 },
            )
        );
    }

    #[test]
    fn test_parse_array_access() {
        // Basic int
        let input = "x[10]";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expr = parser.parse_expression(LOWEST).unwrap();
        assert_eq!(
            expr,
            ExpressionNode {
                expression: Expression::ArrayAccess(
                    Rc::new(ExpressionNode {
                        expression: Expression::Identifier("x".to_string()),
                        typed: AzulaType::Infer,
                        span: Span { start: 0, end: 1 }
                    }),
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(10),
                        typed: AzulaType::Int,
                        span: Span { start: 2, end: 4 }
                    })
                ),
                typed: AzulaType::Infer,
                span: Span { start: 0, end: 5 }
            }
        );
    }

    #[test]
    fn test_parse_typed_identifier() {
        // Basic int
        let input = "x: int";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let (typ, name) = parser.parse_typed_identifier().unwrap();
        assert_eq!(typ, AzulaType::Int);
        assert_eq!(name, "x");

        // Custom struct type
        let input = "my_typed_identifier: CustomStruct";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let (typ, name) = parser.parse_typed_identifier().unwrap();
        assert_eq!(typ, AzulaType::Named("CustomStruct"));
        assert_eq!(name, "my_typed_identifier");

        // Non identifier for name
        let input = "4: CustomStruct";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        parser.parse_typed_identifier();
        assert_eq!(parser.errors.len(), 1);
        assert!(matches!(
            parser.errors[0].error_type,
            ErrorType::ExpectedToken(..)
        ));
        if let ErrorType::ExpectedToken(expected, ..) = &parser.errors[0].error_type {
            assert_eq!(expected.clone(), "Identifier(\"\")".to_string());
        }

        // No colon - EOF
        let input = "x";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        parser.parse_typed_identifier();
        assert_eq!(parser.errors.len(), 1);
        assert!(matches!(
            parser.errors[0].error_type,
            ErrorType::UnexpectedEOF,
        ));

        // Pointer
        let input = "x: &str";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let (typ, name) = parser.parse_typed_identifier().unwrap();
        assert_eq!(parser.errors.len(), 0);
        assert_eq!(typ, AzulaType::Pointer(Rc::new(AzulaType::Str)));
        assert_eq!(name, "x");
    }

    #[test]
    fn test_parse_typed_identifier_list() {
        let input = "(x: int, y: int, z: bool)";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let identifiers = parser.parse_typed_identifier_list(TokenKind::BracketOpen);

        let (typ, name) = &identifiers[0];
        assert_eq!(typ.clone(), AzulaType::Int);
        assert_eq!(name.clone(), "x");

        let (typ, name) = &identifiers[1];
        assert_eq!(typ.clone(), AzulaType::Int);
        assert_eq!(name.clone(), "y");

        let (typ, name) = &identifiers[2];
        assert_eq!(typ.clone(), AzulaType::Bool);
        assert_eq!(name.clone(), "z");
    }

    #[test]
    fn test_parse_expression_list() {
        let input = "(5, \"test\", x, true)";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let identifiers = parser.parse_expression_list(TokenKind::BracketOpen);

        println!("{:?}", parser.errors);

        let num = &identifiers[0];
        assert_eq!(num.typed.clone(), AzulaType::Int);
        matches!(num.expression, Expression::Integer(5));

        let num = &identifiers[1];
        assert_eq!(
            num.typed.clone(),
            AzulaType::Pointer(Rc::new(AzulaType::Str))
        );
        assert_eq!(num.expression, Expression::String("test".to_string()));

        let num = &identifiers[2];
        assert_eq!(num.typed.clone(), AzulaType::Infer);
        assert_eq!(num.expression, Expression::Identifier("x".to_string()));

        let num = &identifiers[3];
        assert_eq!(num.typed.clone(), AzulaType::Bool);
        matches!(num.expression, Expression::Boolean(true));
    }

    #[test]
    fn test_parse_integer_expression() {
        let input = "123";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert!(matches!(expression.expression, Expression::Integer(123)));
    }

    #[test]
    fn test_parse_boolean_expression() {
        let input = "true";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert!(matches!(expression.expression, Expression::Boolean(true)));

        let input = "false";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert!(matches!(expression.expression, Expression::Boolean(false)));
    }

    #[test]
    fn test_parse_infix_expression() {
        // Addition
        let input = "123 + 10";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap().expression;
        assert_eq!(
            expression,
            Expression::Infix(
                Rc::new(ExpressionNode {
                    expression: Expression::Integer(123),
                    typed: AzulaType::Int,
                    span: Span { start: 0, end: 3 },
                }),
                Operator::Add,
                Rc::new(ExpressionNode {
                    expression: Expression::Integer(10),
                    typed: AzulaType::Int,
                    span: Span { start: 6, end: 8 },
                })
            )
        );

        // Precedence
        let input = "123 * 3 + 10 * 5";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap().expression;
        assert_eq!(
            expression,
            Expression::Infix(
                Rc::new(ExpressionNode {
                    expression: Expression::Infix(
                        Rc::new(ExpressionNode {
                            expression: Expression::Integer(123),
                            typed: AzulaType::Int,
                            span: Span { start: 0, end: 3 },
                        }),
                        Operator::Mul,
                        Rc::new(ExpressionNode {
                            expression: Expression::Integer(3),
                            typed: AzulaType::Int,
                            span: Span { start: 6, end: 7 },
                        })
                    ),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 7 },
                }),
                Operator::Add,
                Rc::new(ExpressionNode {
                    expression: Expression::Infix(
                        Rc::new(ExpressionNode {
                            expression: Expression::Integer(10),
                            typed: AzulaType::Int,
                            span: Span { start: 10, end: 12 },
                        }),
                        Operator::Mul,
                        Rc::new(ExpressionNode {
                            expression: Expression::Integer(5),
                            typed: AzulaType::Int,
                            span: Span { start: 15, end: 16 },
                        })
                    ),
                    typed: AzulaType::Infer,
                    span: Span { start: 10, end: 16 },
                })
            )
        );

        // Brackets
        let input = "(123 + 3) * 10";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap().expression;
        assert_eq!(
            expression,
            Expression::Infix(
                Rc::new(ExpressionNode {
                    expression: Expression::Infix(
                        Rc::new(ExpressionNode {
                            expression: Expression::Integer(123),
                            typed: AzulaType::Int,
                            span: Span { start: 1, end: 4 },
                        }),
                        Operator::Add,
                        Rc::new(ExpressionNode {
                            expression: Expression::Integer(3),
                            typed: AzulaType::Int,
                            span: Span { start: 7, end: 8 },
                        })
                    ),
                    typed: AzulaType::Infer,
                    span: Span { start: 1, end: 8 },
                }),
                Operator::Mul,
                Rc::new(ExpressionNode {
                    expression: Expression::Integer(10),
                    typed: AzulaType::Int,
                    span: Span { start: 12, end: 14 },
                })
            )
        );

        // Comparison
        let input = "x || y";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap().expression;
        assert_eq!(
            expression,
            Expression::Infix(
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("x".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 1 },
                }),
                Operator::Or,
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("y".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 5, end: 6 },
                })
            )
        );
    }

    #[test]
    fn test_parse_array() {
        let input = "[1, 2, 3]";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert_eq!(
            expression.expression,
            Expression::Array(vec![
                ExpressionNode {
                    expression: Expression::Integer(1),
                    typed: AzulaType::Int,
                    span: Span { start: 1, end: 2 }
                },
                ExpressionNode {
                    expression: Expression::Integer(2),
                    typed: AzulaType::Int,
                    span: Span { start: 4, end: 5 }
                },
                ExpressionNode {
                    expression: Expression::Integer(3),
                    typed: AzulaType::Int,
                    span: Span { start: 7, end: 8 }
                }
            ])
        );

        // Empty array
        let input = "[]";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert_eq!(expression.expression, Expression::Array(vec![]));

        // Array initialiser
        let input = "[0; 5]";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert_eq!(
            expression.expression,
            Expression::Array(vec![
                ExpressionNode {
                    expression: Expression::Integer(0),
                    typed: AzulaType::Int,
                    span: Span { start: 1, end: 2 }
                },
                ExpressionNode {
                    expression: Expression::Integer(0),
                    typed: AzulaType::Int,
                    span: Span { start: 1, end: 2 }
                },
                ExpressionNode {
                    expression: Expression::Integer(0),
                    typed: AzulaType::Int,
                    span: Span { start: 1, end: 2 }
                },
                ExpressionNode {
                    expression: Expression::Integer(0),
                    typed: AzulaType::Int,
                    span: Span { start: 1, end: 2 }
                },
                ExpressionNode {
                    expression: Expression::Integer(0),
                    typed: AzulaType::Int,
                    span: Span { start: 1, end: 2 }
                }
            ])
        );
    }

    #[test]
    fn test_parse_function_call() {
        let input = "test()";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert_eq!(
            expression.expression,
            Expression::FunctionCall {
                function: Rc::new(ExpressionNode {
                    expression: Expression::Identifier("test".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 4 },
                }),
                args: vec![],
            }
        );

        let input = "test(5, \"test\")";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        println!("{:?}", parser.errors);
        assert!(parser.errors.is_empty());
        assert_eq!(
            expression.expression,
            Expression::FunctionCall {
                function: Rc::new(ExpressionNode {
                    expression: Expression::Identifier("test".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 4 },
                }),
                args: vec![
                    ExpressionNode {
                        expression: Expression::Integer(5),
                        typed: AzulaType::Int,
                        span: Span { start: 5, end: 6 }
                    },
                    ExpressionNode {
                        expression: Expression::String("test".to_string()),
                        typed: AzulaType::Pointer(Rc::new(AzulaType::Str)),
                        span: Span { start: 8, end: 14 }
                    }
                ],
            }
        );
    }

    #[test]
    fn test_string_transform() {
        let test = "\\\\ \\n test \\t";
        let result = string_transform(test).unwrap();
        assert_eq!(result, "\\ \n test \t");

        let test = "\\d";
        let result = string_transform(test);
        assert!(result.is_err());
    }

    #[test]
    fn test_not() {
        let input = "!true";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert!(parser.errors.is_empty());
        assert_eq!(
            expression.expression,
            Expression::Not(Rc::new(ExpressionNode {
                expression: Expression::Boolean(true),
                typed: AzulaType::Bool,
                span: Span { start: 1, end: 5 }
            }))
        );
    }

    #[test]
    fn test_pointer() {
        let input = "&my_var";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert!(parser.errors.is_empty());
        assert_eq!(
            expression.expression,
            Expression::Pointer(Rc::new(ExpressionNode {
                expression: Expression::Identifier("my_var".to_string()),
                typed: AzulaType::Infer,
                span: Span { start: 1, end: 7 }
            }))
        );
    }

    #[test]
    fn test_float() {
        let input = "5.5";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert!(parser.errors.is_empty());
        assert_eq!(expression.expression, Expression::Float(5.5));

        let input = "3.14523";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert!(parser.errors.is_empty());
        assert_eq!(expression.expression, Expression::Float(3.14523));

        let input = "5.2 * 15.2";
        let lexer: Lexer = input.into();
        let mut parser = Parser::new(input, lexer);

        let expression = parser.parse_expression(LOWEST).unwrap();
        assert!(parser.errors.is_empty());
        assert_eq!(
            expression.expression,
            Expression::Infix(
                Rc::new(ExpressionNode {
                    expression: Expression::Float(5.2),
                    typed: AzulaType::Float,
                    span: Span { start: 0, end: 3 }
                }),
                Operator::Mul,
                Rc::new(ExpressionNode {
                    expression: Expression::Float(15.2),
                    typed: AzulaType::Float,
                    span: Span { start: 6, end: 10 }
                })
            )
        );
    }
}
