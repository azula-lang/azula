use std::{collections::HashMap, ops::Deref, rc::Rc};

use azula_ast::prelude::*;
use azula_error::prelude::*;
use azula_type::prelude::AzulaType;

pub struct Typechecker<'a> {
    ast: Statement<'a>,

    functions: HashMap<&'a str, FunctionDefinition<'a>>,
    globals: HashMap<String, VariableDefinition<'a>>,
    structs: HashMap<String, StructDefinition<'a>>,

    pub errors: Vec<AzulaError>,
}

struct FunctionDefinition<'a> {
    name: &'a str,
    args: Vec<(AzulaType<'a>, &'a str)>,
    varargs: bool,
    returns: AzulaType<'a>,
}

struct StructDefinition<'a> {
    name: &'a str,
    attrs: Vec<(AzulaType<'a>, &'a str)>,
}

#[derive(Debug)]
pub struct VariableDefinition<'a> {
    name: String,
    mutable: bool,
    typ: AzulaType<'a>,
}

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
                            .map(|(typ, name)| (AzulaType::from(typ.clone()), *name))
                            .collect();

                        let returns_converted: AzulaType = returns.clone().into();

                        self.functions.insert(
                            name,
                            FunctionDefinition {
                                name,
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
                        let args_converted: Vec<_> =
                            args.iter().map(|typ| (typ.clone(), "xyz")).collect();

                        let returns_converted: AzulaType = returns.clone().into();

                        self.functions.insert(
                            name,
                            FunctionDefinition {
                                name,
                                varargs: false,
                                args: args_converted.clone(),
                                returns: returns_converted.clone(),
                            },
                        );
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
            Statement::Reassign(..) => self.typecheck_reassign(stmt, env),
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
                let type_annotation = type_annotation.clone().unwrap();

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

                if type_annotation != typ {
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

            env.add_variable(
                name.clone(),
                VariableDefinition {
                    name: name.clone(),
                    mutable,
                    typ: typ,
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
                        return Err("unknown variable".to_string());
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
        if let Statement::If(ref expr, ref body, ref span) = stmt {
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

            Ok((Statement::If(expr, stmts, span.clone()), AzulaType::Void))
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
                    return Err("Unknown variable".to_string());
                }
            }
            Expression::FunctionCall { function, args } => {
                let return_type = match &function.expression {
                    Expression::Identifier(i) => match self.functions.get(&i.as_str()) {
                        Some(f) => &f.returns,
                        None => {
                            if i == "printf" || i == "sprintf" || i == "puts" {
                                &AzulaType::Void
                            } else {
                                self.errors.push(AzulaError::new(
                                    ErrorType::FunctionNotFound(i.to_string()),
                                    function.span.start,
                                    function.span.end,
                                ));
                                return Err("Function not found".to_string());
                            }
                        }
                    },
                    _ => todo!(),
                }
                .clone();

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
                            function: function.clone(),
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
        }
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

            let allowed = allowed.get(operator).unwrap();
            if !allowed.contains(&left_typ) {
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

            if !allowed.contains(&right_typ) {
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
}
