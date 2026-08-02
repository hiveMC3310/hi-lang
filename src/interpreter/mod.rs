//! Interpreter for the Hi language, executes AST.

mod builtins;

use crate::ast::{BinOp, Block, Expr, Program, Span, Stmt, UnOp};
use crate::error::{InterpError, InterpResult};
use crate::interpreter::builtins::{Builtin, BuiltinFn};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
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

    pub fn declare(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Value, span: &Span) -> InterpResult<()> {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, value, span)
        } else {
            Err(InterpError::Runtime {
                span: *span,
                message: format!("Undefined variable '{}'", name),
            })
        }
    }
}

pub struct Interpreter {
    pub env: Environment,
    pub functions: HashMap<String, (Vec<String>, Block)>,
    pub builtins: HashMap<String, BuiltinFn>,
    pub return_value: Option<Value>,
    pub break_flag: bool,
    pub loop_depth: usize,
    pub argv: Vec<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut s = Self {
            env: Environment::new(),
            functions: HashMap::new(),
            return_value: None,
            break_flag: false,
            loop_depth: 0,
            builtins: HashMap::new(),
            argv: Vec::new(),
        };
        s.init_builtins();
        s.init_globals();
        s
    }

    fn init_globals(&mut self) {
        // ARGS
        let args_list = Value::List(Rc::new(RefCell::new(Vec::new())));
        self.env.declare("ARGS".to_string(), args_list);
        // ARGS_DICT
        let args_dict = Value::Dict(Rc::new(RefCell::new(HashMap::new())));
        self.env.declare("ARGS_DICT".to_string(), args_dict);
        // Math consts
        self.env
            .declare("PI".to_string(), Value::Float(std::f64::consts::PI));
        self.env
            .declare("E".to_string(), Value::Float(std::f64::consts::E));
    }

    fn init_builtins(&mut self) {
        self.builtins
            .insert("hello".to_string(), Rc::new(Builtin::hello_fn));
        self.builtins
            .insert("len".to_string(), Rc::new(Builtin::len_fn));
        self.builtins
            .insert("keys".to_string(), Rc::new(Builtin::keys_fn));
        self.builtins
            .insert("values".to_string(), Rc::new(Builtin::values_fn));
        self.builtins
            .insert("append".to_string(), Rc::new(Builtin::append_fn));
        self.builtins
            .insert("insert".to_string(), Rc::new(Builtin::insert_fn));
        self.builtins
            .insert("remove".to_string(), Rc::new(Builtin::remove_fn));
        self.builtins
            .insert("contains".to_string(), Rc::new(Builtin::contains_fn));
        self.builtins
            .insert("split".to_string(), Rc::new(Builtin::split_fn));
        self.builtins
            .insert("replace".to_string(), Rc::new(Builtin::replace_fn));
        self.builtins
            .insert("starts".to_string(), Rc::new(Builtin::starts_fn));
        self.builtins
            .insert("ends".to_string(), Rc::new(Builtin::ends_fn));
        self.builtins
            .insert("upper".to_string(), Rc::new(Builtin::upper_fn));
        self.builtins
            .insert("lower".to_string(), Rc::new(Builtin::lower_fn));
        self.builtins
            .insert("trim".to_string(), Rc::new(Builtin::trim_fn));
        self.builtins
            .insert("concat".to_string(), Rc::new(Builtin::concat_fn));
        self.builtins
            .insert("substr".to_string(), Rc::new(Builtin::substr_fn));
        self.builtins
            .insert("slice".to_string(), Rc::new(Builtin::slice_fn));
        self.builtins
            .insert("reverse".to_string(), Rc::new(Builtin::reverse_fn));
        self.builtins
            .insert("indexof".to_string(), Rc::new(Builtin::indexof_fn));
        self.builtins
            .insert("put".to_string(), Rc::new(Builtin::put_fn));
        self.builtins
            .insert("get".to_string(), Rc::new(Builtin::get_fn));
        self.builtins
            .insert("open".to_string(), Rc::new(Builtin::open_fn));
        self.builtins
            .insert("read".to_string(), Rc::new(Builtin::read_fn));
        self.builtins
            .insert("readln".to_string(), Rc::new(Builtin::readln_fn));
        self.builtins
            .insert("write".to_string(), Rc::new(Builtin::write_fn));
        self.builtins
            .insert("writeln".to_string(), Rc::new(Builtin::writeln_fn));
        self.builtins
            .insert("close".to_string(), Rc::new(Builtin::close_fn));
        self.builtins
            .insert("eof".to_string(), Rc::new(Builtin::eof_fn));
        self.builtins
            .insert("sin".to_string(), Rc::new(Builtin::sin_fn));
        self.builtins
            .insert("cos".to_string(), Rc::new(Builtin::cos_fn));
        self.builtins
            .insert("tan".to_string(), Rc::new(Builtin::tan_fn));
        self.builtins
            .insert("asin".to_string(), Rc::new(Builtin::asin_fn));
        self.builtins
            .insert("acos".to_string(), Rc::new(Builtin::acos_fn));
        self.builtins
            .insert("atan".to_string(), Rc::new(Builtin::atan_fn));
        self.builtins
            .insert("sqrt".to_string(), Rc::new(Builtin::sqrt_fn));
        self.builtins
            .insert("torad".to_string(), Rc::new(Builtin::torad_fn));
        self.builtins
            .insert("todeg".to_string(), Rc::new(Builtin::todeg_fn));
        self.builtins
            .insert("exp".to_string(), Rc::new(Builtin::exp_fn));
        self.builtins
            .insert("log".to_string(), Rc::new(Builtin::log_fn));
        self.builtins
            .insert("log2".to_string(), Rc::new(Builtin::log2_fn));
        self.builtins
            .insert("log10".to_string(), Rc::new(Builtin::log10_fn));
        self.builtins
            .insert("ceil".to_string(), Rc::new(Builtin::ceil_fn));
        self.builtins
            .insert("floor".to_string(), Rc::new(Builtin::floor_fn));
        self.builtins
            .insert("round".to_string(), Rc::new(Builtin::round_fn));
        self.builtins
            .insert("abs".to_string(), Rc::new(Builtin::abs_fn));
        self.builtins
            .insert("rand".to_string(), Rc::new(Builtin::rand_fn));
        self.builtins
            .insert("tostring".to_string(), Rc::new(Builtin::tostring_fn));
        self.builtins
            .insert("toint".to_string(), Rc::new(Builtin::toint_fn));
        self.builtins
            .insert("tofloat".to_string(), Rc::new(Builtin::tofloat_fn));
        self.builtins
            .insert("call".to_string(), Rc::new(Builtin::call_fn));
        self.builtins
            .insert("typeof".to_string(), Rc::new(Builtin::typeof_fn));
    }

    pub fn set_argv(&mut self, argv: Vec<String>) {
        self.argv = argv.clone();

        let mut positional = Vec::new();
        let mut dict = HashMap::new();
        let mut iter = argv.iter().peekable();

        while let Some(arg) = iter.next() {
            if arg.starts_with("--") && arg.len() > 2 {
                let key_str = arg[2..].to_string();
                if let Some(eq_pos) = key_str.find('=') {
                    let key = key_str[..eq_pos].to_string();
                    let value = key_str[eq_pos + 1..].to_string();
                    dict.insert(Value::String(key), Value::String(value));
                } else {
                    if let Some(next_arg) = iter.peek() {
                        if !next_arg.starts_with('-') {
                            let val = (*next_arg).clone();
                            dict.insert(Value::String(key_str), Value::String(val));
                            iter.next();
                        } else {
                            dict.insert(Value::String(key_str), Value::Bool(true));
                        }
                    } else {
                        dict.insert(Value::String(key_str), Value::Bool(true));
                    }
                }
            } else if arg.starts_with('-') && arg.len() > 1 {
                let key_str = arg[1..].to_string();
                if let Some(eq_pos) = key_str.find('=') {
                    let key = key_str[..eq_pos].to_string();
                    let value = key_str[eq_pos + 1..].to_string();
                    dict.insert(Value::String(key), Value::String(value));
                } else {
                    if let Some(next_arg) = iter.peek() {
                        if !next_arg.starts_with('-') {
                            let val = (*next_arg).clone();
                            dict.insert(Value::String(key_str), Value::String(val));
                            iter.next();
                        } else {
                            dict.insert(Value::String(key_str), Value::Bool(true));
                        }
                    } else {
                        dict.insert(Value::String(key_str), Value::Bool(true));
                    }
                }
            } else {
                positional.push(Value::String(arg.clone()));
            }
        }

        // Update ARGS
        let args_rc = Rc::new(RefCell::new(positional));
        self.env.declare("ARGS".to_string(), Value::List(args_rc));

        // Update ARGS_DICT
        let dict_rc = Rc::new(RefCell::new(dict));
        self.env
            .declare("ARGS_DICT".to_string(), Value::Dict(dict_rc));
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
                self.env.declare(name.clone(), val);
                Ok(())
            }
            Stmt::Assign(left, right, span) => {
                let value = self.eval_expr(right)?;
                match **left {
                    Expr::Variable(ref name, _) => {
                        self.env.assign(name, value, span)?;
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
            Stmt::Input(prompt_opt, var, span) => {
                if let Some(prompt) = prompt_opt {
                    print!("{}", prompt);
                    std::io::stdout().flush().map_err(InterpError::Io)?;
                }
                let mut input = String::new();
                let bytes_read = std::io::stdin()
                    .read_line(&mut input)
                    .map_err(InterpError::Io)?;
                if bytes_read == 0 {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "EOF reached while reading input".to_string(),
                    });
                }
                let input = input.trim_end_matches(&['\n', '\r'][..]);
                let value = crate::utils::parse(input);
                self.env.declare(var.clone(), value);
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
            Stmt::For(var, start_expr, end_expr, step_expr, body, _) => {
                let start_val = self.eval_expr(start_expr)?;
                let end_val = self.eval_expr(end_expr)?;
                let step_val = if let Some(step_expr) = step_expr {
                    self.eval_expr(step_expr)?
                } else {
                    Value::Int(1)
                };
                // Check all values are numbers
                let start = match start_val {
                    Value::Int(i) => i,
                    _ => {
                        return Err(InterpError::Runtime {
                            span: start_expr.span(),
                            message: "FOR start must be integer".to_string(),
                        });
                    }
                };
                let end = match end_val {
                    Value::Int(i) => i,
                    _ => {
                        return Err(InterpError::Runtime {
                            span: end_expr.span(),
                            message: "FOR end must be integer".to_string(),
                        });
                    }
                };
                let step = match step_val {
                    Value::Int(i) => i,
                    _ => {
                        return Err(InterpError::Runtime {
                            span: step_expr
                                .as_ref()
                                .map(|e| e.span())
                                .unwrap_or(Span::dummy()),
                            message: "FOR step must be integer".to_string(),
                        });
                    }
                };
                if step == 0 {
                    return Err(InterpError::Runtime {
                        span: step_expr
                            .as_ref()
                            .map(|e| e.span())
                            .unwrap_or(Span::dummy()),
                        message: "FOR step cannot be zero".to_string(),
                    });
                }
                let mut current = start;
                self.loop_depth += 1;
                while (step > 0 && current <= end) || (step < 0 && current >= end) {
                    self.env.declare(var.clone(), Value::Int(current));
                    for stmt in body {
                        self.execute_stmt(stmt)?;
                        if self.return_value.is_some() || self.break_flag {
                            break;
                        }
                    }
                    if self.return_value.is_some() || self.break_flag {
                        break;
                    }
                    current += step;
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
            Expr::Variable(name, span) => {
                if let Some(v) = self.env.get(name) {
                    return Ok(v);
                }
                if self.functions.contains_key(name) {
                    return Ok(Value::Function(name.clone()));
                }
                Err(InterpError::Runtime {
                    span: *span,
                    message: format!("Undefined variable or function '{}'", name),
                })
            }
            Expr::Binary(op, left, right, span) => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                Self::evaluate_binary_op(*op, &left_val, &right_val, span)
            }
            Expr::Unary(op, expr, _) => {
                let val = self.eval_expr(expr)?;
                match op {
                    UnOp::Not => Ok(Value::Bool(!val.as_bool())),
                    UnOp::Neg => match val {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(InterpError::Runtime {
                            span: expr.span(),
                            message: format!(
                                "Cannot negate value of type {}",
                                crate::utils::type_name(&val)
                            ),
                        }),
                    },
                }
            }
            Expr::Call(name, args, span) => {
                // Function call

                if let Some(builtin_fn) = self.builtins.get(name) {
                    let f = builtin_fn.clone();
                    return f(self, args, span);
                }

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
                        child_env.declare(param.clone(), arg_val);
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
                let af = match left {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: "Division requires numbers".to_string(),
                        });
                    }
                };
                let bf = match right {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: "Division requires numbers".to_string(),
                        });
                    }
                };
                let result = af / bf;

                if result.fract() == 0.0 {
                    Ok(Value::Int(result as i64))
                } else {
                    Ok(Value::Float(result))
                }
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
