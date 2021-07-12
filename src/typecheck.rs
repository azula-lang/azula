use std::collections::HashMap;

use crate::{
    errors::AzulaError,
    parser::ast::{Expr, Opcode, Statement, Type},
};

pub struct Typechecker {
    variables: HashMap<String, Type>,
    functions: Vec<String>,
    function_returns: HashMap<String, Type>,
    function_params: HashMap<String, Vec<Type>>,
    function_definitions: HashMap<String, (usize, usize)>,
}

impl Default for Typechecker {
    fn default() -> Self {
        Typechecker {
            variables: HashMap::new(),
            functions: vec![],
            function_params: HashMap::new(),
            function_returns: HashMap::new(),
            function_definitions: HashMap::new(),
        }
    }
}

impl Typechecker {
    pub fn typecheck(self: &mut Typechecker, statements: Vec<Statement>) -> Option<AzulaError> {
        for stmt in statements.clone() {
            match stmt {
                Statement::Function(name, parameters, return_type, _, l, r) => {
                    self.function_definitions.insert(name.clone(), (l, r));
                    self.functions.push(name.clone());
                    if let Some(return_type) = return_type {
                        self.function_returns.insert(name.clone(), return_type);
                    } else {
                        self.function_returns.insert(name.clone(), Type::Void);
                    }

                    if let Some(parameters) = parameters {
                        self.function_params
                            .insert(name, parameters.iter().map(|(t, _)| *t).collect());

                        for (typ, name) in parameters.iter() {
                            self.variables.insert(name.clone(), *typ);
                        }
                    }
                }
                _ => continue,
            }
        }
        for stmt in statements {
            let (_, er) = self.check(stmt);
            if er.is_some() {
                return er;
            }
        }

        None
    }

    fn check(self: &mut Typechecker, stmt: Statement) -> (Type, Option<AzulaError>) {
        match stmt {
            Statement::Let(_mutability, name, annotated_type, val, l, r) => {
                let (typ, er) = self.check_expr(&val);
                if er.is_some() {
                    return (Type::Void, er);
                }

                if let Some(annot) = annotated_type {
                    if annot != typ {
                        return (
                            Type::Void,
                            Some(AzulaError::VariableWrongType {
                                annotated: annot,
                                found: typ,
                                l,
                                r,
                            }),
                        );
                    }
                }

                self.variables.insert(name, typ);
                (Type::Void, None)
            }
            Statement::Function(_, _, _, stmts, _, _) => {
                let variables = self.variables.clone();

                for stmt in stmts {
                    let (_, er) = self.check(stmt);
                    if er.is_some() {
                        return (Type::Void, er);
                    }
                }

                self.variables = variables;

                (Type::Void, None)
            }
            Statement::Return(val, _, _) => {
                if let Some(val) = val {
                    let (typ, er) = self.check_expr(&val);
                    return (typ, er);
                }

                (Type::Void, None)
            }
            Statement::Expression(expr, _, _) => self.check_expr(&expr),
            Statement::If(cond, stmts, _, _) => {
                let (cond_typ, er) = self.check_expr(&cond);
                if er.is_some() {
                    return (Type::Void, er);
                }
                if cond_typ != Type::Boolean {
                    let (l, r) = get_pos(&cond);
                    return (
                        Type::Void,
                        Some(AzulaError::NonBooleanIfCond {
                            found: cond_typ,
                            l,
                            r,
                        }),
                    );
                }

                for stmt in stmts {
                    let (_, er) = self.check(stmt);
                    if er.is_some() {
                        return (Type::Void, er);
                    }
                }

                (Type::Void, None)
            }
        }
    }

    fn check_expr(self: &Typechecker, expr: &Expr) -> (Type, Option<AzulaError>) {
        match expr {
            Expr::Number(_, _, _) => (Type::Integer(32), None),
            Expr::Identifier(name, l, r) => {
                if !self.variables.contains_key(name) {
                    return (
                        Type::Void,
                        Some(AzulaError::VariableNotFound {
                            name: name.clone(),
                            l: *l,
                            r: *r,
                        }),
                    );
                }

                return (*self.variables.get(name).unwrap(), None);
            }
            Expr::Boolean(_, _, _) => (Type::Boolean, None),
            Expr::String(_, _, _) => (Type::String, None),
            Expr::Op(left, op, _, _, _) => match *op {
                Opcode::Eq
                | Opcode::NotEq
                | Opcode::GreaterThan
                | Opcode::GreaterEqual
                | Opcode::LessThan
                | Opcode::LessEqual => (Type::Boolean, None),
                _ => {
                    let (typ, er) = self.check_expr(left);
                    if er.is_some() {
                        return (Type::Void, er);
                    }

                    (typ, None)
                }
            },
            Expr::FunctionCall(name, params, l, r) => {
                if name.as_str() == "printf" {
                    return (Type::Void, None);
                }

                if !self.functions.contains(name) {
                    return (
                        Type::Void,
                        Some(AzulaError::FunctionNotFound {
                            name: name.clone(),
                            l: *l,
                            r: *r,
                        }),
                    );
                }

                let parameters = self.function_params.get(name).unwrap();
                for (i, expr) in params.iter().enumerate() {
                    let (typ, er) = self.check_expr(expr);
                    if er.is_some() {
                        return (Type::Void, er);
                    }

                    if typ != parameters[i] {
                        let (function_l, function_r) = self.function_definitions.get(name).unwrap();
                        let (l, r) = get_pos(expr);
                        return (
                            Type::Void,
                            Some(AzulaError::FunctionIncorrectParams {
                                expected: parameters[i],
                                found: typ,
                                function_l: *function_l,
                                function_r: *function_r,
                                l,
                                r,
                            }),
                        );
                    }
                }

                return (*self.function_returns.get(name).unwrap(), None);
            }
        }
    }
}

fn get_pos(exp: &Expr) -> (usize, usize) {
    match *exp {
        Expr::Number(_, l, r) => (l, r),
        Expr::Identifier(_, l, r) => (l, r),
        Expr::Boolean(_, l, r) => (l, r),
        Expr::String(_, l, r) => (l, r),
        Expr::Op(_, _, _, l, r) => (l, r),
        Expr::FunctionCall(_, _, l, r) => (l, r),
    }
}
