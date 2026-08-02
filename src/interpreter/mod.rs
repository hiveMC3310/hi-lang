//! Interpreter for the Hi language, executes AST.

use crate::ast::{BinOp, Block, Expr, Program, Span, Stmt, UnOp};
use crate::error::{InterpError, InterpResult};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct Environment {
    pub parent: Option<Box<Environment>>,
    pub vars: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            parent: None,
            vars: HashMap::new(),
        }
    }

    pub fn child(&self) -> Self {
        Environment {
            parent: Some(Box::new(self.clone())),
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.get(name) {
            Some(v.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        if self.vars.contains_key(&name) {
            self.vars.insert(name, value);
        } else if let Some(parent) = &mut self.parent {
            parent.set(name, value);
        } else {
            self.vars.insert(name, value);
        }
    }
}

pub struct Interpreter {
    pub env: Environment,
    pub functions: HashMap<String, (Vec<String>, Block)>,
    pub return_value: Option<Value>,
    pub break_flag: bool,
    pub loop_depth: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            functions: HashMap::new(),
            return_value: None,
            break_flag: false,
            loop_depth: 0,
        }
    }

    /// Entry point: execute the program.
    pub fn run(&mut self, program: &Program) -> InterpResult<Option<Value>> {
        for stmt in &program.stmts {
            self.execute_stmt(stmt)?;
            if self.return_value.is_some() || self.break_flag {
                break;
            }
        }
        Ok(self.return_value.take())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> InterpResult<()> {
        match stmt {
            Stmt::Let(name, expr, _) => {
                let val = self.eval_expr(expr)?;
                self.env.set(name.clone(), val);
                Ok(())
            }
            Stmt::Assign(left, right, _) => {
                let value = self.eval_expr(right)?;
                match **left {
                    Expr::Variable(ref name, _) => {
                        self.env.set(name.clone(), value);
                    }
                    Expr::Index(ref base, ref index, span) => {
                        let base_value = self.eval_expr(base)?;
                        let index_value = self.eval_expr(index)?;
                        self.assign_index(base_value, index_value, value, span)?;
                    }
                    _ => {
                        return Err(InterpError::Runtime {
                            span: left.span(),
                            message: "Invalid left-hand side in assignment".to_string(),
                        });
                    }
                }
                Ok(())
            }
            Stmt::If(cond, then_block, else_block, _) => {
                if self.eval_expr(cond)?.as_bool() {
                    for stmt in then_block {
                        self.execute_stmt(stmt)?;
                        if self.return_value.is_some() || self.break_flag {
                            break;
                        }
                    }
                } else if let Some(else_block) = else_block {
                    for stmt in else_block {
                        self.execute_stmt(stmt)?;
                        if self.return_value.is_some() || self.break_flag {
                            break;
                        }
                    }
                }
                Ok(())
            }
            Stmt::While(cond, body, _) => {
                self.loop_depth += 1;
                while self.eval_expr(cond)?.as_bool() {
                    for stmt in body {
                        self.execute_stmt(stmt)?;
                        if self.return_value.is_some() {
                            break;
                        }
                        if self.break_flag {
                            self.break_flag = false;
                            break;
                        }
                    }
                    if self.return_value.is_some() || self.break_flag {
                        break;
                    }
                }
                self.loop_depth -= 1;
                Ok(())
            }
            Stmt::Break(span) => {
                if self.loop_depth == 0 {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "BREAK used outside of a loop".to_string(),
                    });
                }
                self.break_flag = true;
                Ok(())
            }
            Stmt::Func(name, params, body, _) => {
                self.functions
                    .insert(name.clone(), (params.clone(), body.clone()));
                Ok(())
            }
            Stmt::Return(expr, _) => {
                let val = if let Some(e) = expr {
                    self.eval_expr(e)?
                } else {
                    Value::Nil
                };
                self.return_value = Some(val);
                Ok(())
            }
            Stmt::Print(args, _) => {
                let mut output = String::new();
                for expr in args {
                    let val = self.eval_expr(expr)?;
                    output.push_str(&val.to_string());
                }
                println!("{}", output);
                Ok(())
            }
            Stmt::Expr(expr, _) => {
                self.eval_expr(expr)?;
                Ok(())
            }
        }
    }

    // ---- Expression evaluation ----
    fn eval_expr(&mut self, expr: &Expr) -> InterpResult<Value> {
        match expr {
            Expr::Int(i, _) => Ok(Value::Int(*i)),
            Expr::Float(f, _) => Ok(Value::Float(*f)),
            Expr::String(s, _) => Ok(Value::String(s.clone())),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Variable(name, span) => self.env.get(name).ok_or_else(|| InterpError::Runtime {
                span: *span,
                message: format!("Undefined variable '{}'", name),
            }),
            Expr::Binary(op, left, right, span) => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                Self::evaluate_binary_op(*op, &left_val, &right_val, span)
            }
            Expr::Unary(op, expr, _) => {
                let val = self.eval_expr(expr)?;
                match op {
                    UnOp::Not => Ok(Value::Bool(!val.as_bool())),
                }
            }
            Expr::Call(name, args, span) => {
                // Function call
                if let Some((params, body)) = self.functions.get(name).cloned() {
                    if args.len() != params.len() {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: format!(
                                "Function '{}' expects {} arguments, got {}",
                                name,
                                params.len(),
                                args.len()
                            ),
                        });
                    }
                    // Create a new environment for the function
                    let mut child_env = self.env.child();
                    for (param, arg_expr) in params.iter().zip(args) {
                        let arg_val = self.eval_expr(arg_expr)?;
                        child_env.set(param.clone(), arg_val);
                    }
                    let old_env = std::mem::replace(&mut self.env, child_env);
                    let old_return = self.return_value.take();
                    // Execute body
                    for stmt in body {
                        self.execute_stmt(&stmt)?;
                        if self.return_value.is_some() {
                            break;
                        }
                        if self.break_flag {
                            self.break_flag = false;
                        }
                    }
                    let result = self.return_value.take().unwrap_or(Value::Nil);
                    self.env = old_env;
                    self.return_value = old_return;
                    Ok(result)
                } else {
                    Err(InterpError::Runtime {
                        span: *span,
                        message: format!("Function '{}' not found", name),
                    })
                }
            }
            Expr::List(elements, _) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.eval_expr(element)?);
                }
                Ok(Value::List(Rc::new(RefCell::new(values))))
            }
            Expr::Dict(pairs, _) => {
                let mut map = HashMap::new();
                for (key_expr, val_expr) in pairs {
                    let key = self.eval_expr(key_expr)?;
                    if !key.is_hashable() {
                        return Err(InterpError::Runtime {
                            span: key_expr.span(),
                            message: "Dictionary key must be hashable".to_string(),
                        });
                    }
                    let val = self.eval_expr(val_expr)?;
                    map.insert(key, val);
                }
                Ok(Value::Dict(Rc::new(RefCell::new(map))))
            }
            Expr::Index(base, index, span) => {
                let base_val = self.eval_expr(base)?;
                let idx_val = self.eval_expr(index)?;
                match base_val {
                    Value::List(list) => {
                        let idx = match idx_val {
                            Value::Int(i) => i,
                            _ => {
                                return Err(InterpError::Runtime {
                                    span: *span,
                                    message: "List index must be an integer".to_string(),
                                });
                            }
                        };
                        let list_ref = list.borrow();
                        if idx < 0 || idx as usize >= list_ref.len() {
                            return Err(InterpError::Runtime {
                                span: *span,
                                message: format!("Index {} out of bounds", idx),
                            });
                        }
                        Ok(list_ref[idx as usize].clone())
                    }
                    Value::Dict(dict) => {
                        if !idx_val.is_hashable() {
                            return Err(InterpError::Runtime {
                                span: *span,
                                message: "Dictionary key must be hashable".to_string(),
                            });
                        }
                        let dict_ref = dict.borrow();
                        if let Some(val) = dict_ref.get(&idx_val) {
                            Ok(val.clone())
                        } else {
                            Err(InterpError::Runtime {
                                span: *span,
                                message: format!("Key {:?} not found", idx_val),
                            })
                        }
                    }
                    _ => Err(InterpError::Runtime {
                        span: *span,
                        message: format!(
                            "Cannot index value of type {}",
                            crate::utils::type_name(&base_val)
                        ),
                    }),
                }
            }
        }
    }

    fn assign_index(
        &mut self,
        base_val: Value,
        idx_val: Value,
        value: Value,
        span: Span,
    ) -> InterpResult<()> {
        match base_val {
            Value::List(list) => {
                let idx = match idx_val {
                    Value::Int(i) => i,
                    _ => {
                        return Err(InterpError::Runtime {
                            span,
                            message: "List index must be integer".to_string(),
                        });
                    }
                };
                let mut list_ref = list.borrow_mut();
                if idx < 0 || idx as usize >= list_ref.len() {
                    return Err(InterpError::Runtime {
                        span,
                        message: format!("Index {} out of bounds", idx),
                    });
                }
                list_ref[idx as usize] = value;
                Ok(())
            }
            Value::Dict(dict) => {
                if !idx_val.is_hashable() {
                    return Err(InterpError::Runtime {
                        span,
                        message: "Dictionary key must be hashable".to_string(),
                    });
                }
                let mut dict_ref = dict.borrow_mut();
                dict_ref.insert(idx_val, value);
                Ok(())
            }
            _ => Err(InterpError::Runtime {
                span,
                message: format!(
                    "Cannot assign to index of type {}",
                    crate::utils::type_name(&base_val)
                ),
            }),
        }
    }

    /// Evaluates a binary operation and returns a Value (for arithmetic, comparison, logic).
    fn evaluate_binary_op(
        op: BinOp,
        left: &Value,
        right: &Value,
        span: &Span,
    ) -> InterpResult<Value> {
        match op {
            BinOp::Add => Self::apply_arithmetic(left, right, |x, y| x + y, |x, y| x + y, span),
            BinOp::Sub => Self::apply_arithmetic(left, right, |x, y| x - y, |x, y| x - y, span),
            BinOp::Mul => Self::apply_arithmetic(left, right, |x, y| x * y, |x, y| x * y, span),
            BinOp::Div => {
                if crate::utils::is_zero(right) {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "Division by zero".to_string(),
                    });
                }
                Self::apply_arithmetic(left, right, |x, y| x / y, |x, y| x / y, span)
            }
            BinOp::Mod => {
                if crate::utils::is_zero(right) {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "Modulo by zero".to_string(),
                    });
                }
                match (left, right) {
                    (Value::Int(ai), Value::Int(bi)) => Ok(Value::Int(ai % bi)),
                    _ => {
                        let af = match left {
                            Value::Int(i) => *i as f64,
                            Value::Float(f) => *f,
                            _ => {
                                return Err(InterpError::Runtime {
                                    span: *span,
                                    message: "Operands must be numbers".to_string(),
                                });
                            }
                        };
                        let bf = match right {
                            Value::Int(i) => *i as f64,
                            Value::Float(f) => *f,
                            _ => {
                                return Err(InterpError::Runtime {
                                    span: *span,
                                    message: "Operands must be numbers".to_string(),
                                });
                            }
                        };
                        Ok(Value::Float(af % bf))
                    }
                }
            }
            BinOp::Pow => match (left, right) {
                (Value::Int(ai), Value::Int(bi)) => {
                    if *bi < 0 {
                        let af = *ai as f64;
                        let bf = *bi as f64;
                        Ok(Value::Float(af.powf(bf)))
                    } else {
                        match ai.checked_pow(*bi as u32) {
                            Some(result) => Ok(Value::Int(result)),
                            None => {
                                let af = *ai as f64;
                                let bf = *bi as f64;
                                Ok(Value::Float(af.powf(bf)))
                            }
                        }
                    }
                }
                _ => {
                    let af = match left {
                        Value::Int(i) => *i as f64,
                        Value::Float(f) => *f,
                        _ => {
                            return Err(InterpError::Runtime {
                                span: *span,
                                message: "Operands must be numbers".to_string(),
                            });
                        }
                    };
                    let bf = match right {
                        Value::Int(i) => *i as f64,
                        Value::Float(f) => *f,
                        _ => {
                            return Err(InterpError::Runtime {
                                span: *span,
                                message: "Operands must be numbers".to_string(),
                            });
                        }
                    };
                    Ok(Value::Float(af.powf(bf)))
                }
            },
            _ => {
                let bool_result = Self::evaluate_binary_op_bool(op, left, right, span)?;
                Ok(Value::Bool(bool_result))
            }
        }
    }

    /// Evaluates a binary operation that yields a boolean (comparisons and logical AND/OR).
    fn evaluate_binary_op_bool(
        op: BinOp,
        left: &Value,
        right: &Value,
        span: &Span,
    ) -> InterpResult<bool> {
        match op {
            BinOp::Eq | BinOp::Ne | BinOp::Gt | BinOp::Ge | BinOp::Lt | BinOp::Le => {
                Self::compare_values(left, right, op, span)
            }
            BinOp::And => Ok(left.as_bool() && right.as_bool()),
            BinOp::Or => Ok(left.as_bool() || right.as_bool()),
            _ => Err(InterpError::Internal(format!(
                "Non-boolean operation: {:?}",
                op
            ))),
        }
    }

    /// Compares two values according to the comparison operator.
    fn compare_values(left: &Value, right: &Value, op: BinOp, span: &Span) -> InterpResult<bool> {
        use std::cmp::Ordering;
        let cmp_result = match (left, right) {
            (Value::Int(li), Value::Int(ri)) => Some(li.cmp(ri)),
            (Value::Int(li), Value::Float(rf)) => {
                Some((*li as f64).partial_cmp(rf).unwrap_or(Ordering::Equal))
            }
            (Value::Float(lf), Value::Int(ri)) => {
                Some(lf.partial_cmp(&(*ri as f64)).unwrap_or(Ordering::Equal))
            }
            (Value::Float(lf), Value::Float(rf)) => {
                Some(lf.partial_cmp(rf).unwrap_or(Ordering::Equal))
            }
            (Value::String(ls), Value::String(rs)) => Some(ls.cmp(rs)),
            (Value::Bool(lb), Value::Bool(rb)) => Some(lb.cmp(rb)),
            _ => None,
        };

        match (op, cmp_result) {
            (BinOp::Eq, Some(ord)) => Ok(ord == Ordering::Equal),
            (BinOp::Ne, Some(ord)) => Ok(ord != Ordering::Equal),
            (BinOp::Gt, Some(ord)) => Ok(ord == Ordering::Greater),
            (BinOp::Ge, Some(ord)) => Ok(ord == Ordering::Greater || ord == Ordering::Equal),
            (BinOp::Lt, Some(ord)) => Ok(ord == Ordering::Less),
            (BinOp::Le, Some(ord)) => Ok(ord == Ordering::Less || ord == Ordering::Equal),
            _ => {
                let left_type = crate::utils::type_name(left);
                let right_type = crate::utils::type_name(right);
                Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "Cannot compare values of types '{}' and '{}'",
                        left_type, right_type
                    ),
                })
            }
        }
    }

    /// Helper to apply arithmetic operations on two Values.
    fn apply_arithmetic<FInt, FFloat>(
        a: &Value,
        b: &Value,
        op_int: FInt,
        op_float: FFloat,
        span: &Span,
    ) -> InterpResult<Value>
    where
        FInt: Fn(i64, i64) -> i64,
        FFloat: Fn(f64, f64) -> f64,
    {
        match (a, b) {
            (Value::Int(ai), Value::Int(bi)) => {
                let result = op_int(*ai, *bi);
                Ok(Value::Int(result))
            }
            _ => {
                let af = match a {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => {
                        let a_type = crate::utils::type_name(a);
                        let b_type = crate::utils::type_name(b);
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: format!(
                                "Arithmetic operation requires numbers, got '{}' and '{}'",
                                a_type, b_type
                            ),
                        });
                    }
                };
                let bf = match b {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => {
                        let a_type = crate::utils::type_name(a);
                        let b_type = crate::utils::type_name(b);
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: format!(
                                "Arithmetic operation requires numbers, got '{}' and '{}'",
                                a_type, b_type
                            ),
                        });
                    }
                };
                let result = op_float(af, bf);
                Ok(Value::Float(result))
            }
        }
    }
}
