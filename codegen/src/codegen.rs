use core::panic;

use azula_ast::prelude::*;
use azula_ir::prelude::*;

pub struct Codegen<'a> {
    root: Statement<'a>,

    pub module: Module<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new(name: &'a str, root: Statement<'a>) -> Self {
        Self {
            root,
            module: Module::new(name),
        }
    }

    pub fn codegen(&mut self) {
        if let Statement::Root(stmts) = &self.root {
            for stmt in stmts.clone() {
                match stmt {
                    Statement::Function { .. } => self.codegen_function(stmt.clone()),
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
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn insert_implicit_return(&mut self) {
        for (name, func) in self.module.functions.iter_mut() {
            let cloned = func.blocks.clone();
            for (index, (block_name, ref block)) in cloned.iter().enumerate() {
                let mut block = block.clone();
                if block.instructions.is_empty() {
                    block.instructions.push(Instruction::Return(None));
                    *func.blocks.get_mut(index).unwrap() = (name.to_string(), block.clone());
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

            self.module.add_function(name, function)
        } else {
            unreachable!()
        }
    }

    pub fn codegen_statement(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        match stmt {
            Statement::Assign(..) => self.codegen_assign(stmt, func),
            Statement::Return(..) => self.codegen_return(stmt, func),
            Statement::ExpressionStatement(expr, ..) => {
                self.codegen_expr(expr.clone(), func);
            }
            Statement::If(..) => self.codegen_if(stmt, func),
            _ => panic!(),
        }
    }

    pub fn codegen_assign(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::Assign(_, name, _, expr, _) = stmt {
            let value = self.codegen_expr(expr.clone(), func);
            func.store(name.clone(), value, expr.typed.clone());
            func.variables.insert(name, expr.typed);
        } else {
            unreachable!()
        }
    }

    pub fn codegen_return(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::Return(val, _) = stmt {
            match val {
                Some(expr) => {
                    let value = self.codegen_expr(expr, func);
                    func.ret(Some(value));
                }
                None => func.ret(None),
            }
        } else {
            unreachable!()
        }
    }

    pub fn codegen_if(&mut self, stmt: Statement<'a>, func: &mut Function<'a>) {
        if let Statement::If(cond, body, ..) = stmt {
            let cond = self.codegen_expr(cond, func);

            let true_name = format!("true-{}", func.if_block_index);
            let end_name = format!("end-{}", func.if_block_index);

            func.if_block_index += 1;

            func.jcond(cond, true_name.clone(), end_name.clone());
            func.blocks.push((true_name.clone(), Block::new()));

            func.current_block = true_name.clone();

            for stmt in body {
                self.codegen_statement(stmt, func);
            }

            func.blocks.push((end_name.clone(), Block::new()));
            func.current_block = end_name.clone();
        } else {
            unreachable!()
        }
    }

    pub fn codegen_expr(&mut self, expr: ExpressionNode<'a>, func: &mut Function<'a>) -> Value {
        match expr.expression {
            Expression::Infix(..) => self.codegen_infix(expr, func),
            Expression::Integer(val) => func.const_int(val),
            Expression::Float(val) => func.const_float(val),
            Expression::Identifier(name) => {
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
            Expression::String(val) => self.module.add_string(val),
            Expression::Boolean(val) => {
                if val {
                    func.const_true()
                } else {
                    func.const_false()
                }
            }
            Expression::FunctionCall { function, args } => {
                let name = match &function.expression {
                    Expression::Identifier(name) => name,
                    _ => todo!(),
                };

                let args = args
                    .iter()
                    .map(|arg| self.codegen_expr(arg.clone(), func))
                    .collect();
                func.function_call(name.clone(), args)
            }
            Expression::Not(expr) => {
                let val = self.codegen_expr(expr.as_ref().clone(), func);

                func.not(val)
            }
            Expression::Pointer(expr) => match &expr.expression {
                Expression::Identifier(ident) => func.ptr(ident.clone()),
                _ => unreachable!(),
            },
        }
    }

    pub fn codegen_infix(&mut self, expr: ExpressionNode<'a>, func: &mut Function<'a>) -> Value {
        if let Expression::Infix(val1, op, val2) = expr.expression {
            match op {
                Operator::Add => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.add(val1, val2)
                }
                Operator::Sub => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.sub(val1, val2)
                }
                Operator::Mul => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.mul(val1, val2)
                }
                Operator::Div => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.div(val1, val2)
                }
                Operator::Mod => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.modulus(val1, val2)
                }
                Operator::Power => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.pow(val1, val2)
                }
                Operator::Or => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.or(val1, val2)
                }
                Operator::And => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.and(val1, val2)
                }
                Operator::Eq => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.eq(val1, val2)
                }
                Operator::Neq => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.neq(val1, val2)
                }
                Operator::Lt => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.lt(val1, val2)
                }
                Operator::Lte => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.lte(val1, val2)
                }
                Operator::Gt => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.gt(val1, val2)
                }
                Operator::Gte => {
                    let val1 = self.codegen_expr(val1.as_ref().clone(), func);
                    let val2 = self.codegen_expr(val2.as_ref().clone(), func);

                    func.gte(val1, val2)
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
}
