use std::collections::HashMap;

use crate::{
    errors::AzulaError,
    parser::ast::{Expr, Opcode, Statement, Type},
};

struct Function {
    returns: Type,
    params: Vec<Type>,
    defined: (usize, usize),
}

#[derive(Debug, Clone)]
struct Variable {
    typ: Type,
    mutable: bool,
    defined: (usize, usize),
}

struct Method {
    name: String,
    returns: Type,
    params: Vec<Type>,
    defined: (usize, usize),
}

pub struct Typechecker {
    variables: HashMap<String, Variable>,
    functions: HashMap<String, Function>,
    methods: HashMap<Type, Vec<Method>>,
}

impl Default for Typechecker {
    fn default() -> Self {
        Typechecker {
            variables: HashMap::new(),
            functions: HashMap::new(),
            methods: HashMap::new(),
        }
    }
}

impl Typechecker {
    pub fn typecheck(
        self: &mut Typechecker,
        statements: &mut Vec<Statement>,
    ) -> Option<AzulaError> {
        for stmt in statements.clone() {
            match stmt {
                Statement::Function(name, parameters, return_type, _, l, r) => {
                    let mut f = Function {
                        returns: Type::Void,
                        params: Vec::new(),
                        defined: (l, r),
                    };
                    if let Some(return_type) = return_type {
                        f.returns = return_type;
                    }

                    if let Some(parameters) = parameters {
                        f.params = parameters.iter().map(|(t, _)| t.clone()).collect();
                    }

                    self.functions.insert(name, f);
                }
                Statement::Impl(impl_type, methods, _, _) => {
                    self.methods.insert(impl_type.clone(), vec![]);
                    for m in methods {
                        match m {
                            Statement::Function(name, parameters, return_type, _, l, r) => {
                                let mut f = Method {
                                    name,
                                    returns: Type::Void,
                                    params: Vec::new(),
                                    defined: (l, r),
                                };
                                if let Some(return_type) = return_type {
                                    f.returns = return_type;
                                }

                                if let Some(parameters) = parameters {
                                    f.params = parameters.iter().map(|(t, _)| t.clone()).collect();
                                }

                                self.methods.get_mut(&impl_type).unwrap().push(f);
                            }
                            _ => continue,
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

    fn check(self: &mut Typechecker, stmt: &mut Statement) -> (Type, Option<AzulaError>) {
        match stmt {
            Statement::Let(mutability, name, annotated_type, val, l, r) => {
                let (typ, er) = self.check_expr(val);
                if er.is_some() {
                    return (Type::Void, er);
                }

                if let Some(annot) = annotated_type {
                    if *annot != typ {
                        return (
                            Type::Void,
                            Some(AzulaError::VariableWrongType {
                                annotated: annot.clone(),
                                found: typ,
                                l: *l,
                                r: *r,
                            }),
                        );
                    }
                }

                self.variables.insert(
                    name.clone(),
                    Variable {
                        typ: typ,
                        mutable: mutability.is_some(),
                        defined: (*l, *r),
                    },
                );
                (Type::Void, None)
            }
            Statement::Function(_, params, _, stmts, l, r) => {
                let variables = self.variables.clone();

                if let Some(params) = params {
                    for (typ, name) in params.iter() {
                        self.variables.insert(
                            name.clone(),
                            Variable {
                                typ: typ.clone(),
                                mutable: false,
                                defined: (*l, *r),
                            },
                        );
                    }
                }

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
                    let (typ, er) = self.check_expr(val);
                    return (typ, er);
                }

                (Type::Void, None)
            }
            Statement::Expression(expr, _, _) => self.check_expr(expr),
            Statement::If(cond, stmts, _, _) => {
                let (cond_typ, er) = self.check_expr(cond);
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
            Statement::Macro(_, _, _, _, _, _) => panic!(),
            Statement::Reassign(name, val, l, r) => {
                let (typ, er) = self.check_expr(val);
                if er.is_some() {
                    return (Type::Void, er);
                }

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

                let var = self.variables.get(name).unwrap();
                let (variable_l, variable_r) = var.defined;
                if !var.mutable {
                    return (
                        Type::Void,
                        Some(AzulaError::VariableNotMutable {
                            name: name.clone(),
                            variable_l,
                            variable_r,
                            l: *l,
                            r: *r,
                        }),
                    );
                }

                let annotated = var.typ.clone();
                if annotated != typ {
                    return (
                        Type::Void,
                        Some(AzulaError::VariableReassignWrongType {
                            annotated,
                            found: typ,
                            l: *l,
                            r: *r,
                            variable_l,
                            variable_r,
                        }),
                    );
                }

                (Type::Void, er)
            }
            Statement::Impl(_, funcs, _, _) => {
                for f in funcs {
                    match f {
                        Statement::Function(_, params, _, stmts, l, r) => {
                            let variables = self.variables.clone();

                            if let Some(params) = params {
                                for (typ, name) in params.iter() {
                                    self.variables.insert(
                                        name.clone(),
                                        Variable {
                                            typ: typ.clone(),
                                            mutable: false,
                                            defined: (*l, *r),
                                        },
                                    );
                                }
                            }

                            for stmt in stmts {
                                let (_, er) = self.check(stmt);
                                if er.is_some() {
                                    return (Type::Void, er);
                                }
                            }

                            self.variables = variables;
                        }
                        _ => continue,
                    }
                }
                (Type::Void, None)
            }
        }
    }

    fn check_expr(self: &Typechecker, expr: &mut Expr) -> (Type, Option<AzulaError>) {
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

                return (self.variables.get(name).unwrap().typ.clone(), None);
            }
            Expr::Boolean(_, _, _) => (Type::Boolean, None),
            Expr::String(_, _, _) => (Type::String, None),
            Expr::Op(left, op, right, _, _) => match *op {
                Opcode::Eq
                | Opcode::NotEq
                | Opcode::GreaterThan
                | Opcode::GreaterEqual
                | Opcode::LessThan
                | Opcode::LessEqual => {
                    let (_, er) = self.check_expr(left);
                    if er.is_some() {
                        return (Type::Void, er);
                    }

                    let (_, er) = self.check_expr(right);
                    if er.is_some() {
                        return (Type::Void, er);
                    }

                    (Type::Boolean, None)
                }
                _ => {
                    let (typ, er) = self.check_expr(left);
                    if er.is_some() {
                        return (Type::Void, er);
                    }

                    let (_, er) = self.check_expr(right);
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

                if !self.functions.contains_key(name) {
                    return (
                        Type::Void,
                        Some(AzulaError::FunctionNotFound {
                            name: name.clone(),
                            l: *l,
                            r: *r,
                        }),
                    );
                }

                if self.functions.contains_key(name) {
                    let f = self.functions.get(name).unwrap();
                    for (i, expr) in params.iter_mut().enumerate() {
                        let (typ, er) = self.check_expr(expr);
                        if er.is_some() {
                            return (Type::Void, er);
                        }

                        if typ != f.params[i] {
                            let (function_l, function_r) = f.defined;
                            let (l, r) = get_pos(expr);
                            return (
                                Type::Void,
                                Some(AzulaError::FunctionIncorrectParams {
                                    expected: f.params[i].clone(),
                                    found: typ,
                                    function_l: function_l,
                                    function_r: function_r,
                                    l,
                                    r,
                                }),
                            );
                        }
                    }
                }

                return (self.functions.get(name).unwrap().returns.clone(), None);
            }
            Expr::ArrayLiteral(vals, l, r) => {
                let mut array_typ = None;
                for expr in vals {
                    let (typ, er) = self.check_expr(expr);
                    if er.is_some() {
                        return (Type::Void, er);
                    }

                    if let Some(array_type) = array_typ {
                        if array_type != typ {
                            return (
                                Type::Void,
                                Some(AzulaError::ArrayDifferentTypes {
                                    array_type,
                                    found: typ,
                                    l: *l,
                                    r: *r,
                                }),
                            );
                        }
                    }
                    array_typ = Some(typ);
                }

                (Type::Array(Box::new(array_typ.unwrap())), None)
            }
            Expr::ArrayIndex(arr, _, l, r) => {
                let (arr_type, er) = self.check_expr(arr);
                if er.is_some() {
                    return (Type::Void, er);
                }

                match arr_type {
                    Type::Array(x) => (*x, None),
                    _ => (Type::Void, Some(AzulaError::NonArrayIndex { l: *l, r: *r })),
                }
            }
            Expr::MethodCall(expr, name, params, ref mut type_data, l, r) => {
                let (left_type, er) = self.check_expr(expr);
                if er.is_some() {
                    return (Type::Void, er);
                }

                *type_data = Some(left_type.clone());

                if !self.methods.contains_key(&left_type.clone()) {
                    return (
                        Type::Void,
                        Some(AzulaError::MethodNotFound {
                            impl_type: left_type,
                            method_name: name.clone(),
                            l: *l,
                            r: *r,
                        }),
                    );
                }

                let methods = self.methods.get(&left_type).unwrap();
                let method = methods.into_iter().find(|x| x.name == name.clone());
                if method.is_none() {
                    return (
                        Type::Void,
                        Some(AzulaError::MethodNotFound {
                            impl_type: left_type,
                            method_name: name.clone(),
                            l: *l,
                            r: *r,
                        }),
                    );
                }

                let m = method.unwrap();
                for (i, expr) in params.iter_mut().enumerate() {
                    let (typ, er) = self.check_expr(expr);
                    if er.is_some() {
                        return (Type::Void, er);
                    }

                    if typ != m.params[i] {
                        let (function_l, function_r) = m.defined;
                        let (l, r) = get_pos(expr);
                        return (
                            Type::Void,
                            Some(AzulaError::FunctionIncorrectParams {
                                expected: m.params[i].clone(),
                                found: typ,
                                function_l: function_l,
                                function_r: function_r,
                                l,
                                r,
                            }),
                        );
                    }
                }

                return (m.returns.clone(), None);
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
        Expr::ArrayLiteral(_, l, r) => (l, r),
        Expr::ArrayIndex(_, _, l, r) => (l, r),
        Expr::MethodCall(_, _, _, _, l, r) => (l, r),
    }
}
