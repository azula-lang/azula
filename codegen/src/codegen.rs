use core::panic;
use std::{collections::HashMap, ops::Deref, rc::Rc};

use azula_ast::prelude::*;
use azula_ir::prelude::*;
use azula_type::prelude::AzulaType;

fn struct_byte_size<'a>(module: &Module<'a>, typ: &AzulaType<'a>) -> usize {
    match typ {
        AzulaType::Named(name) => {
            if let Some(s) = module.structs.get(name.as_str()) {
                s.attributes
                    .iter()
                    .map(|(t, _)| struct_byte_size(module, t))
                    .sum()
            } else {
                8
            }
        }
        _ => 8,
    }
}

fn is_terminator(instr: Option<&Instruction<'_>>) -> bool {
    matches!(
        instr,
        Some(Instruction::Return(_)) | Some(Instruction::Jump(_)) | Some(Instruction::Jcond(..))
    )
}

pub struct Codegen<'a> {
    root: Statement<'a>,

    pub module: Module<'a>,
    pub function_calls: HashMap<String, Vec<AzulaType<'a>>>,
    loop_end_stack: Vec<String>,
    loop_continue_stack: Vec<String>,
}

impl<'a> Codegen<'a> {
    pub fn new(name: &'a str, root: Statement<'a>) -> Self {
        Self {
            root,
            module: Module::new(name),
            function_calls: HashMap::new(),
            loop_end_stack: vec![],
            loop_continue_stack: vec![],
        }
    }

    pub fn codegen(&mut self) {
        let stmts = if let Statement::Root(stmts) = &self.root {
            stmts.clone()
        } else {
            return;
        };
        // Pass 1: register types, externs, impl blocks (so method signatures are
        // known before any function body is processed)
        for stmt in stmts.clone() {
            match stmt {
                Statement::ExternFunction {
                    name,
                    varargs,
                    args,
                    returns,
                    ..
                } => self.module.add_extern_function(
                    name,
                    ExternFunction {
                        varargs,
                        arguments: args,
                        returns: returns,
                    },
                ),
                Statement::Impl { .. } => self.codegen_impl(stmt.clone()),
                Statement::Assign(_, name, _, val, ..) => {
                    let value = match val.expression {
                        Expression::Integer(i) => GlobalValue::Int(i),
                        Expression::Float(f) => GlobalValue::Float(f),
                        Expression::Boolean(b) => GlobalValue::Bool(b),
                        Expression::String(s) => {
                            let ptr = match self.module.add_string(s) {
                                Value::Global(v) => v,
                                _ => unreachable!(),
                            };
                            GlobalValue::String(ptr)
                        }
                        _ => unreachable!(),
                    };
                    self.module.global_values.insert(name, value);
                }
                Statement::Struct {
                    name, attributes, ..
                } => {
                    self.module.add_struct(name, Struct { name, attributes });
                }
                Statement::Enum { name, variants, .. } => {
                    self.module.add_enum(
                        name.to_string(),
                        variants.iter().map(|v| v.to_string()).collect(),
                    );
                }
                _ => {}
            }
        }
        // Pass 2: codegen function bodies only
        for stmt in stmts {
            if let Statement::Function { .. } = stmt {
                self.codegen_function(stmt.clone());
            }
        }
    }

    pub fn insert_implicit_return(&mut self) {
        for (_, func) in self.module.functions.iter_mut() {
            let cloned = func.blocks.clone();
            for (index, (block_name, ref block)) in cloned.iter().enumerate() {
                let mut block = block.clone();
                if block.instructions.is_empty() {
                    block.instructions.push(Instruction::Return(None));
                    *func.blocks.get_mut(index).unwrap() = (block_name.to_string(), block.clone());
                    continue;
                }

                match block.instructions.last().unwrap() {
                    Instruction::Jcond(..) => continue,
                    Instruction::Jump(..) => continue,
                    Instruction::Return(..) => continue,
                    _ => {
                        block.instructions.push(Instruction::Return(None));
                        *func.blocks.get_mut(index).unwrap() = (block_name.clone(), block.clone());
                        continue;
                    }
                }
            }
        }
    }

    pub fn codegen_function(&mut self, stmt: Statement<'a>) {
        if let Statement::Function {
            name,
            args,
            returns,
            body,
            ..
        } = stmt
        {
            let mut arguments = vec![];
            for (typ, name) in args {
                arguments.push((name.to_string(), typ));
            }

            let mut function = Function::new();
            function.arguments = arguments;
            function.returns = returns;

            if let Statement::Block(stmts) = body.as_ref().clone() {
                for stmt in stmts {
                    self.codegen_statement(stmt, &mut function);
                }
            }

            self.module.add_function(name.to_string(), function)
        } else {
            unreachable!()
        }
    }

    pub fn codegen_impl(&mut self, stmt: Statement<'a>) {
        if let Statement::Impl {
            struct_impl,
            trait_impl: _,
            funcs,
            span: _,
        } = stmt
        {
            for func in funcs {
                match func {
                    Statement::Function {
                        name,
                        args,
                        returns,
                        body,
                        ..
                    } => {
                        let mut arguments = vec![];
                        for (typ, name) in args {
                            arguments.push((name.to_string(), typ));
                        }

                        let mut function = Function::new();
                        function.arguments = arguments;
                        function.returns = returns;

                        if let Statement::Block(stmts) = body.as_ref().clone() {
                            for stmt in stmts {
                                self.codegen_statement(stmt, &mut function);
                            }
                        }

                        let gen_name = format!("{}_{}", struct_impl.to_string(), name);

                        self.module.add_function(gen_name, function)
                    }
                    _ => unreachable!(),
                };
            }
        } else {
            unreachable!()
        }
    }

    pub fn codegen_statement(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        match stmt {
            Statement::Assign(..) => self.codegen_assign(stmt, func),
            Statement::Return(..) => self.codegen_return(stmt, func),
            Statement::ExpressionStatement(expr, ..) => {
                self.codegen_expr(expr.clone(), func, true);
            }
            Statement::If(..) => self.codegen_if(stmt, func),
            Statement::While(..) => self.codegen_while(stmt, func),
            Statement::For(..) => self.codegen_for(stmt, func),
            Statement::Break(..) => {
                let end = self.loop_end_stack.last().expect("break outside loop").clone();
                func.jump(end);
            }
            Statement::Continue(..) => {
                let cont = self.loop_continue_stack.last().expect("continue outside loop").clone();
                func.jump(cont);
            }
            Statement::Reassign(..) => self.codegen_reassign(stmt, func),
            Statement::Block(stmts) => {
                for s in stmts {
                    self.codegen_statement(s, func);
                }
            }
            _ => panic!(),
        }
    }

    pub fn codegen_assign(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::Assign(_, name, annotation, expr, _) = stmt {
            let value = self.codegen_expr(expr.clone(), func, true);
            let var_type = annotation.unwrap_or_else(|| expr.typed.clone());
            func.store(name.clone(), value, var_type.clone());
            func.variables.insert(name, var_type);
        } else {
            unreachable!()
        }
    }

    pub fn codegen_reassign(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::Reassign(var, val, _) = stmt {
            let value = self.codegen_expr(val.clone(), func, true);
            match var.expression {
                Expression::Identifier(v) => func.store(v.clone(), value, val.typed.clone()),
                Expression::ArrayAccess(array, index) => {
                    let elem_type = val.typed.clone();
                    let array = self.codegen_expr(array.deref().clone(), func, true);
                    let index = self.codegen_expr(index.deref().clone(), func, true);
                    func.store_element(array.clone(), index, value, elem_type);
                }
                Expression::StructAccess(struc, member) => {
                    // For heap pointers (&T), load the pointer value; for stack structs (T), take its address.
                    let already_ptr = matches!(struc.typed, AzulaType::Pointer(_));
                    let struc_val = self.codegen_expr(struc.deref().clone(), func, already_ptr);
                    let member_name = match &member.expression {
                        Expression::Identifier(v) => v,
                        _ => unreachable!(),
                    };
                    let struct_name = match &struc.typed {
                        AzulaType::Named(name) => name.clone(),
                        AzulaType::Pointer(nested) => match nested.deref().clone() {
                            AzulaType::Named(name) => name.clone(),
                            _ => unreachable!("{:?}", struc.typed),
                        },
                        _ => unreachable!("{:?}", struc.typed),
                    };

                    let struct_def = self.module.structs.get(struct_name.as_str()).unwrap();
                    let index = struct_def
                        .attributes
                        .iter()
                        .enumerate()
                        .find(|(_, (_, name))| name.to_string() == member_name.to_string())
                        .map(|(index, _)| index)
                        .unwrap();
                    func.store_struct_member(struc_val.clone(), index, value, struct_name)
                }
                _ => todo!(),
            }
        } else {
            unreachable!()
        }
    }

    pub fn codegen_return(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::Return(val, _) = stmt {
            match val {
                Some(expr) => {
                    let value = self.codegen_expr(expr, func, true);
                    func.ret(Some(value));
                }
                None => func.ret(None),
            }
        } else {
            unreachable!()
        }
    }

    pub fn codegen_if(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::If(cond, body, else_branch, ..) = stmt {
            let cond = self.codegen_expr(cond, func, true);

            let true_name = format!("true-{}", func.if_block_index);
            let else_name = format!("else-{}", func.if_block_index);
            let end_name = format!("end-{}", func.if_block_index);

            func.if_block_index += 1;

            let false_target = if else_branch.is_some() { else_name.clone() } else { end_name.clone() };
            func.jcond(cond, true_name.clone(), false_target);
            func.blocks.push((true_name.clone(), Block::new()));

            func.current_block = true_name.clone();

            for stmt in body {
                self.codegen_statement(stmt, func);
            }

            for (name, block) in &func.blocks.clone() {
                if name.clone() == func.current_block {
                    if !is_terminator(block.instructions.last()) {
                        func.jump(end_name.clone());
                    }
                }
            }

            if let Some(else_stmt) = else_branch {
                func.blocks.push((else_name.clone(), Block::new()));
                func.current_block = else_name.clone();
                self.codegen_statement(else_stmt.as_ref().clone(), func);

                for (name, block) in &func.blocks.clone() {
                    if name.clone() == func.current_block {
                        if !is_terminator(block.instructions.last()) {
                            func.jump(end_name.clone());
                        }
                    }
                }
            }

            func.blocks.push((end_name.clone(), Block::new()));
            func.current_block = end_name.clone();
        } else {
            unreachable!()
        }
    }

    pub fn codegen_while(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::While(cond, body, ..) = stmt {
            let eval_name = format!("eval-{}", func.if_block_index);
            let true_name = format!("loop-{}", func.if_block_index);
            let end_name = format!("end-{}", func.if_block_index);

            func.if_block_index += 1;

            self.loop_end_stack.push(end_name.clone());
            self.loop_continue_stack.push(eval_name.clone());

            func.jump(eval_name.clone());
            func.blocks.push((eval_name.clone(), Block::new()));
            func.current_block = eval_name.clone();
            let cond_val = self.codegen_expr(cond.clone(), func, true);
            func.jcond(cond_val, true_name.clone(), end_name.clone());

            func.blocks.push((true_name.clone(), Block::new()));
            func.current_block = true_name.clone();

            for stmt in body {
                self.codegen_statement(stmt, func);
            }
            func.jump(eval_name.clone());

            self.loop_end_stack.pop();
            self.loop_continue_stack.pop();

            func.blocks.push((end_name.clone(), Block::new()));
            func.current_block = end_name.clone();
        } else {
            unreachable!()
        }
    }

    pub fn codegen_for(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::For(cond, body, ..) = stmt {
            let loop_name = format!("loop-{}", func.if_block_index);
            let end_name = format!("end-{}", func.if_block_index);

            self.loop_end_stack.push(end_name.clone());

            match cond {
                Some(cond) => {
                    let eval_name = format!("eval-{}", func.if_block_index);
                    func.if_block_index += 1;

                    // continue → jump back to eval (re-check condition)
                    self.loop_continue_stack.push(eval_name.clone());

                    func.jump(eval_name.clone());
                    func.blocks.push((eval_name.clone(), Block::new()));
                    func.current_block = eval_name.clone();
                    let cond_val = self.codegen_expr(cond, func, true);
                    func.jcond(cond_val, loop_name.clone(), end_name.clone());

                    func.blocks.push((loop_name.clone(), Block::new()));
                    func.current_block = loop_name.clone();
                    for s in body {
                        self.codegen_statement(s, func);
                    }
                    let last = func.blocks.iter()
                        .find(|(n, _)| n == &func.current_block)
                        .and_then(|(_, b)| b.instructions.last());
                    if !is_terminator(last) {
                        func.jump(eval_name.clone());
                    }
                    self.loop_continue_stack.pop();
                }
                None => {
                    func.if_block_index += 1;

                    // continue → jump back to loop top
                    self.loop_continue_stack.push(loop_name.clone());

                    func.jump(loop_name.clone());
                    func.blocks.push((loop_name.clone(), Block::new()));
                    func.current_block = loop_name.clone();
                    for s in body {
                        self.codegen_statement(s, func);
                    }
                    let last = func.blocks.iter()
                        .find(|(n, _)| n == &func.current_block)
                        .and_then(|(_, b)| b.instructions.last());
                    if !is_terminator(last) {
                        func.jump(loop_name.clone());
                    }
                    self.loop_continue_stack.pop();
                }
            }

            self.loop_end_stack.pop();

            func.blocks.push((end_name.clone(), Block::new()));
            func.current_block = end_name.clone();
        } else {
            unreachable!()
        }
    }

    pub fn codegen_expr(
        &mut self,
        expr: ExpressionNode<'a>,
        func: &mut Function<'a>,
        resolve_pointer: bool,
    ) -> Value {
        match expr.expression {
            Expression::Infix(..) => self.codegen_infix(expr, func, resolve_pointer),
            Expression::Integer(val) => func.const_int(val),
            Expression::Float(val) => func.const_float(val),
            Expression::Identifier(name) if resolve_pointer => {
                if let Some((index, _)) = func
                    .arguments
                    .iter()
                    .enumerate()
                    .map(|(index, (name, _))| (index, name))
                    .filter(|(_, n)| n.clone().clone() == name)
                    .next()
                {
                    func.load_arg(index, expr.typed)
                } else if func.variables.contains_key(&name) {
                    func.load(name, expr.typed)
                } else if let Some(val) = self.module.global_values.get(&name) {
                    if let GlobalValue::String(v) = val {
                        return Value::Global(*v);
                    }
                    func.load_global(name, expr.typed)
                } else if name == "nil" {
                    func.const_null()
                } else {
                    unreachable!()
                }
            }
            Expression::Identifier(name) => {
                if let Some((index, _)) = func
                    .arguments
                    .iter()
                    .enumerate()
                    .map(|(index, (name, _))| (index, name))
                    .filter(|(_, n)| n.clone().clone() == name)
                    .next()
                {
                    func.load_arg(index, expr.typed.clone())
                } else {
                    func.ptr(name)
                }
            }
            Expression::String(val) => self.module.add_string(val),
            Expression::Boolean(val) => {
                if val {
                    func.const_true()
                } else {
                    func.const_false()
                }
            }
            Expression::FunctionCall { function, mut args } => {
                let name = self.resolve_function(function.deref().clone());

                // if name == "__array_len" {
                //     match args[0].typed {
                //         AzulaType::Array(_, size) => return func.const_int(size.unwrap() as i64),
                //         _ => unreachable!("{:?}", args[0].typed),
                //     }
                // }

                if let Expression::StructAccess(left, right) = &function.expression {
                    // Check if the method's first parameter is a pointer (mutating self)
                    let receiver_type = left.typed.to_string();
                    let method_name = if let Expression::Identifier(m) = &right.expression { m.clone() } else { String::new() };
                    let mangled = format!("{}_{}", receiver_type, method_name);
                    let self_is_ptr = self.module.functions.get(&mangled)
                        .and_then(|f| f.arguments.first())
                        .map(|(_, t)| matches!(t, AzulaType::Pointer(_)))
                        .unwrap_or(false);

                    if self_is_ptr && !matches!(left.typed, AzulaType::Pointer(_)) {
                        // Receiver is a value type — take its address
                        args.insert(0, ExpressionNode {
                            expression: Expression::Pointer(left.clone()),
                            typed: AzulaType::Pointer(Rc::new(left.typed.clone())),
                            span: left.span.clone(),
                        });
                    } else {
                        // Either pass-by-value or already a pointer — pass directly
                        args.insert(0, left.as_ref().clone());
                    }
                }

                let args = args
                    .iter()
                    .map(|arg| self.codegen_expr(arg.clone(), func, true))
                    .collect();

                func.function_call(name.clone(), args)
            }
            Expression::Not(expr) => {
                let val = self.codegen_expr(expr.as_ref().clone(), func, true);

                func.not(val)
            }
            Expression::Negate(expr) => {
                let inner = expr.as_ref().clone();
                let zero = match inner.typed {
                    AzulaType::Float => func.const_float(0.0),
                    _ => func.const_int(0),
                };
                let val = self.codegen_expr(inner, func, true);
                func.sub(zero, val)
            }
            Expression::Pointer(expr) => {
                //     match &expr.expression {
                //     Expression::Identifier(ident) => func.ptr(ident.clone()),
                //     _ => unreachable!(),
                // }

                self.codegen_expr(expr.deref().clone(), func, false)
            }
            Expression::Array(vals) => {
                let elem_type = vals[0].typed.clone();
                let array = func.create_array(elem_type.clone(), vals.len());

                for (index, val) in vals.iter().enumerate() {
                    let gened = self.codegen_expr(val.clone(), func, true);
                    let index = func.const_int(index as i64);
                    func.store_element(array.clone(), index, gened, elem_type.clone());
                }

                return array;
            }
            Expression::ArrayAccess(array, index) => {
                let elem_type = match &array.typed {
                    AzulaType::Array(inner, _) => inner.as_ref().clone(),
                    AzulaType::Str => AzulaType::SizedSignedInt(8),
                    AzulaType::Pointer(inner) => inner.as_ref().clone(),
                    _ => unreachable!("array access on non-array type"),
                };
                let array = self.codegen_expr(array.deref().clone(), func, true);
                let index = self.codegen_expr(index.deref().clone(), func, true);

                func.access_element(array, index, elem_type)
            }
            Expression::StructInitialisation(struc, vals) => {
                let values: Vec<_> = vals
                    .iter()
                    .map(|(_, v)| self.codegen_expr(v.deref().clone(), func, true))
                    .collect();

                let name = match &struc.expression {
                    Expression::Identifier(s) => s,
                    _ => unreachable!(),
                };

                func.create_struct(name.clone(), values)
            }
            Expression::StructAccess(struc, member) => {
                let struct_value = self.codegen_expr(struc.deref().clone(), func, true);

                let member_name = match &member.expression {
                    Expression::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };

                let struct_name = match &struc.typed {
                    AzulaType::Named(name) => name.clone(),
                    AzulaType::Pointer(nested) => match nested.deref().clone() {
                        AzulaType::Named(name) => name.clone(),
                        _ => unreachable!("{:?}", struc.typed),
                    },
                    _ => unreachable!("{:?}", struc.typed),
                };

                let struct_def = self.module.structs.get(struct_name.as_str()).unwrap();
                let index = struct_def
                    .attributes
                    .iter()
                    .enumerate()
                    .find(|(_, (_, name))| name.to_string() == member_name)
                    .map(|(index, _)| index)
                    .unwrap();

                func.access_struct_member(struct_value, index, resolve_pointer, struct_name)
            }
            Expression::NamespaceAccess(ns, variant) => {
                let enum_name = match &ns.expression {
                    Expression::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                let variant_name = match &variant.expression {
                    Expression::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                let variants = self.module.enums.get(&enum_name)
                    .unwrap_or_else(|| panic!("Unknown enum {}", enum_name));
                let index = variants.iter().position(|v| *v == variant_name)
                    .unwrap_or_else(|| panic!("Unknown variant {} on {}", variant_name, enum_name));
                func.const_int(index as i64)
            }
            Expression::Match(scrutinee, arms) => {
                self.codegen_match(scrutinee, arms, expr.typed, func)
            }
            Expression::Cast(inner, target_type) => {
                let val = self.codegen_expr(inner.as_ref().clone(), func, true);
                func.cast(val, target_type)
            }
            Expression::Null => func.const_null(),
            Expression::Block(stmts, final_expr) => {
                for stmt in stmts {
                    self.codegen_statement(stmt, func);
                }
                match final_expr {
                    Some(fe) => self.codegen_expr(fe.as_ref().clone(), func, resolve_pointer),
                    None => Value::LiteralInteger(0),
                }
            }
            Expression::Alloc(inner) => {
                if let Expression::StructInitialisation(struc, vals) = &inner.expression {
                    let struct_name = match &struc.expression {
                        Expression::Identifier(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    // Compute field values before malloc so they don't alias the pointer
                    let field_vals: Vec<_> = vals
                        .iter()
                        .map(|(_, v)| self.codegen_expr(v.clone(), func, true))
                        .collect();
                    // Compute the true byte size by summing field sizes (struct fields may be >8 bytes)
                    let size_bytes: usize = if let Some(struc_def) = self.module.structs.get(struct_name.as_str()) {
                        struc_def.attributes.iter().map(|(t, _)| struct_byte_size(&self.module, t)).sum()
                    } else {
                        vals.len() * 8
                    };
                    let size = func.const_int(size_bytes as i64);
                    let ptr = func.function_call("malloc".to_string(), vec![size]);
                    // Store each field through the pointer
                    for (i, val) in field_vals.into_iter().enumerate() {
                        func.store_struct_member(ptr.clone(), i, val, struct_name.clone());
                    }
                    ptr
                } else {
                    unreachable!("alloc requires a struct initialisation expression")
                }
            }
        }
    }

    fn codegen_match(
        &mut self,
        scrutinee: Rc<ExpressionNode<'a>>,
        arms: Vec<(MatchPattern<'a>, ExpressionNode<'a>)>,
        result_type: AzulaType<'a>,
        func: &mut Function<'a>,
    ) -> Value {
        let n = func.match_block_index;
        func.match_block_index += 1;

        let scrut_val = self.codegen_expr(scrutinee.deref().clone(), func, true);

        let is_void = result_type == AzulaType::Void;
        let result_var = format!("__match_{}", n);
        if !is_void {
            func.variables.insert(result_var.clone(), result_type.clone());
        }

        let end_block = format!("match-end-{}", n);

        let arm_count = arms.len();
        for (i, (pattern, body)) in arms.into_iter().enumerate() {
            let arm_block = format!("match-arm-{}-{}", n, i);
            let next_block = if i + 1 < arm_count {
                format!("match-next-{}-{}", n, i + 1)
            } else {
                end_block.clone()
            };

            match pattern {
                MatchPattern::Wildcard => {
                    // No condition — fall through directly to arm block
                    func.jump(arm_block.clone());
                    func.blocks.push((arm_block.clone(), Block::new()));
                    func.current_block = arm_block.clone();
                    let body_val = self.codegen_expr(body, func, true);
                    if !is_void {
                        func.store(result_var.clone(), body_val, result_type.clone());
                    }
                    func.jump(end_block.clone());
                }
                MatchPattern::Integer(n) => {
                    let idx_val = func.const_int(n);
                    let cond = func.eq(scrut_val.clone(), idx_val);
                    func.jcond(cond, arm_block.clone(), next_block.clone());

                    func.blocks.push((arm_block.clone(), Block::new()));
                    func.current_block = arm_block.clone();
                    let body_val = self.codegen_expr(body, func, true);
                    if !is_void {
                        func.store(result_var.clone(), body_val, result_type.clone());
                    }
                    func.jump(end_block.clone());

                    if i + 1 < arm_count {
                        func.blocks.push((next_block.clone(), Block::new()));
                        func.current_block = next_block.clone();
                    }
                }
                MatchPattern::Variant(enum_name, variant_name) => {
                    let variants = self.module.enums.get(enum_name)
                        .unwrap_or_else(|| panic!("Unknown enum {}", enum_name));
                    let index = variants.iter().position(|v| v == variant_name)
                        .unwrap_or_else(|| panic!("Unknown variant {}", variant_name)) as i64;
                    let idx_val = func.const_int(index);
                    let cond = func.eq(scrut_val.clone(), idx_val);
                    func.jcond(cond, arm_block.clone(), next_block.clone());

                    func.blocks.push((arm_block.clone(), Block::new()));
                    func.current_block = arm_block.clone();
                    let body_val = self.codegen_expr(body, func, true);
                    if !is_void {
                        func.store(result_var.clone(), body_val, result_type.clone());
                    }
                    func.jump(end_block.clone());

                    if i + 1 < arm_count {
                        func.blocks.push((next_block.clone(), Block::new()));
                        func.current_block = next_block.clone();
                    }
                }
            }
        }

        func.blocks.push((end_block.clone(), Block::new()));
        func.current_block = end_block.clone();

        if is_void {
            Value::LiteralInteger(0)
        } else {
            func.load(result_var, result_type)
        }
    }

    pub fn codegen_infix(
        &mut self,
        expr: ExpressionNode<'a>,
        func: &mut Function<'a>,
        _: bool,
    ) -> Value {
        if let Expression::Infix(val1, op, val2) = expr.expression {
            match op {
                Operator::Add => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.add(val1, val2)
                }
                Operator::Sub => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.sub(val1, val2)
                }
                Operator::Mul => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.mul(val1, val2)
                }
                Operator::Div => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.div(val1, val2)
                }
                Operator::Mod => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.modulus(val1, val2)
                }
                Operator::Power => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.pow(val1, val2)
                }
                Operator::Or => {
                    // Short-circuit: if LHS is true, skip RHS
                    let result_name = format!("__sc_{}", func.if_block_index);
                    let rhs_block = format!("sc_rhs_{}", func.if_block_index);
                    let end_block = format!("sc_end_{}", func.if_block_index);
                    func.if_block_index += 1;

                    func.variables.insert(result_name.clone(), AzulaType::Bool);
                    let t = func.const_true();
                    func.store(result_name.clone(), t, AzulaType::Bool);

                    let lhs = self.codegen_expr(val1.as_ref().clone(), func, true);
                    func.jcond(lhs, end_block.clone(), rhs_block.clone());

                    func.blocks.push((rhs_block.clone(), Block::new()));
                    func.current_block = rhs_block.clone();
                    let rhs = self.codegen_expr(val2.as_ref().clone(), func, true);
                    func.store(result_name.clone(), rhs, AzulaType::Bool);
                    func.jump(end_block.clone());

                    func.blocks.push((end_block.clone(), Block::new()));
                    func.current_block = end_block.clone();
                    func.load(result_name, AzulaType::Bool)
                }
                Operator::And => {
                    // Short-circuit: if LHS is false, skip RHS
                    let result_name = format!("__sc_{}", func.if_block_index);
                    let rhs_block = format!("sc_rhs_{}", func.if_block_index);
                    let end_block = format!("sc_end_{}", func.if_block_index);
                    func.if_block_index += 1;

                    func.variables.insert(result_name.clone(), AzulaType::Bool);
                    let f = func.const_false();
                    func.store(result_name.clone(), f, AzulaType::Bool);

                    let lhs = self.codegen_expr(val1.as_ref().clone(), func, true);
                    func.jcond(lhs, rhs_block.clone(), end_block.clone());

                    func.blocks.push((rhs_block.clone(), Block::new()));
                    func.current_block = rhs_block.clone();
                    let rhs = self.codegen_expr(val2.as_ref().clone(), func, true);
                    func.store(result_name.clone(), rhs, AzulaType::Bool);
                    func.jump(end_block.clone());

                    func.blocks.push((end_block.clone(), Block::new()));
                    func.current_block = end_block.clone();
                    func.load(result_name, AzulaType::Bool)
                }
                Operator::Eq => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.eq(val1, val2)
                }
                Operator::Neq => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.neq(val1, val2)
                }
                Operator::Lt => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.lt(val1, val2)
                }
                Operator::Lte => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.lte(val1, val2)
                }
                Operator::Gt => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.gt(val1, val2)
                }
                Operator::Gte => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func, true);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func, true);

                    func.gte(val1, val2)
                }
            }
        } else {
            unreachable!()
        }
    }

    fn resolve_function(&self, func: ExpressionNode<'a>) -> String {
        match func.expression {
            Expression::Identifier(name) => name,
            Expression::NamespaceAccess(ns, func) => {
                let namespace = if let Expression::Identifier(s) = ns.deref().clone().expression {
                    s
                } else {
                    unreachable!()
                };

                let func = if let Expression::Identifier(func) = &func.expression {
                    func
                } else {
                    unreachable!()
                };

                format!("{}_{}", namespace, func)
            }
            Expression::StructAccess(left, right) => {
                let namespace = left.deref().clone().typed.to_string();

                let func = if let Expression::Identifier(func) = &right.expression {
                    func
                } else {
                    unreachable!()
                };

                format!("{}_{}", namespace, func)
            }
            _ => unreachable!("{:?}", func.expression),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use azula_ast::prelude::Span;
    use azula_type::prelude::*;
    use std::rc::Rc;

    #[test]
    fn test_codegen_function() {
        let mut codegen = Codegen::new("test", Statement::Root(vec![]));

        codegen.codegen_function(Statement::Function {
            name: "test",
            args: vec![(AzulaType::Bool, "x")],
            returns: AzulaType::Int,
            body: Rc::new(Statement::Block(vec![])),
            span: Span { start: 0, end: 1 },
        });

        assert_eq!(codegen.module.functions.len(), 1);
        let function = codegen.module.functions.get("test").unwrap();
        let mut args = vec![];
        args.push(("x".to_string(), AzulaType::Bool));
        assert_eq!(function.arguments, args);
        assert_eq!(function.returns, AzulaType::Int);
    }

    #[test]
    fn test_codegen_consts() {
        let mut codegen = Codegen::new("test", Statement::Root(vec![]));

        // Integer
        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
                expression: Expression::Integer(5),
                typed: AzulaType::Int,
                span: Span { start: 0, end: 1 },
            },
            &mut func,
            true,
        );
        assert_eq!(
            func.blocks[0].1.instructions,
            vec![Instruction::ConstInt(5, 0)]
        );

        // True
        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
                expression: Expression::Boolean(true),
                typed: AzulaType::Int,
                span: Span { start: 0, end: 1 },
            },
            &mut func,
            true,
        );
        assert_eq!(
            func.blocks[0].1.instructions,
            vec![Instruction::ConstTrue(0)]
        );

        // False
        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
                expression: Expression::Boolean(false),
                typed: AzulaType::Int,
                span: Span { start: 0, end: 1 },
            },
            &mut func,
            true,
        );
        assert_eq!(
            func.blocks[0].1.instructions,
            vec![Instruction::ConstFalse(0)]
        );
    }

    #[test]
    fn test_codegen_if() {
        let mut codegen = Codegen::new("test", Statement::Root(vec![]));
        let mut func = Function::new();

        codegen.codegen_if(
            Statement::If(
                ExpressionNode {
                    expression: Expression::Boolean(true),
                    typed: AzulaType::Bool,
                    span: Span { start: 0, end: 0 },
                },
                vec![],
                None,
                Span { start: 0, end: 0 },
            ),
            &mut func,
        );

        assert_eq!(func.blocks.len(), 3);
    }

    #[test]
    fn test_codegen_infix() {
        let mut codegen = Codegen::new("test", Statement::Root(vec![]));

        // Addition
        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
                expression: Expression::Infix(
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(10),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                    Operator::Add,
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(20),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                ),
                typed: AzulaType::Int,
                span: Span { start: 0, end: 1 },
            },
            &mut func,
            true,
        );
        assert_eq!(
            func.blocks[0].1.instructions,
            vec![
                Instruction::ConstInt(10, 0),
                Instruction::ConstInt(20, 1),
                Instruction::Add(Value::Local(0), Value::Local(1), 2)
            ]
        );

        // Subtraction
        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
                expression: Expression::Infix(
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(10),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                    Operator::Sub,
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(20),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                ),
                typed: AzulaType::Int,
                span: Span { start: 0, end: 1 },
            },
            &mut func,
            true,
        );
        assert_eq!(
            func.blocks[0].1.instructions,
            vec![
                Instruction::ConstInt(10, 0),
                Instruction::ConstInt(20, 1),
                Instruction::Sub(Value::Local(0), Value::Local(1), 2)
            ]
        );

        // Multiplication
        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
                expression: Expression::Infix(
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(10),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                    Operator::Mul,
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(20),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                ),
                typed: AzulaType::Int,
                span: Span { start: 0, end: 1 },
            },
            &mut func,
            true,
        );
        assert_eq!(
            func.blocks[0].1.instructions,
            vec![
                Instruction::ConstInt(10, 0),
                Instruction::ConstInt(20, 1),
                Instruction::Mul(Value::Local(0), Value::Local(1), 2)
            ]
        );

        // Divide
        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
                expression: Expression::Infix(
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(10),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                    Operator::Div,
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(20),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                ),
                typed: AzulaType::Int,
                span: Span { start: 0, end: 1 },
            },
            &mut func,
            true,
        );
        assert_eq!(
            func.blocks[0].1.instructions,
            vec![
                Instruction::ConstInt(10, 0),
                Instruction::ConstInt(20, 1),
                Instruction::Div(Value::Local(0), Value::Local(1), 2)
            ]
        );

        // Modulus
        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
                expression: Expression::Infix(
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(10),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                    Operator::Mod,
                    Rc::new(ExpressionNode {
                        expression: Expression::Integer(20),
                        typed: AzulaType::Int,
                        span: Span { start: 0, end: 1 },
                    }),
                ),
                typed: AzulaType::Int,
                span: Span { start: 0, end: 1 },
            },
            &mut func,
            true,
        );
        assert_eq!(
            func.blocks[0].1.instructions,
            vec![
                Instruction::ConstInt(10, 0),
                Instruction::ConstInt(20, 1),
                Instruction::Mod(Value::Local(0), Value::Local(1), 2)
            ]
        );
    }

    #[test]
    fn test_codegen_enum_registration() {
        let root = Statement::Root(vec![Statement::Enum {
            name: "Color",
            variants: vec!["Red", "Green", "Blue"],
            span: Span { start: 0, end: 1 },
        }]);
        let mut codegen = Codegen::new("test", root);
        codegen.codegen();

        assert!(codegen.module.enums.contains_key("Color"));
        assert_eq!(
            codegen.module.enums["Color"],
            vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()]
        );
    }

    #[test]
    fn test_codegen_enum_variant_is_const_int() {
        let mut codegen = Codegen::new("test", Statement::Root(vec![]));
        codegen
            .module
            .add_enum("Color".to_string(), vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()]);

        let mut func = Function::new();
        codegen.codegen_expr(
            ExpressionNode {
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
                typed: AzulaType::Named("Color".to_string()),
                span: Span { start: 0, end: 12 },
            },
            &mut func,
            true,
        );
        // Green is index 1
        assert_eq!(func.blocks[0].1.instructions, vec![Instruction::ConstInt(1, 0)]);
    }

    #[test]
    fn test_codegen_match_block_count() {
        let mut codegen = Codegen::new("test", Statement::Root(vec![]));
        codegen
            .module
            .add_enum("Color".to_string(), vec!["Red".to_string(), "Green".to_string()]);

        let mut func = Function::new();
        // Manually add scrutinee as a stored variable
        let scrut = func.const_int(0);
        func.variables.insert("__scrut".to_string(), AzulaType::Named("Color".to_string()));
        func.store("__scrut".to_string(), scrut, AzulaType::Named("Color".to_string()));
        let scrut_loaded = func.load("__scrut".to_string(), AzulaType::Named("Color".to_string()));

        let arms = vec![
            (
                MatchPattern::Variant("Color", "Red"),
                ExpressionNode {
                    expression: Expression::Integer(10),
                    typed: AzulaType::Int,
                    span: Span { start: 0, end: 1 },
                },
            ),
            (
                MatchPattern::Variant("Color", "Green"),
                ExpressionNode {
                    expression: Expression::Integer(20),
                    typed: AzulaType::Int,
                    span: Span { start: 0, end: 1 },
                },
            ),
        ];

        codegen.codegen_match(
            Rc::new(ExpressionNode {
                expression: Expression::Identifier("__scrut".to_string()),
                typed: AzulaType::Named("Color".to_string()),
                span: Span { start: 0, end: 1 },
            }),
            arms,
            AzulaType::Int,
            &mut func,
        );

        // entry + match-arm-0-0 + match-next-0-1 + match-arm-0-1 + match-end-0 = 5 blocks
        assert_eq!(func.blocks.len(), 5);
        assert_eq!(func.match_block_index, 1);
    }
}
