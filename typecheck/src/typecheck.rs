use std::{collections::HashMap, ops::Deref, rc::Rc};

use azula_ast::prelude::*;
use azula_error::prelude::*;
use azula_type::prelude::AzulaType;

pub struct Typechecker<'a> {
    ast: Statement<'a>,

    functions: HashMap<&'a str, FunctionDefinition<'a>>,
    globals: HashMap<String, VariableDefinition<'a>>,
    structs: HashMap<String, StructDefinition<'a>>,
    namespaces: HashMap<String, Namespace<'a>>,
    enums: HashMap<String, Vec<String>>,
    type_aliases: HashMap<String, AzulaType<'a>>,

    pub errors: Vec<AzulaError>,
}

#[derive(Clone, Debug)]
struct FunctionDefinition<'a> {
    name: String,
    args: Vec<(AzulaType<'a>, String)>,
    varargs: bool,
    returns: AzulaType<'a>,
}

struct StructDefinition<'a> {
    name: &'a str,
    attrs: Vec<(AzulaType<'a>, &'a str)>,
}

#[derive(Debug, Clone)]
pub struct VariableDefinition<'a> {
    name: String,
    mutable: bool,
    typ: AzulaType<'a>,
}

#[derive(Debug)]
pub struct Namespace<'a> {
    name: String,
    funcs: HashMap<&'a str, FunctionDefinition<'a>>,
}

#[derive(Clone)]
pub struct Environment<'a> {
    variable_definitions: HashMap<String, VariableDefinition<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            variable_definitions: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, name: String, def: VariableDefinition<'a>) {
        self.variable_definitions.insert(name, def);
    }
}

impl<'a> Typechecker<'a> {
    pub fn new(root: Statement<'a>) -> Self {
        Typechecker {
            ast: root,
            functions: HashMap::new(),
            globals: HashMap::new(),
            structs: HashMap::new(),
            namespaces: HashMap::new(),
            enums: HashMap::new(),
            type_aliases: HashMap::new(),
            errors: vec![],
        }
    }

    pub fn typecheck(&mut self) -> Result<Statement<'a>, String> {
        if let Statement::Root(mut x) = self.ast.clone() {
            for stmt in x.iter_mut() {
                match stmt {
                    Statement::Function {
                        name,
                        args,
                        returns,
                        ..
                    } => {
                        let args_converted: Vec<_> = args
                            .iter()
                            .map(|(typ, name)| (AzulaType::from(typ.clone()), name.to_string()))
                            .collect();

                        let returns_converted: AzulaType = returns.clone().into();

                        self.functions.insert(
                            name,
                            FunctionDefinition {
                                name: name.to_string(),
                                varargs: true,
                                args: args_converted.clone(),
                                returns: returns_converted.clone(),
                            },
                        );
                    }
                    Statement::ExternFunction {
                        name,
                        varargs,
                        args,
                        returns,
                        ..
                    } => {
                        let args_converted: Vec<_> = args
                            .iter()
                            .map(|typ| (typ.clone(), "xyz".to_string()))
                            .collect();

                        let returns_converted: AzulaType = returns.clone().into();

                        self.functions.insert(
                            name,
                            FunctionDefinition {
                                name: name.to_string(),
                                varargs: false,
                                args: args_converted.clone(),
                                returns: returns_converted.clone(),
                            },
                        );
                    }
                    Statement::Impl {
                        struct_impl,
                        trait_impl,
                        funcs,
                        span,
                    } => {
                        let mut new_funcs = HashMap::new();
                        for func in funcs {
                            if let Statement::Function {
                                name,
                                args,
                                returns,
                                body,
                                span,
                            } = func
                            {
                                let args_converted: Vec<_> = args
                                    .iter()
                                    .map(|(typ, name)| {
                                        (AzulaType::from(typ.clone()), name.to_string())
                                    })
                                    .collect();

                                let returns_converted: AzulaType = returns.clone().into();

                                new_funcs.insert(
                                    name.clone(),
                                    FunctionDefinition {
                                        name: name.to_string(),
                                        varargs: true,
                                        args: args_converted.clone(),
                                        returns: returns_converted.clone(),
                                    },
                                );
                            }
                        }

                        let struc_name = struct_impl.to_string();

                        let namespace = Namespace {
                            name: struc_name.clone(),
                            funcs: new_funcs,
                        };

                        self.namespaces.insert(struc_name, namespace);
                    }
                    Statement::Enum { name, variants, .. } => {
                        self.enums.insert(
                            name.to_string(),
                            variants.iter().map(|v| v.to_string()).collect(),
                        );
                    }
                    Statement::TypeAlias { name, typ, .. } => {
                        self.type_aliases.insert(name.to_string(), typ.clone());
                    }
                    _ => {}
                }
            }
        } else {
            return Err("Not a root node".to_string());
        }

        if let Statement::Root(mut x) = self.ast.clone() {
            for stmt in x.iter_mut() {
                *stmt = match self.typecheck_top_level_statement(stmt.clone()) {
                    Ok(stmt) => stmt,
                    Err(e) => return Err(e),
                };
            }
            Ok(Statement::Root(x))
        } else {
            Err("Not a root node".to_string())
        }
    }

    pub fn typecheck_top_level_statement(
        &mut self,
        stmt: Statement<'a>,
    ) -> Result<Statement<'a>, String> {
        match stmt {
            Statement::Function { .. } => self.typecheck_function(stmt),
            Statement::ExternFunction { .. } => Ok(stmt),
            Statement::Assign(..) => self.typecheck_global_assign(stmt),
            Statement::Struct {
                name,
                attributes,
                span,
            } => {
                self.structs.insert(
                    name.to_string(),
                    StructDefinition {
                        name,
                        attrs: attributes.clone(),
                    },
                );

                Ok(Statement::Struct {
                    name: name,
                    attributes: attributes,
                    span: span,
                })
            }
            Statement::Impl {
                struct_impl,
                trait_impl,
                funcs,
                span,
            } => {
                let mut new_funcs = vec![];
                for func in funcs {
                    new_funcs.push(self.typecheck_function(func).unwrap());
                }

                Ok(Statement::Impl {
                    struct_impl: struct_impl,
                    trait_impl: trait_impl,
                    funcs: new_funcs,
                    span: span,
                })
            }
            Statement::Enum { .. } => Ok(stmt),
            Statement::TypeAlias { .. } => Ok(stmt),
            Statement::Import(..) => Ok(stmt),
            _ => unreachable!(),
        }
    }

    pub fn typecheck_statement(
        &mut self,
        stmt: Statement<'a>,
        env: &mut Environment<'a>,
    ) -> Result<(Statement<'a>, AzulaType<'a>), String> {
        match stmt {
            Statement::Assign(..) => self.typecheck_assign(stmt, env),
            Statement::Return(..) => self.typecheck_return(stmt, env),
            Statement::ExpressionStatement(expr, span) => {
                let result = match self.typecheck_expression(expr, env) {
                    Ok((expr, _)) => expr,
                    Err(e) => return Err(e),
                };
                Ok((
                    Statement::ExpressionStatement(result, span),
                    AzulaType::Void,
                ))
            }
            Statement::If(..) => self.typecheck_if(stmt, env),
            Statement::While(..) => self.typecheck_while(stmt, env),
            Statement::For(..) => self.typecheck_for(stmt, env),
            Statement::Break(span) => Ok((Statement::Break(span), AzulaType::Void)),
            Statement::Continue(span) => Ok((Statement::Continue(span), AzulaType::Void)),
            Statement::Reassign(..) => self.typecheck_reassign(stmt, env),
            Statement::Block(stmts) => {
                let mut checked = vec![];
                for s in stmts {
                    match self.typecheck_statement(s, env) {
                        Ok((s, _)) => checked.push(s),
                        Err(e) => return Err(e),
                    }
                }
                Ok((Statement::Block(checked), AzulaType::Void))
            }
            _ => unreachable!("{:?}", stmt),
        }
    }

    fn typecheck_function(&mut self, stmt: Statement<'a>) -> Result<Statement<'a>, String> {
        if let Statement::Function {
            name,
            args,
            returns,
            body,
            span,
        } = stmt.clone()
        {
            let args_converted: Vec<_> = args
                .iter()
                .map(|(typ, name)| (AzulaType::from(typ.clone()), *name))
                .collect();

            let mut environment = Environment::new();
            for (typ, name) in &args_converted {
                environment.add_variable(
                    name.to_string(),
                    VariableDefinition {
                        name: name.to_string(),
                        mutable: false,
                        typ: typ.clone(),
                    },
                )
            }
            let mut statements = vec![];
            if let Statement::Block(mut stmts) = body.deref().clone() {
                for stmt in stmts.iter_mut() {
                    statements.push(
                        match self.typecheck_statement(stmt.clone(), &mut environment) {
                            Ok((stmt, _)) => stmt,
                            Err(e) => return Err(e),
                        },
                    );
                }
            }

            return Ok(Statement::Function {
                name,
                args,
                returns,
                body: Rc::new(Statement::Block(statements)),
                span,
            });
        }

        unreachable!()
    }

    fn typecheck_global_assign(&mut self, expr: Statement<'a>) -> Result<Statement<'a>, String> {
        if let Statement::Assign(mutable, name, type_annotation, value, span) = expr {
            if mutable {
                self.errors.push(AzulaError::new(
                    ErrorType::NonGlobalConstant,
                    span.start,
                    span.end,
                ));
                return Err("Non constant at top-level".to_string());
            }

            // let (expr, typ) = match self.typecheck_expression(value, &Environment::new()) {
            //     Ok((expr, value)) => (expr, value),
            //     Err(e) => return Err(e),
            // };

            let typ = match value.expression.clone() {
                Expression::Integer(_) => AzulaType::Int,
                Expression::Float(_) => AzulaType::Float,
                Expression::Boolean(_) => AzulaType::Bool,
                Expression::String(_) => AzulaType::Pointer(Rc::new(AzulaType::Str)),
                Expression::Array(val) => {
                    AzulaType::Array(Rc::new(val[0].typed.clone()), Some(val.len()))
                }
                _ => {
                    self.errors.push(AzulaError::new(
                        ErrorType::NonGlobalConstant,
                        span.start,
                        span.end,
                    ));
                    return Err("Non constant at top-level".to_string());
                }
            };

            if type_annotation.is_some() {
                let type_annotation = self.resolve_type(type_annotation.clone().unwrap());

                if type_annotation != typ {
                    self.errors.push(AzulaError::new(
                        ErrorType::MismatchedAssignTypes(
                            format!("{:?}", type_annotation),
                            format!("{:?}", typ),
                        ),
                        span.start,
                        value.span.end,
                    ));
                    return Err("mismatched types in assign".to_string());
                }
            }

            self.globals.insert(
                name.clone(),
                VariableDefinition {
                    name: name.clone(),
                    mutable,
                    typ,
                },
            );

            Ok(Statement::Assign(
                mutable,
                name,
                type_annotation,
                value,
                span,
            ))
        } else {
            unreachable!()
        }
    }

    fn typecheck_assign(
        &mut self,
        expr: Statement<'a>,
        env: &mut Environment<'a>,
    ) -> Result<(Statement<'a>, AzulaType<'a>), String> {
        if let Statement::Assign(mutable, name, type_annotation, value, span) = expr {
            let type_annotation = type_annotation.map(|t| self.resolve_type(t));
            let (expr, typ) = match self.typecheck_expression(value, env) {
                Ok((expr, value)) => (expr, value),
                Err(e) => return Err(e),
            };

            if type_annotation.is_some() {
                let mut type_annotation = type_annotation.clone().unwrap();

                if let AzulaType::Array(arr_typ, size) = typ.clone() {
                    if let AzulaType::Array(inner_type, inner_size) = type_annotation.clone() {
                        if arr_typ != inner_type {
                            self.errors.push(AzulaError::new(
                                ErrorType::MismatchedAssignTypes(
                                    format!("{:?}", type_annotation),
                                    format!("{:?}", typ),
                                ),
                                span.start,
                                expr.span.end,
                            ));
                            return Err("mismatched types in assign".to_string());
                        }

                        if size.is_some() && inner_size.is_some() {
                            if size.unwrap() != inner_size.unwrap() {
                                self.errors.push(AzulaError::new(
                                    ErrorType::MismatchedAssignTypes(
                                        format!("{:?}", type_annotation),
                                        format!("{:?}", typ),
                                    ),
                                    span.start,
                                    expr.span.end,
                                ));
                                return Err("mismatched types in assign".to_string());
                            }
                        }

                        type_annotation = AzulaType::Array(arr_typ, size);
                    }
                }

                let types_compatible = type_annotation == typ
                    || (matches!(typ, AzulaType::Int) && matches!(type_annotation, AzulaType::SizedSignedInt(_)))
                    || (matches!(typ, AzulaType::SizedSignedInt(_)) && matches!(type_annotation, AzulaType::Int))
                    || (matches!(typ, AzulaType::Str) && matches!(type_annotation, AzulaType::Pointer(_)))
                    || (matches!(typ, AzulaType::Pointer(_)) && matches!(type_annotation, AzulaType::Str));

                if !types_compatible {
                    self.errors.push(AzulaError::new(
                        ErrorType::MismatchedAssignTypes(
                            format!("{:?}", type_annotation),
                            format!("{:?}", typ),
                        ),
                        span.start,
                        expr.span.end,
                    ));
                    return Err("mismatched types in assign".to_string());
                }
            }

            let resolved_typ = if let Some(ann) = type_annotation.clone() { ann } else { typ };

            env.add_variable(
                name.clone(),
                VariableDefinition {
                    name: name.clone(),
                    mutable,
                    typ: resolved_typ.clone(),
                },
            );

            Ok((
                Statement::Assign(mutable, name, type_annotation, expr, span),
                AzulaType::Void,
            ))
        } else {
            unreachable!()
        }
    }

    fn typecheck_reassign(
        &mut self,
        expr: Statement<'a>,
        env: &mut Environment<'a>,
    ) -> Result<(Statement<'a>, AzulaType<'a>), String> {
        if let Statement::Reassign(var, val, span) = expr {
            let (val, typ) = match self.typecheck_expression(val, env) {
                Ok((expr, value)) => (expr, value),
                Err(e) => return Err(e),
            };

            let mut mutable = true;
            match var.expression {
                Expression::Identifier(ref v) => match env.variable_definitions.get(v) {
                    Some(var) => {
                        mutable = var.mutable;
                    }
                    _ => {
                        self.errors.push(AzulaError::new(
                            ErrorType::UnknownVariable(v.clone()),
                            var.span.start,
                            var.span.end,
                        ));
                        return Err(format!("unknown variable {:?}", var.expression));
                    }
                },
                Expression::ArrayAccess(..) => {}
                Expression::StructAccess(..) => {}
                _ => {
                    unreachable!("{:?}", var.expression)
                }
            }

            if !mutable {
                self.errors.push(AzulaError::new(
                    ErrorType::ConstantAssign,
                    var.span.start,
                    var.span.end,
                ));
                return Err("constant assign".to_string());
            }

            let (variable, var_type) = match self.typecheck_expression(var, env) {
                Ok((expr, value)) => (expr, value),
                Err(e) => return Err(e),
            };

            if var_type != typ {
                self.errors.push(AzulaError::new(
                    ErrorType::MismatchedAssignTypes(
                        format!("{:?}", var_type),
                        format!("{:?}", typ),
                    ),
                    span.start,
                    val.span.end,
                ));
                return Err("mismatched types in assign".to_string());
            }

            Ok((Statement::Reassign(variable, val, span), AzulaType::Void))
        } else {
            unreachable!()
        }
    }

    fn typecheck_return(
        &mut self,
        expr: Statement<'a>,
        env: &mut Environment<'a>,
    ) -> Result<(Statement<'a>, AzulaType<'a>), String> {
        if let Statement::Return(ref value, ref span) = expr {
            if value.is_none() {
                return Ok((expr.clone(), AzulaType::Void));
            }
            let value = value.as_ref().unwrap();
            let (expr, typ) = match self.typecheck_expression(value.clone(), env) {
                Ok((expr, value)) => (expr, value),
                Err(e) => return Err(e),
            };

            Ok((Statement::Return(Some(expr.clone()), span.clone()), typ))
        } else {
            unreachable!()
        }
    }

    fn typecheck_if(
        &mut self,
        stmt: Statement<'a>,
        env: &mut Environment<'a>,
    ) -> Result<(Statement<'a>, AzulaType<'a>), String> {
        if let Statement::If(ref expr, ref body, ref else_branch, ref span) = stmt {
            let (expr, typ) = match self.typecheck_expression(expr.clone(), env) {
                Ok((expr, value)) => (expr, value),
                Err(e) => return Err(e),
            };

            if typ != AzulaType::Bool {
                self.errors.push(AzulaError::new(
                    ErrorType::NonBoolCondition(format!("{:?}", typ)),
                    expr.span.start,
                    expr.span.end,
                ));
                return Err("Non boolean condition".to_string());
            }

            let mut stmts = vec![];
            for stmt in body {
                match self.typecheck_statement(stmt.clone(), env) {
                    Ok((stmt, _)) => stmts.push(stmt),
                    Err(e) => return Err(e),
                };
            }

            let checked_else = match else_branch {
                Some(else_stmt) => {
                    match self.typecheck_statement(else_stmt.as_ref().clone(), env) {
                        Ok((stmt, _)) => Some(Rc::new(stmt)),
                        Err(e) => return Err(e),
                    }
                }
                None => None,
            };

            Ok((Statement::If(expr, stmts, checked_else, span.clone()), AzulaType::Void))
        } else {
            unreachable!()
        }
    }

    fn typecheck_while(
        &mut self,
        stmt: Statement<'a>,
        env: &mut Environment<'a>,
    ) -> Result<(Statement<'a>, AzulaType<'a>), String> {
        if let Statement::While(ref expr, ref body, ref span) = stmt {
            let (expr, typ) = match self.typecheck_expression(expr.clone(), env) {
                Ok((expr, value)) => (expr, value),
                Err(e) => return Err(e),
            };

            if typ != AzulaType::Bool {
                self.errors.push(AzulaError::new(
                    ErrorType::NonBoolCondition(format!("{:?}", typ)),
                    expr.span.start,
                    expr.span.end,
                ));
                return Err("Non boolean condition".to_string());
            }

            let mut stmts = vec![];
            for stmt in body {
                match self.typecheck_statement(stmt.clone(), env) {
                    Ok((stmt, _)) => stmts.push(stmt),
                    Err(e) => return Err(e),
                };
            }

            Ok((Statement::While(expr, stmts, span.clone()), AzulaType::Void))
        } else {
            unreachable!()
        }
    }

    fn typecheck_for(
        &mut self,
        stmt: Statement<'a>,
        env: &mut Environment<'a>,
    ) -> Result<(Statement<'a>, AzulaType<'a>), String> {
        if let Statement::For(ref cond, ref body, ref span) = stmt {
            let checked_cond = match cond {
                Some(expr) => {
                    let (checked, typ) = match self.typecheck_expression(expr.clone(), env) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };
                    if typ != AzulaType::Bool {
                        self.errors.push(AzulaError::new(
                            ErrorType::NonBoolCondition(format!("{:?}", typ)),
                            checked.span.start,
                            checked.span.end,
                        ));
                        return Err("Non boolean condition".to_string());
                    }
                    Some(checked)
                }
                None => None,
            };

            let mut stmts = vec![];
            for stmt in body {
                match self.typecheck_statement(stmt.clone(), env) {
                    Ok((stmt, _)) => stmts.push(stmt),
                    Err(e) => return Err(e),
                };
            }

            Ok((Statement::For(checked_cond, stmts, span.clone()), AzulaType::Void))
        } else {
            unreachable!()
        }
    }

    fn typecheck_expression(
        &mut self,
        mut expr: ExpressionNode<'a>,
        env: &Environment<'a>,
    ) -> Result<(ExpressionNode<'a>, AzulaType<'a>), String> {
        match expr.expression {
            Expression::Infix(..) => self.typecheck_infix_expression(expr, env),
            Expression::Integer(_) => {
                expr.typed = AzulaType::Int;
                Ok((expr.clone(), AzulaType::Int))
            }
            Expression::Float(_) => {
                expr.typed = AzulaType::Float;
                Ok((expr.clone(), AzulaType::Float))
            }
            Expression::Boolean(_) => {
                expr.typed = AzulaType::Bool;
                Ok((expr.clone(), AzulaType::Bool))
            }
            Expression::String(_) => {
                expr.typed = AzulaType::Pointer(Rc::new(AzulaType::Str));
                Ok((expr.clone(), AzulaType::Pointer(Rc::new(AzulaType::Str))))
            }
            Expression::Identifier(ref name) => {
                if name == "nil" {
                    return Ok((expr.clone(), AzulaType::Void));
                }
                if let Some(variable) = env.variable_definitions.get(name) {
                    expr.typed = variable.typ.clone().into();

                    Ok((expr.clone(), variable.typ.clone()))
                } else if let Some(variable) = self.globals.get(name) {
                    expr.typed = variable.typ.clone().into();

                    Ok((expr.clone(), variable.typ.clone()))
                } else {
                    self.errors.push(AzulaError::new(
                        ErrorType::UnknownVariable(name.clone()),
                        expr.span.start,
                        expr.span.end,
                    ));
                    return Err(format!("Unknown variable {:?}", name));
                }
            }
            Expression::FunctionCall { mut function, args } => {
                let func = match self.resolve_function(
                    self.functions.clone(),
                    function.deref().clone(),
                    env,
                ) {
                    Ok(f) => f,
                    Err(e) => {
                        self.errors.push(AzulaError::new(
                            ErrorType::FunctionNotFound(e),
                            function.span.start,
                            function.span.end,
                        ));
                        return Err(format!("Function not found {:?}", function));
                    }
                };

                let function = self.typecheck_function_def(function.deref().clone(), env);

                let return_type = func.returns;

                let mut new_args = vec![];
                for arg in args.clone() {
                    let (arg, _) = match self.typecheck_expression(arg, env) {
                        Ok((arg, typ)) => (arg, typ),
                        Err(e) => return Err(e),
                    };
                    new_args.push(arg);
                }

                return Ok((
                    ExpressionNode {
                        expression: Expression::FunctionCall {
                            function: Rc::new(function),
                            args: new_args,
                        },
                        typed: return_type.clone(),
                        span: expr.span,
                    },
                    return_type.clone(),
                ));
            }
            Expression::Not(exp) => {
                let (node, typ) = match self.typecheck_expression(exp.deref().clone(), env) {
                    Ok((node, typ)) => (node, typ),
                    Err(e) => return Err(e),
                };

                if typ != AzulaType::Bool {
                    self.errors.push(AzulaError::new(
                        ErrorType::NonBoolCondition(format!("{:?}", typ)),
                        expr.span.start,
                        expr.span.end,
                    ));

                    return Err("Non-bool in condition".to_string());
                }

                return Ok((
                    ExpressionNode {
                        expression: Expression::Not(Rc::new(node)),
                        typed: typ,
                        span: expr.span,
                    },
                    AzulaType::Bool,
                ));
            }
            Expression::Negate(exp) => {
                let (node, typ) = match self.typecheck_expression(exp.deref().clone(), env) {
                    Ok((node, typ)) => (node, typ),
                    Err(e) => return Err(e),
                };

                if typ != AzulaType::Int && typ != AzulaType::Float {
                    self.errors.push(AzulaError::new(
                        ErrorType::MismatchedTypes(format!("{:?}", typ), "Int or Float".to_string()),
                        expr.span.start,
                        expr.span.end,
                    ));
                    return Err("Negate requires numeric type".to_string());
                }

                return Ok((
                    ExpressionNode {
                        expression: Expression::Negate(Rc::new(node)),
                        typed: typ.clone(),
                        span: expr.span,
                    },
                    typ,
                ));
            }
            Expression::Pointer(exp) => {
                let (node, typ) = match self.typecheck_expression(exp.deref().clone(), env) {
                    Ok((node, typ)) => (node, typ),
                    Err(e) => return Err(e),
                };

                return Ok((
                    ExpressionNode {
                        expression: Expression::Pointer(Rc::new(node)),
                        typed: AzulaType::Pointer(Rc::new(typ.clone())),
                        span: expr.span,
                    },
                    AzulaType::Pointer(Rc::new(typ)),
                ));
            }
            Expression::Array(items) => {
                let typs = items
                    .iter()
                    .map(|v| self.typecheck_expression(v.clone(), env))
                    .collect::<Vec<_>>();

                if typs.is_empty() {
                    return Ok((
                        ExpressionNode {
                            expression: Expression::Array(vec![]),
                            typed: AzulaType::Array(Rc::new(AzulaType::Infer), Some(0)),
                            span: expr.span,
                        },
                        AzulaType::Array(Rc::new(AzulaType::Infer), Some(0)),
                    ));
                }

                let first_typ = &typs[0].as_ref().unwrap().1;

                for val in typs.clone() {
                    let (node, typ) = match val {
                        Ok((node, typ)) => (node, typ),
                        Err(_) => continue,
                    };
                    if typ != first_typ.clone() {
                        self.errors.push(AzulaError::new(
                            ErrorType::MismatchedTypes(
                                format!("{:?}", typ),
                                format!("{:?}", first_typ),
                            ),
                            node.span.start,
                            node.span.end,
                        ));
                    }
                }

                Ok((
                    ExpressionNode {
                        expression: Expression::Array(
                            typs.clone()
                                .iter()
                                .map(|s| s.as_ref().unwrap().clone())
                                .map(|(node, _)| node)
                                .collect(),
                        ),
                        typed: AzulaType::Array(Rc::new(first_typ.clone()), Some(typs.len())),
                        span: expr.span,
                    },
                    AzulaType::Array(Rc::new(first_typ.clone()), Some(typs.len())),
                ))
            }
            Expression::ArrayAccess(array, index) => {
                let (array, array_typ) = self
                    .typecheck_expression(array.deref().clone(), env)
                    .unwrap();

                let (index, typ) = self
                    .typecheck_expression(index.deref().clone(), env)
                    .unwrap();

                if typ != AzulaType::Int {
                    self.errors.push(AzulaError::new(
                        ErrorType::NonIntIndex(format!("{:?}", typ)),
                        array.span.start,
                        index.span.end,
                    ));

                    return Err("Non int index".to_string());
                }

                let return_typ = if array_typ.is_indexable() {
                    match array_typ {
                        AzulaType::Array(nested, _) => nested.deref().clone(),
                        AzulaType::Str => AzulaType::SizedSignedInt(8),
                        AzulaType::Pointer(nested) => match nested.deref().clone() {
                            AzulaType::Str => AzulaType::SizedSignedInt(8),
                            _ => nested.deref().clone(),
                        },
                        _ => unreachable!(),
                    }
                } else {
                    self.errors.push(AzulaError::new(
                        ErrorType::NonArrayInIndex(format!("{:?}", array_typ)),
                        array.span.start,
                        array.span.end,
                    ));
                    return Err("non-array in index".to_string());
                };

                return Ok((
                    ExpressionNode {
                        expression: Expression::ArrayAccess(Rc::new(array), Rc::new(index)),
                        typed: return_typ.clone(),
                        span: expr.span,
                    },
                    return_typ,
                ));
            }
            Expression::StructInitialisation(struc, attrs) => {
                let name = match &struc.clone().expression {
                    Expression::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };

                let mut attrs_new = vec![];
                for (name, attr) in attrs.iter() {
                    let expr = match self.typecheck_expression(attr.clone(), env) {
                        Ok((expr, _)) => expr,
                        Err(e) => return Err(e),
                    };
                    attrs_new.push((*name, expr));
                }

                return Ok((
                    ExpressionNode {
                        expression: Expression::StructInitialisation(struc, attrs_new),
                        typed: AzulaType::Named(name.clone()),
                        span: expr.span,
                    },
                    AzulaType::Named(name.clone()),
                ));
            }
            Expression::StructAccess(struc, access) => {
                let (struc, struc_type) =
                    match self.typecheck_expression(struc.deref().clone(), env) {
                        Ok(x) => x,
                        Err(e) => return Err(e),
                    };

                let struc_name = match struc_type {
                    AzulaType::Named(s) => s,
                    AzulaType::Pointer(nested) => match nested.deref().clone() {
                        AzulaType::Named(s) => s,
                        _ => {
                            self.errors.push(AzulaError::new(
                                ErrorType::AccessNonStruct,
                                struc.span.start,
                                struc.span.end,
                            ));
                            return Err("accessing non-struct".to_string());
                        }
                    },
                    _ => {
                        self.errors.push(AzulaError::new(
                            ErrorType::AccessNonStruct,
                            struc.span.start,
                            struc.span.end,
                        ));
                        return Err("accessing non-struct".to_string());
                    }
                };

                let struct_type = if let Some(struct_type) = self.structs.get(&struc_name) {
                    struct_type
                } else {
                    self.errors.push(AzulaError::new(
                        ErrorType::UnknownStruct(struc_name),
                        struc.span.start,
                        struc.span.end,
                    ));
                    return Err("Struct not found".to_string());
                };

                let member_name = match &access.expression {
                    Expression::Identifier(s) => s,
                    _ => {
                        self.errors.push(AzulaError::new(
                            ErrorType::AccessNonStruct,
                            access.span.start,
                            access.span.end,
                        ));
                        return Err("accessing non-struct".to_string());
                    }
                };

                let typ = match struct_type
                    .attrs
                    .iter()
                    .find(|(_, name)| name.to_string() == member_name.clone())
                {
                    Some((typ, _)) => typ,
                    _ => {
                        self.errors.push(AzulaError::new(
                            ErrorType::UnknownStructMember(member_name.clone(), struc_name),
                            access.span.start,
                            access.span.end,
                        ));
                        return Err("unknown struct member".to_string());
                    }
                };

                return Ok((
                    ExpressionNode {
                        expression: Expression::StructAccess(Rc::new(struc), access),
                        typed: typ.clone(),
                        span: expr.span,
                    },
                    typ.clone(),
                ));
            }
            Expression::NamespaceAccess(ns, identifier) => {
                let ns_name = if let Expression::Identifier(ref ident) = ns.deref().expression {
                    ident.clone()
                } else {
                    unreachable!()
                };

                // Check if it's an enum variant access
                if let Some(variants) = self.enums.get(&ns_name).cloned() {
                    let variant_name = match &identifier.expression {
                        Expression::Identifier(v) => v.clone(),
                        _ => unreachable!(),
                    };
                    if !variants.contains(&variant_name) {
                        self.errors.push(AzulaError::new(
                            ErrorType::UnknownVariant(variant_name.clone(), ns_name.clone()),
                            identifier.span.start,
                            identifier.span.end,
                        ));
                        return Err(format!("Unknown variant {} on {}", variant_name, ns_name));
                    }
                    let typ = AzulaType::Named(ns_name.clone());
                    return Ok((
                        ExpressionNode {
                            expression: Expression::NamespaceAccess(ns, identifier),
                            typed: typ.clone(),
                            span: expr.span,
                        },
                        typ,
                    ));
                }

                let namespace = self.namespaces.get(&ns_name);
                let namespace = match namespace {
                    Some(f) => f.clone(),
                    None => return Err("namespace".to_string()),
                };

                let mut ns_node = ns.deref().clone();
                ns_node.typed = AzulaType::Named(namespace.name.clone());

                return Ok((
                    ExpressionNode {
                        expression: Expression::NamespaceAccess(Rc::new(ns_node), identifier),
                        typed: AzulaType::Infer,
                        span: expr.span,
                    },
                    AzulaType::Infer,
                ));
            }
            Expression::Match(scrutinee, arms) => {
                self.typecheck_match(scrutinee, arms, expr.span, env)
            }
            Expression::Cast(_, _) => self.typecheck_cast_expression(expr, env),
            Expression::Null => Ok((
                ExpressionNode { expression: Expression::Null, typed: AzulaType::Str, span: expr.span },
                AzulaType::Str,
            )),
            Expression::Alloc(inner) => {
                let (inner_node, inner_typ) = self.typecheck_expression(inner.as_ref().clone(), env)?;
                let ptr_type = AzulaType::Pointer(Rc::new(inner_typ));
                Ok((
                    ExpressionNode {
                        expression: Expression::Alloc(Rc::new(inner_node)),
                        typed: ptr_type.clone(),
                        span: expr.span,
                    },
                    ptr_type,
                ))
            }
            Expression::Block(stmts, final_expr) => {
                let mut new_env = env.clone();
                let mut new_stmts = vec![];
                for stmt in stmts {
                    match self.typecheck_statement(stmt, &mut new_env) {
                        Ok((s, _)) => new_stmts.push(s),
                        Err(e) => return Err(e),
                    }
                }
                match final_expr {
                    Some(fe) => {
                        let (fe_node, fe_type) =
                            self.typecheck_expression(fe.as_ref().clone(), &new_env)?;
                        Ok((
                            ExpressionNode {
                                expression: Expression::Block(new_stmts, Some(Rc::new(fe_node))),
                                typed: fe_type.clone(),
                                span: expr.span,
                            },
                            fe_type,
                        ))
                    }
                    None => Ok((
                        ExpressionNode {
                            expression: Expression::Block(new_stmts, None),
                            typed: AzulaType::Void,
                            span: expr.span,
                        },
                        AzulaType::Void,
                    )),
                }
            }
        }
    }

    fn typecheck_match(
        &mut self,
        scrutinee: Rc<ExpressionNode<'a>>,
        arms: Vec<(MatchPattern<'a>, ExpressionNode<'a>)>,
        span: Span,
        env: &Environment<'a>,
    ) -> Result<(ExpressionNode<'a>, AzulaType<'a>), String> {
        let (scrut_node, scrut_type) = match self.typecheck_expression(scrutinee.deref().clone(), env) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };

        // Dispatch based on scrutinee type: enum match or integer match
        let is_integer_match = matches!(scrut_type, AzulaType::Int | AzulaType::SizedSignedInt(_) | AzulaType::SizedUnsignedInt(_));

        let (enum_name, variants) = if is_integer_match {
            (String::new(), vec![])
        } else {
            let name = match &scrut_type {
                AzulaType::Named(n) => n.clone(),
                _ => {
                    self.errors.push(AzulaError::new(
                        ErrorType::MatchOnNonEnum(format!("{:?}", scrut_type)),
                        scrut_node.span.start,
                        scrut_node.span.end,
                    ));
                    return Err("match on non-enum".to_string());
                }
            };
            let vars = match self.enums.get(&name).cloned() {
                Some(v) => v,
                None => {
                    self.errors.push(AzulaError::new(
                        ErrorType::UnknownEnum(name.clone()),
                        scrut_node.span.start,
                        scrut_node.span.end,
                    ));
                    return Err(format!("Unknown enum {}", name));
                }
            };
            (name, vars)
        };

        let mut covered: Vec<String> = vec![];
        let mut has_wildcard = false;
        let mut result_type: Option<AzulaType<'a>> = None;
        let mut new_arms = vec![];

        for (pattern, body) in arms {
            match &pattern {
                MatchPattern::Wildcard => {
                    has_wildcard = true;
                }
                MatchPattern::Integer(_) => {
                    if !is_integer_match {
                        self.errors.push(AzulaError::new(
                            ErrorType::MatchOnNonEnum(format!("{:?}", scrut_type)),
                            body.span.start,
                            body.span.end,
                        ));
                        return Err("integer pattern on non-integer scrutinee".to_string());
                    }
                }
                MatchPattern::Variant(pat_enum, variant) => {
                    if *pat_enum != enum_name.as_str() {
                        self.errors.push(AzulaError::new(
                            ErrorType::UnknownEnum(pat_enum.to_string()),
                            body.span.start,
                            body.span.end,
                        ));
                        return Err(format!("Unknown enum {}", pat_enum));
                    }
                    if !variants.contains(&variant.to_string()) {
                        self.errors.push(AzulaError::new(
                            ErrorType::UnknownVariant(variant.to_string(), enum_name.clone()),
                            body.span.start,
                            body.span.end,
                        ));
                        return Err(format!("Unknown variant {}", variant));
                    }
                    covered.push(variant.to_string());
                }
            }

            let (body_node, body_type) = match self.typecheck_expression(body, env) {
                Ok(x) => x,
                Err(e) => return Err(e),
            };

            if let Some(ref rt) = result_type.clone() {
                if *rt != body_type {
                    self.errors.push(AzulaError::new(
                        ErrorType::MismatchedTypes(
                            format!("{:?}", rt),
                            format!("{:?}", body_type),
                        ),
                        body_node.span.start,
                        body_node.span.end,
                    ));
                    return Err("mismatched arm types".to_string());
                }
            } else {
                result_type = Some(body_type);
            }

            new_arms.push((pattern, body_node));
        }

        // Exhaustiveness: enum match requires all variants covered or wildcard;
        // integer match just requires a wildcard (infinite domain).
        if !has_wildcard && !is_integer_match {
            let uncovered: Vec<_> = variants.iter().filter(|v| !covered.contains(v)).collect();
            if !uncovered.is_empty() {
                self.errors.push(AzulaError::new(
                    ErrorType::NonExhaustiveMatch(enum_name.clone()),
                    span.start,
                    span.end,
                ));
                return Err("non-exhaustive match".to_string());
            }
        }

        let typ = result_type.unwrap_or(AzulaType::Void);
        Ok((
            ExpressionNode {
                expression: Expression::Match(Rc::new(scrut_node), new_arms),
                typed: typ.clone(),
                span,
            },
            typ,
        ))
    }

    fn resolve_type(&self, typ: AzulaType<'a>) -> AzulaType<'a> {
        match &typ {
            AzulaType::Named(n) => {
                if let Some(resolved) = self.type_aliases.get(n.as_str()) {
                    self.resolve_type(resolved.clone())
                } else {
                    typ
                }
            }
            AzulaType::Pointer(inner) => {
                AzulaType::Pointer(Rc::new(self.resolve_type(inner.as_ref().clone())))
            }
            AzulaType::Array(inner, size) => {
                AzulaType::Array(Rc::new(self.resolve_type(inner.as_ref().clone())), *size)
            }
            _ => typ,
        }
    }

    fn typecheck_cast_expression(
        &mut self,
        expr: ExpressionNode<'a>,
        env: &Environment<'a>,
    ) -> Result<(ExpressionNode<'a>, AzulaType<'a>), String> {
        if let Expression::Cast(ref inner, ref target_type) = expr.expression {
            let (inner_node, _) = self.typecheck_expression(inner.deref().clone(), env)?;
            let typ = target_type.clone();
            return Ok((
                ExpressionNode {
                    expression: Expression::Cast(Rc::new(inner_node), typ.clone()),
                    typed: typ.clone(),
                    span: expr.span,
                },
                typ,
            ));
        }
        unreachable!()
    }

    fn typecheck_infix_expression(
        &mut self,
        mut expr: ExpressionNode<'a>,
        env: &Environment<'a>,
    ) -> Result<(ExpressionNode<'a>, AzulaType<'a>), String> {
        if let Expression::Infix(ref left, ref operator, ref right) = expr.expression {
            let (left, left_typ) = match self.typecheck_expression(left.deref().clone(), env) {
                Ok((left, typ)) => (left, typ),
                Err(e) => return Err(e),
            };

            let (right, right_typ) = match self.typecheck_expression(right.deref().clone(), env) {
                Ok((right, typ)) => (right, typ),
                Err(e) => return Err(e),
            };

            let allowed = hashmap! {
                Operator::Add => vec![AzulaType::Int, AzulaType::Float],
                Operator::Sub => vec![AzulaType::Int, AzulaType::Float],
                Operator::Mul => vec![AzulaType::Int, AzulaType::Float],
                Operator::Div => vec![AzulaType::Int, AzulaType::Float],
                Operator::Mod => vec![AzulaType::Int, AzulaType::Float],
                Operator::Power => vec![AzulaType::Int, AzulaType::Float],
                Operator::Or => vec![AzulaType::Bool],
                Operator::And => vec![AzulaType::Bool],
                Operator::Eq => vec![AzulaType::Int, AzulaType::Float, AzulaType::Bool],
                Operator::Neq => vec![AzulaType::Int, AzulaType::Float, AzulaType::Bool],
                Operator::Lt => vec![AzulaType::Int, AzulaType::Float],
                Operator::Lte => vec![AzulaType::Int, AzulaType::Float],
                Operator::Gt => vec![AzulaType::Int, AzulaType::Float],
                Operator::Gte => vec![AzulaType::Int, AzulaType::Float],
            };

            // Enum types lower to i64 and pointer types lower to ptr — both compare as Int
            let effective_left = match &left_typ {
                AzulaType::Named(n) if self.enums.contains_key(n.as_str()) => AzulaType::Int,
                AzulaType::Pointer(_) | AzulaType::Str => AzulaType::Int,
                AzulaType::SizedSignedInt(_) => AzulaType::Int,
                _ => left_typ.clone(),
            };
            let effective_right = match &right_typ {
                AzulaType::Named(n) if self.enums.contains_key(n.as_str()) => AzulaType::Int,
                AzulaType::Pointer(_) | AzulaType::Str => AzulaType::Int,
                AzulaType::SizedSignedInt(_) => AzulaType::Int,
                _ => right_typ.clone(),
            };

            let allowed = allowed.get(operator).unwrap();
            if !allowed.contains(&effective_left) {
                self.errors.push(AzulaError::new(
                    ErrorType::NonOperatorType(
                        format!("{:?}", left_typ),
                        format!("{:?}", operator),
                    ),
                    left.span.start,
                    left.span.end,
                ));
                return Err("cannot use operator with type".to_string());
            }

            if !allowed.contains(&effective_right) {
                self.errors.push(AzulaError::new(
                    ErrorType::NonOperatorType(
                        format!("{:?}", right_typ),
                        format!("{:?}", operator),
                    ),
                    right.span.start,
                    right.span.end,
                ));
                return Err("cannot use operator with type".to_string());
            }

            match operator {
                Operator::Add => {
                    if left_typ != right_typ {
                        self.errors.push(AzulaError::new(
                            ErrorType::MismatchedTypes(
                                format!("{:?}", left_typ),
                                format!("{:?}", right_typ),
                            ),
                            left.span.start,
                            right.span.end,
                        ));
                        return Err("mismatched types in infix".to_string());
                    }

                    expr.typed = left.clone().typed;
                    Ok((
                        ExpressionNode {
                            expression: Expression::Infix(
                                Rc::new(left),
                                operator.clone(),
                                Rc::new(right),
                            ),
                            typed: left_typ.clone().into(),
                            span: expr.span,
                        },
                        left_typ,
                    ))
                }
                Operator::Sub => {
                    if left_typ != right_typ {
                        self.errors.push(AzulaError::new(
                            ErrorType::MismatchedTypes(
                                format!("{:?}", left_typ),
                                format!("{:?}", right_typ),
                            ),
                            left.span.start,
                            right.span.end,
                        ));
                        return Err("mismatched types in infix".to_string());
                    }

                    expr.typed = left.clone().typed;
                    Ok((
                        ExpressionNode {
                            expression: Expression::Infix(
                                Rc::new(left),
                                operator.clone(),
                                Rc::new(right),
                            ),
                            typed: left_typ.clone().into(),
                            span: expr.span,
                        },
                        left_typ,
                    ))
                }
                Operator::Mul => {
                    if left_typ != right_typ {
                        self.errors.push(AzulaError::new(
                            ErrorType::MismatchedTypes(
                                format!("{:?}", left_typ),
                                format!("{:?}", right_typ),
                            ),
                            left.span.start,
                            right.span.end,
                        ));
                        return Err("mismatched types in infix".to_string());
                    }

                    expr.typed = left.clone().typed;
                    Ok((
                        ExpressionNode {
                            expression: Expression::Infix(
                                Rc::new(left),
                                operator.clone(),
                                Rc::new(right),
                            ),
                            typed: left_typ.clone().into(),
                            span: expr.span,
                        },
                        left_typ,
                    ))
                }
                Operator::Div => {
                    if left_typ != right_typ {
                        self.errors.push(AzulaError::new(
                            ErrorType::MismatchedTypes(
                                format!("{:?}", left_typ),
                                format!("{:?}", right_typ),
                            ),
                            left.span.start,
                            right.span.end,
                        ));
                        return Err("mismatched types in infix".to_string());
                    }

                    expr.typed = left.clone().typed;
                    Ok((
                        ExpressionNode {
                            expression: Expression::Infix(
                                Rc::new(left),
                                operator.clone(),
                                Rc::new(right),
                            ),
                            typed: left_typ.clone().into(),
                            span: expr.span,
                        },
                        left_typ,
                    ))
                }
                Operator::Mod => {
                    if left_typ != right_typ {
                        self.errors.push(AzulaError::new(
                            ErrorType::MismatchedTypes(
                                format!("{:?}", left_typ),
                                format!("{:?}", right_typ),
                            ),
                            left.span.start,
                            right.span.end,
                        ));
                        return Err("mismatched types in infix".to_string());
                    }

                    expr.typed = left.clone().typed;
                    Ok((
                        ExpressionNode {
                            expression: Expression::Infix(
                                Rc::new(left),
                                operator.clone(),
                                Rc::new(right),
                            ),
                            typed: left_typ.clone().into(),
                            span: expr.span,
                        },
                        left_typ,
                    ))
                }
                Operator::Power => {
                    if left_typ != right_typ {
                        self.errors.push(AzulaError::new(
                            ErrorType::MismatchedTypes(
                                format!("{:?}", left_typ),
                                format!("{:?}", right_typ),
                            ),
                            left.span.start,
                            right.span.end,
                        ));
                        return Err("mismatched types in infix".to_string());
                    }

                    expr.typed = left.clone().typed;
                    Ok((
                        ExpressionNode {
                            expression: Expression::Infix(
                                Rc::new(left),
                                operator.clone(),
                                Rc::new(right),
                            ),
                            typed: left_typ.clone().into(),
                            span: expr.span,
                        },
                        left_typ,
                    ))
                }
                Operator::Or
                | Operator::And
                | Operator::Eq
                | Operator::Neq
                | Operator::Lt
                | Operator::Lte
                | Operator::Gt
                | Operator::Gte => {
                    expr.typed = AzulaType::Bool;
                    Ok((
                        ExpressionNode {
                            expression: Expression::Infix(
                                Rc::new(left),
                                operator.clone(),
                                Rc::new(right),
                            ),
                            typed: AzulaType::Bool,
                            span: expr.span,
                        },
                        AzulaType::Bool,
                    ))
                }
            }
        } else {
            unreachable!()
        }
    }

    fn typecheck_function_def(
        &mut self,
        mut expr: ExpressionNode<'a>,
        env: &Environment<'a>,
    ) -> ExpressionNode<'a> {
        if let Expression::Identifier(..) = expr.expression {
            return expr.clone();
        }

        if let Expression::NamespaceAccess(ref ns, _) = expr.expression {
            let namespace = if let Expression::Identifier(ident) = ns.deref().clone().expression {
                ident
            } else {
                unreachable!()
            };

            expr.typed = AzulaType::Named(namespace);
            return expr.clone();
        }

        if let Expression::StructAccess(ref struc, ref right) = expr.expression {
            let (typechecked_struc, resolved_type) = self
                .typecheck_expression(struc.deref().clone(), env)
                .unwrap();

            return ExpressionNode {
                expression: Expression::StructAccess(
                    Rc::new(typechecked_struc),
                    right.clone(),
                ),
                typed: resolved_type.clone(),
                span: expr.span,
            };
        }

        expr
    }

    fn resolve_function(
        &mut self,
        namespace: HashMap<&'a str, FunctionDefinition<'a>>,
        expr: ExpressionNode<'a>,
        env: &Environment<'a>,
    ) -> Result<FunctionDefinition<'a>, String> {
        if let Expression::Identifier(s) = expr.expression {
            return match namespace.get(&s.as_str()) {
                Some(f) => Ok(f.clone()),
                None => {
                    if s == "printf" || s == "sprintf" || s == "puts" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![],
                            varargs: true,
                            returns: AzulaType::Void,
                        })
                    } else if s == "strlen" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "s".to_string())],
                            varargs: false,
                            returns: AzulaType::Int,
                        })
                    } else if s == "strcmp" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "a".to_string()), (AzulaType::Str, "b".to_string())],
                            varargs: false,
                            returns: AzulaType::Int,
                        })
                    } else if s == "malloc" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Int, "size".to_string())],
                            varargs: false,
                            returns: AzulaType::Str,
                        })
                    } else if s == "memcpy" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![
                                (AzulaType::Str, "dest".to_string()),
                                (AzulaType::Str, "src".to_string()),
                                (AzulaType::Int, "n".to_string()),
                            ],
                            varargs: false,
                            returns: AzulaType::Str,
                        })
                    } else if s == "free" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "ptr".to_string())],
                            varargs: false,
                            returns: AzulaType::Void,
                        })
                    } else if s == "realloc" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "ptr".to_string()), (AzulaType::Int, "size".to_string())],
                            varargs: false,
                            returns: AzulaType::Str,
                        })
                    } else if s == "ptr_read_int" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "ptr".to_string())],
                            varargs: false,
                            returns: AzulaType::Int,
                        })
                    } else if s == "ptr_write_int" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "ptr".to_string()), (AzulaType::Int, "val".to_string())],
                            varargs: false,
                            returns: AzulaType::Void,
                        })
                    } else if s == "ptr_read_str" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "ptr".to_string())],
                            varargs: false,
                            returns: AzulaType::Str,
                        })
                    } else if s == "ptr_write_str" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "ptr".to_string()), (AzulaType::Str, "val".to_string())],
                            varargs: false,
                            returns: AzulaType::Void,
                        })
                    } else if s == "ptr_add" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "ptr".to_string()), (AzulaType::Int, "offset".to_string())],
                            varargs: false,
                            returns: AzulaType::Str,
                        })
                    } else if s == "fopen" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "path".to_string()), (AzulaType::Str, "mode".to_string())],
                            varargs: false,
                            returns: AzulaType::Str,
                        })
                    } else if s == "fclose" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "file".to_string())],
                            varargs: false,
                            returns: AzulaType::Int,
                        })
                    } else if s == "fseek" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![
                                (AzulaType::Str, "file".to_string()),
                                (AzulaType::Int, "offset".to_string()),
                                (AzulaType::Int, "whence".to_string()),
                            ],
                            varargs: false,
                            returns: AzulaType::Int,
                        })
                    } else if s == "ftell" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![(AzulaType::Str, "file".to_string())],
                            varargs: false,
                            returns: AzulaType::Int,
                        })
                    } else if s == "fread" {
                        Ok(FunctionDefinition {
                            name: s,
                            args: vec![
                                (AzulaType::Str, "buf".to_string()),
                                (AzulaType::Int, "size".to_string()),
                                (AzulaType::Int, "count".to_string()),
                                (AzulaType::Str, "file".to_string()),
                            ],
                            varargs: false,
                            returns: AzulaType::Int,
                        })
                    } else {
                        Err(s)
                    }
                }
            };
        }

        if let Expression::NamespaceAccess(ns, identifier) = expr.expression {
            let namespace = if let Expression::Identifier(ident) = ns.deref().clone().expression {
                let namespace = self.namespaces.get(&ident);
                match namespace {
                    Some(f) => Ok(f.clone()),
                    None => Err(ident),
                }
            } else {
                unreachable!()
            };

            if namespace.is_err() {
                return Err("namespace".to_string());
            }

            let namespace = namespace.unwrap();

            return self.resolve_function(namespace.funcs.clone(), identifier.deref().clone(), env);
        }

        if let Expression::StructAccess(struc, method) = expr.expression {
            let (_, resolved_type) = self
                .typecheck_expression(struc.deref().clone(), env)
                .unwrap();

            let namespace = self.namespaces.get(&resolved_type.to_string());

            if namespace.is_none() {
                return Err("namespace".to_string());
            }

            let namespace = namespace.unwrap();

            return self.resolve_function(namespace.funcs.clone(), method.deref().clone(), env);
        }

        Err("none".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azula_ast::prelude::Span;
    use std::rc::Rc;

    #[test]
    fn test_function() {
        let root = Statement::Root(vec![Statement::Function {
            name: "main",
            args: vec![(AzulaType::Int, "x")],
            returns: AzulaType::Bool,
            body: Rc::new(Statement::Block(vec![])),
            span: Span { start: 0, end: 1 },
        }]);

        let mut typechecker = Typechecker::new(root);
        typechecker.typecheck().unwrap();
    }

    #[test]
    fn test_assign() {
        let mut typechecker = Typechecker::new(Statement::Root(vec![]));

        let mut environment = Environment::new();
        typechecker.typecheck_assign(
            Statement::Assign(
                true,
                "test".to_string(),
                None,
                ExpressionNode {
                    expression: Expression::Integer(5),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 1 },
                },
                Span { start: 0, end: 1 },
            ),
            &mut environment,
        );

        let var = environment
            .variable_definitions
            .get(&"test".to_string())
            .unwrap();
        assert_eq!(var.name, "test");
        assert_eq!(var.typ, AzulaType::Int);
    }

    #[test]
    fn test_return() {
        // Return value
        let mut typechecker = Typechecker::new(Statement::Root(vec![]));

        let mut environment = Environment::new();
        let (_, typ) = typechecker
            .typecheck_return(
                Statement::Return(
                    Some(ExpressionNode {
                        expression: Expression::Integer(5),
                        typed: AzulaType::Infer,
                        span: Span { start: 0, end: 1 },
                    }),
                    Span { start: 0, end: 1 },
                ),
                &mut environment,
            )
            .unwrap();
        assert_eq!(typ, AzulaType::Int);

        // Return none
        let mut typechecker = Typechecker::new(Statement::Root(vec![]));

        let mut environment = Environment::new();
        let (_, typ) = typechecker
            .typecheck_return(
                Statement::Return(None, Span { start: 0, end: 1 }),
                &mut environment,
            )
            .unwrap();
        assert_eq!(typ, AzulaType::Void);
    }

    #[test]
    fn test_integer_expression() {
        let integer_node = ExpressionNode {
            expression: Expression::Integer(5),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 0 },
        };

        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        let mut environment = Environment::new();
        let (expr, typ) = typechecker
            .typecheck_expression(integer_node, &environment)
            .unwrap();

        assert_eq!(typ, AzulaType::Int);
        assert_eq!(expr.typed, AzulaType::Int);
    }

    #[test]
    fn test_identifier_expression() {
        // Integer
        let identifier_node = ExpressionNode {
            expression: Expression::Identifier("test".to_string()),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 0 },
        };

        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        let mut environment = Environment::new();
        environment.add_variable(
            "test".to_string(),
            VariableDefinition {
                name: "test".to_string(),
                mutable: true,
                typ: AzulaType::Int,
            },
        );
        let (expr, typ) = typechecker
            .typecheck_expression(identifier_node, &environment)
            .unwrap();

        assert_eq!(typ, AzulaType::Int);
        assert_eq!(expr.typed, AzulaType::Int);

        // Pointer
        let identifier_node = ExpressionNode {
            expression: Expression::Identifier("test".to_string()),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 0 },
        };

        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        let mut environment = Environment::new();
        environment.add_variable(
            "test".to_string(),
            VariableDefinition {
                typ: AzulaType::Pointer(Rc::new(AzulaType::Str)),
                name: "test".to_string(),
                mutable: true,
            },
        );
        let (expr, typ) = typechecker
            .typecheck_expression(identifier_node, &environment)
            .unwrap();

        assert_eq!(typ, AzulaType::Pointer(Rc::new(AzulaType::Str)));
        assert_eq!(expr.typed, AzulaType::Pointer(Rc::new(AzulaType::Str)));
    }

    #[test]
    fn test_infix_expression() {
        // Int
        let infix_node = ExpressionNode {
            expression: Expression::Infix(
                Rc::new(ExpressionNode {
                    expression: Expression::Integer(5),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 0 },
                }),
                Operator::Add,
                Rc::new(ExpressionNode {
                    expression: Expression::Integer(20),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 0 },
                }),
            ),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 0 },
        };

        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        let mut environment = Environment::new();
        let (expr, typ) = typechecker
            .typecheck_expression(infix_node, &environment)
            .unwrap();

        assert_eq!(typ, AzulaType::Int);
        assert_eq!(expr.typed, AzulaType::Int);

        // Non operator type
        let infix_node = ExpressionNode {
            expression: Expression::Infix(
                Rc::new(ExpressionNode {
                    expression: Expression::Integer(5),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 0 },
                }),
                Operator::Add,
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("test".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 0 },
                }),
            ),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 0 },
        };

        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        let mut environment = Environment::new();
        environment.add_variable(
            "test".to_string(),
            VariableDefinition {
                name: "test".to_string(),
                typ: AzulaType::Bool,
                mutable: true,
            },
        );
        typechecker.typecheck_expression(infix_node, &environment);

        assert_eq!(typechecker.errors.len(), 1);
        assert!(matches!(
            typechecker.errors[0].error_type,
            ErrorType::NonOperatorType(..)
        ));
    }

    #[test]
    fn test_array_expression() {
        // Int
        let array = ExpressionNode {
            expression: Expression::Array(vec![
                ExpressionNode {
                    expression: Expression::Integer(1),
                    typed: AzulaType::Int,
                    span: Span { start: 0, end: 1 },
                },
                ExpressionNode {
                    expression: Expression::Integer(2),
                    typed: AzulaType::Int,
                    span: Span { start: 0, end: 1 },
                },
            ]),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 0 },
        };

        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        let mut environment = Environment::new();
        let (expr, typ) = typechecker
            .typecheck_expression(array, &environment)
            .unwrap();

        assert_eq!(typ, AzulaType::Array(Rc::new(AzulaType::Int), Some(2)));
        assert_eq!(
            expr.typed,
            AzulaType::Array(Rc::new(AzulaType::Int), Some(2))
        );

        // Different types
        let array = ExpressionNode {
            expression: Expression::Array(vec![
                ExpressionNode {
                    expression: Expression::Integer(1),
                    typed: AzulaType::Int,
                    span: Span { start: 0, end: 1 },
                },
                ExpressionNode {
                    expression: Expression::Boolean(false),
                    typed: AzulaType::Bool,
                    span: Span { start: 0, end: 1 },
                },
            ]),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 0 },
        };

        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        let mut environment = Environment::new();
        let (expr, typ) = typechecker
            .typecheck_expression(array, &environment)
            .unwrap();

        assert_eq!(typechecker.errors.len(), 1);
        assert!(matches!(
            typechecker.errors[0].error_type,
            ErrorType::MismatchedTypes(..)
        ));
    }

    #[test]
    fn test_struct_access_expression() {
        // Int
        let array = ExpressionNode {
            expression: Expression::StructAccess(
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("x".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 1 },
                }),
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("test".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 1 },
                }),
            ),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 0 },
        };

        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        typechecker.structs.insert(
            "Test".to_string(),
            StructDefinition {
                name: "Test",
                attrs: vec![(AzulaType::Int, "test")],
            },
        );
        let mut environment = Environment::new();
        environment.add_variable(
            "x".to_string(),
            VariableDefinition {
                name: "x".to_string(),
                mutable: false,
                typ: AzulaType::Named("Test".to_string()),
            },
        );
        let (expr, typ) = typechecker
            .typecheck_expression(array, &environment)
            .unwrap();

        assert_eq!(typ, AzulaType::Int);
        assert_eq!(expr.typed, AzulaType::Int);
    }

    #[test]
    fn test_enum_registration() {
        let root = Statement::Root(vec![Statement::Enum {
            name: "Color",
            variants: vec!["Red", "Green", "Blue"],
            span: Span { start: 0, end: 1 },
        }]);
        let mut typechecker = Typechecker::new(root);
        typechecker.typecheck().unwrap();
        assert!(typechecker.enums.contains_key("Color"));
        assert_eq!(
            typechecker.enums["Color"],
            vec!["Red", "Green", "Blue"]
        );
    }

    #[test]
    fn test_namespace_access_enum_variant() {
        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        typechecker
            .enums
            .insert("Color".to_string(), vec!["Red".to_string(), "Green".to_string()]);

        let node = ExpressionNode {
            expression: Expression::NamespaceAccess(
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("Color".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 0, end: 5 },
                }),
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("Green".to_string()),
                    typed: AzulaType::Infer,
                    span: Span { start: 7, end: 12 },
                }),
            ),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 12 },
        };

        let env = Environment::new();
        let (expr, typ) = typechecker.typecheck_expression(node, &env).unwrap();
        assert_eq!(typ, AzulaType::Named("Color".to_string()));
        assert_eq!(expr.typed, AzulaType::Named("Color".to_string()));
        assert!(typechecker.errors.is_empty());
    }

    #[test]
    fn test_match_expression() {
        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        typechecker
            .enums
            .insert("Color".to_string(), vec!["Red".to_string(), "Green".to_string()]);

        let mut env = Environment::new();
        env.add_variable(
            "c".to_string(),
            VariableDefinition {
                name: "c".to_string(),
                mutable: false,
                typ: AzulaType::Named("Color".to_string()),
            },
        );

        let node = ExpressionNode {
            expression: Expression::Match(
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("c".to_string()),
                    typed: AzulaType::Named("Color".to_string()),
                    span: Span { start: 0, end: 1 },
                }),
                vec![
                    (
                        MatchPattern::Variant("Color", "Red"),
                        ExpressionNode {
                            expression: Expression::Integer(1),
                            typed: AzulaType::Int,
                            span: Span { start: 0, end: 1 },
                        },
                    ),
                    (
                        MatchPattern::Variant("Color", "Green"),
                        ExpressionNode {
                            expression: Expression::Integer(2),
                            typed: AzulaType::Int,
                            span: Span { start: 0, end: 1 },
                        },
                    ),
                ],
            ),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 10 },
        };

        let (expr, typ) = typechecker.typecheck_expression(node, &env).unwrap();
        assert_eq!(typ, AzulaType::Int);
        assert_eq!(expr.typed, AzulaType::Int);
        assert!(typechecker.errors.is_empty());
    }

    #[test]
    fn test_match_non_exhaustive() {
        let mut typechecker = Typechecker::new(Statement::Root(vec![]));
        typechecker.enums.insert(
            "Color".to_string(),
            vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
        );

        let mut env = Environment::new();
        env.add_variable(
            "c".to_string(),
            VariableDefinition {
                name: "c".to_string(),
                mutable: false,
                typ: AzulaType::Named("Color".to_string()),
            },
        );

        let node = ExpressionNode {
            expression: Expression::Match(
                Rc::new(ExpressionNode {
                    expression: Expression::Identifier("c".to_string()),
                    typed: AzulaType::Named("Color".to_string()),
                    span: Span { start: 0, end: 1 },
                }),
                vec![(
                    MatchPattern::Variant("Color", "Red"),
                    ExpressionNode {
                        expression: Expression::Integer(1),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    },
                )],
            ),
            typed: AzulaType::Infer,
            span: Span { start: 0, end: 10 },
        };

        let result = typechecker.typecheck_expression(node, &env);
        assert!(result.is_err());
        assert!(matches!(
            typechecker.errors[0].error_type,
            ErrorType::NonExhaustiveMatch(_)
        ));
    }
}
