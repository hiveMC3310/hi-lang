//! Interpreter for the Hi language, executes AST.
#![allow(clippy::mutable_key_type)]

use crate::ast::{BinOp, Block, Expr, Program, Span, Stmt, UnOp};
use crate::builtins;
use crate::error::{InterpError, InterpResult};
use crate::modules::{BuiltinFn, BuiltinModule, Module, UserModule};
use crate::parser::Parser;
use crate::parser::lexer::Lexer;
use crate::value::Value;
use hi_common::Symbol;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone)]
pub enum Binding {
    Variable(Value),
    UserFunction(Vec<Symbol>, Block),
    BuiltinFunction(BuiltinFn),
    Module(Rc<RefCell<dyn Module>>),
}

#[derive(Clone)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    bindings: HashMap<Symbol, Binding>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
        }
    }

    pub fn child(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            parent: Some(parent),
            bindings: HashMap::new(),
        }
    }

    // Unified lookup considering parent chain
    pub fn lookup(&self, name: Symbol) -> Option<Binding> {
        self.bindings
            .get(&name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.borrow().lookup(name)))
    }

    // Declare any binding in the current environment
    pub fn define(&mut self, name: Symbol, binding: Binding) {
        self.bindings.insert(name, binding);
    }

    // Assign a value to a variable (only for existing Variable)
    pub fn assign(&mut self, name: Symbol, value: Value, span: &Span) -> InterpResult<()> {
        if matches!(self.bindings.get(&name), Some(Binding::Variable(_))) {
            self.bindings.insert(name, Binding::Variable(value));
            return Ok(());
        }
        if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value, span)
        } else {
            Err(InterpError::Runtime {
                span: *span,
                message: format!("Undefined variable '{}'", name),
            })
        }
    }

    /// Returns an iterator over all bindings in this environment (excluding parents).
    pub fn bindings(&self) -> impl Iterator<Item = (Symbol, &Binding)> {
        self.bindings.iter().map(|(k, v)| (*k, v))
    }
}

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
    pub return_value: Option<Value>,
    pub break_flag: bool,
    pub loop_depth: usize,
    pub argv: Vec<String>,
    pub current_file: Option<PathBuf>,
    modules_cache: HashMap<PathBuf, Rc<RefCell<dyn Module>>>,
    load_stack: Vec<PathBuf>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let mut s = Self {
            env: Rc::new(RefCell::new(Environment::new())),
            return_value: None,
            break_flag: false,
            loop_depth: 0,
            argv: Vec::new(),
            modules_cache: HashMap::new(),
            load_stack: Vec::new(),
            current_file: None,
        };
        Self::init_builtins(&mut s.env.borrow_mut());
        s.init_globals();
        s
    }

    fn init_globals(&mut self) {
        // ARGS
        let args_list = Value::List(Rc::new(RefCell::new(Vec::new())));
        self.env
            .borrow_mut()
            .define(hi_common::intern("ARGS"), Binding::Variable(args_list));
        // ARGS_DICT
        let args_dict = Value::Dict(Rc::new(RefCell::new(HashMap::new())));
        self.env
            .borrow_mut()
            .define(hi_common::intern("ARGS_DICT"), Binding::Variable(args_dict));
    }

    fn init_builtins(env: &mut Environment) {
        // Global functions
        for gf in builtins::get_global_functions() {
            env.define(gf.name, Binding::BuiltinFunction(Rc::new(gf.func)));
        }
        // Built-in modules
        for (module_sym, funcs) in builtins::get_module_functions_map() {
            let mut func_map: HashMap<Symbol, BuiltinFn> = HashMap::new();
            for mf in &funcs {
                func_map.insert(mf.name, Rc::new(mf.func));
            }
            let mut vars = HashMap::new();
            if hi_common::resolve(module_sym) == "math" {
                vars.insert(hi_common::intern("PI"), Value::Float(std::f64::consts::PI));
                vars.insert(hi_common::intern("E"), Value::Float(std::f64::consts::E));
            }
            let builtin = Rc::new(RefCell::new(BuiltinModule {
                vars,
                funcs: func_map,
            }));
            env.define(module_sym, Binding::Module(builtin));
        }
    }

    pub fn set_argv(&mut self, argv: Vec<String>) -> InterpResult<()> {
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
        self.env.borrow_mut().assign(
            hi_common::intern("ARGS"),
            Value::List(args_rc),
            &Span::dummy(),
        )?;

        // Update ARGS_DICT
        let dict_rc = Rc::new(RefCell::new(dict));
        self.env.borrow_mut().assign(
            hi_common::intern("ARGS_DICT"),
            Value::Dict(dict_rc),
            &Span::dummy(),
        )?;

        Ok(())
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

    pub(crate) fn execute_stmt(&mut self, stmt: &Stmt) -> InterpResult<()> {
        match stmt {
            Stmt::Let(name, expr, _, _) => {
                let val = self.eval_expr(expr)?;
                self.env.borrow_mut().define(*name, Binding::Variable(val));
                Ok(())
            }
            Stmt::Assign(left, right, span) => {
                let value = self.eval_expr(right)?;
                self.assign_to_lvalue(left, value, span)
            }
            Stmt::CompoundAssign(left, op, right, span) => {
                let current_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                let result = Self::evaluate_binary_op(*op, &current_val, &right_val, span)?;
                self.assign_to_lvalue(left, result, span)?;
                Ok(())
            }

            Stmt::Input(prompt_opt, var, span) => {
                if let Some(prompt) = prompt_opt {
                    print!("{}", prompt);
                    std::io::stdout().flush().map_err(|e| InterpError::Io {
                        source: e,
                        span: Some(*span),
                    })?;
                }
                let mut input = String::new();
                let bytes_read =
                    std::io::stdin()
                        .read_line(&mut input)
                        .map_err(|e| InterpError::Io {
                            source: e,
                            span: Some(*span),
                        })?;
                if bytes_read == 0 {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "EOF reached while reading input".to_string(),
                    });
                }
                let input = input.trim_end_matches(&['\n', '\r'][..]);
                let value = crate::utils::parse(input);
                self.env.borrow_mut().define(*var, Binding::Variable(value));
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
            Stmt::For(var, start_expr, end_expr, step_expr, body, _, _) => {
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
                    self.env
                        .borrow_mut()
                        .define(*var, Binding::Variable(Value::Int(current)));
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
            Stmt::Func(name, params, body, _, _, _) => {
                let param_names: Vec<Symbol> = params.iter().map(|(s, _)| *s).collect();
                self.env
                    .borrow_mut()
                    .define(*name, Binding::UserFunction(param_names, body.clone()));
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
            Stmt::Import(path, alias, span) => {
                // Determine if it's a user module or built-in
                if path.ends_with(".hi") {
                    // User module: ensure it's not a built-in name
                    let module_name = path.trim_end_matches(".hi");
                    let module_sym = hi_common::intern(module_name);
                    // Check if it's a built-in module (should not happen, but guard)
                    if builtins::get_module_functions_map().contains_key(&module_sym) {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: format!(
                                "Cannot import built-in module '{}' with .hi extension. Use 'IMPORT \"{}\"'.",
                                module_name, module_name
                            ),
                        });
                    }
                    // Load user module from disk
                    let base_dir = self
                        .current_file
                        .as_ref()
                        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                        .unwrap_or_else(|| {
                            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                        });
                    let import_path = base_dir.join(path);
                    self.load_user_module(&import_path, alias.as_ref(), span)?;
                } else {
                    // Built-in module: must exist
                    let module_sym = hi_common::intern(path);
                    if !builtins::get_module_functions_map().contains_key(&module_sym) {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: format!(
                                "Unknown module '{}'. If this is a user module, use '.hi' extension.",
                                path
                            ),
                        });
                    }
                    // Check if already loaded (from previous imports)
                    let found_module = {
                        let env = self.env.borrow();
                        match env.lookup(module_sym) {
                            Some(Binding::Module(m)) => Some(m.clone()),
                            _ => None,
                        }
                    };
                    if let Some(module_rc) = found_module {
                        self.attach_module(module_rc, alias.as_ref(), span)?;
                    } else {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: format!(
                                "Built-in module '{}' not found (internal error)",
                                path
                            ),
                        });
                    }
                }
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
            Expr::Variable(name, span) => match self.env.borrow().lookup(*name) {
                Some(Binding::Variable(val)) => Ok(val),
                Some(Binding::UserFunction(..)) | Some(Binding::BuiltinFunction(_)) => {
                    Ok(Value::Function(*name))
                }
                Some(Binding::Module(module_rc)) => Ok(Value::Module(module_rc)),
                None => Err(InterpError::Runtime {
                    span: *span,
                    message: format!("Undefined variable or function '{}'", name),
                }),
            },
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
            Expr::Call(name, args, _, span) => {
                let binding =
                    self.env
                        .borrow()
                        .lookup(*name)
                        .ok_or_else(|| InterpError::Runtime {
                            span: *span,
                            message: format!("Function '{}' not found", name),
                        })?;

                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<InterpResult<_>>()?;

                match binding {
                    Binding::BuiltinFunction(f) => f(self, &arg_vals, span),
                    Binding::UserFunction(params, body) => {
                        if arg_vals.len() != params.len() {
                            return Err(InterpError::Runtime {
                                span: *span,
                                message: format!(
                                    "Function '{}' expects {} arguments, got {}",
                                    name,
                                    params.len(),
                                    arg_vals.len()
                                ),
                            });
                        }
                        let mut child = Environment::child(self.env.clone());
                        for (p, v) in params.iter().zip(arg_vals) {
                            child.define(*p, Binding::Variable(v));
                        }
                        let old_env =
                            std::mem::replace(&mut self.env, Rc::new(RefCell::new(child)));
                        let old_ret = self.return_value.take();
                        for s in body {
                            self.execute_stmt(&s)?;
                            if self.return_value.is_some() || self.break_flag {
                                break;
                            }
                        }
                        let result = self.return_value.take().unwrap_or(Value::Nil);
                        self.env = old_env;
                        self.return_value = old_ret;
                        Ok(result)
                    }
                    _ => Err(InterpError::Runtime {
                        span: *span,
                        message: format!("'{}' is not callable", name),
                    }),
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
            Expr::ModuleAccess(module_name, var_name, _, span) => {
                let module_val = self.eval_expr(&Expr::Variable(*module_name, *span))?;
                match module_val {
                    Value::Module(module_rc) => {
                        let module = module_rc.borrow();
                        if let Some(val) = module.get_var(*var_name) {
                            Ok(val)
                        } else {
                            Err(InterpError::Runtime {
                                span: *span,
                                message: format!(
                                    "Variable '{}' not found in module '{}'",
                                    var_name, module_name
                                ),
                            })
                        }
                    }
                    _ => Err(InterpError::Runtime {
                        span: *span,
                        message: format!("Module '{}' not found", module_name),
                    }),
                }
            }
            Expr::CallModule(module_name, func_name, args, _, span) => {
                let module_val = self.eval_expr(&Expr::Variable(*module_name, *span))?;
                match module_val {
                    Value::Module(module_rc) => {
                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.eval_expr(arg)?);
                        }
                        let module = module_rc.borrow();
                        module.call_function(*func_name, &arg_values, self, span)
                    }
                    _ => Err(InterpError::Runtime {
                        span: *span,
                        message: format!("Module '{}' not found", module_name),
                    }),
                }
            }
        }
    }

    fn load_user_module(
        &mut self,
        path: &Path,
        alias: Option<&Symbol>,
        span: &Span,
    ) -> InterpResult<()> {
        let abs_path = path.canonicalize().map_err(|e| InterpError::Io {
            source: e,
            span: Some(*span),
        })?;

        // Cyclic import
        if self.load_stack.contains(&abs_path) {
            return Err(InterpError::CyclicImport {
                path: abs_path.display().to_string(),
            });
        }

        // Cache
        if let Some(module) = self.modules_cache.get(&abs_path) {
            return self.attach_module(module.clone(), alias, span);
        }

        let module_env = Rc::new(RefCell::new(Environment::child(self.env.clone())));
        let old_env = std::mem::replace(&mut self.env, module_env.clone());
        let old_return = self.return_value.take();
        let old_break = self.break_flag;

        let old_file = self.current_file.clone();
        self.current_file = Some(abs_path.clone());

        let source = std::fs::read_to_string(&abs_path).map_err(|e| InterpError::Io {
            source: e,
            span: Some(*span),
        })?;
        let tokens = Lexer::tokenize(&source)?;
        let mut parser = Parser::new(&tokens);
        let program = parser.parse()?;

        self.load_stack.push(abs_path.clone());
        for stmt in program.stmts {
            self.execute_stmt(&stmt)?;
            if self.return_value.is_some() {
                break;
            }
            if self.break_flag {
                self.break_flag = false;
            }
        }
        self.load_stack.pop();

        // Восстанавливаем всё
        self.current_file = old_file;
        self.env = old_env;
        self.return_value = old_return;
        self.break_flag = old_break;

        let module = Rc::new(RefCell::new(UserModule { env: module_env }));
        self.modules_cache.insert(abs_path, module.clone());
        self.attach_module(module, alias, span)
    }

    fn attach_module(
        &mut self,
        module: Rc<RefCell<dyn Module>>,
        alias: Option<&Symbol>,
        _: &Span,
    ) -> InterpResult<()> {
        match alias {
            Some(sym) => {
                self.env.borrow_mut().define(*sym, Binding::Module(module));
                Ok(())
            }
            None => {
                let module_ref = module.borrow();
                module_ref.inline_into(&mut self.env.borrow_mut())
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

    fn assign_to_lvalue(&mut self, left: &Expr, value: Value, span: &Span) -> InterpResult<()> {
        match left {
            Expr::Variable(name, _) => {
                self.env.borrow_mut().assign(*name, value, span)?;
                Ok(())
            }
            Expr::Index(base, index, span) => {
                let base_val = self.eval_expr(base)?;
                let idx_val = self.eval_expr(index)?;
                self.assign_index(base_val, idx_val, value, *span)
            }
            _ => Err(InterpError::Runtime {
                span: *span,
                message: "Invalid left-hand side for assignment".to_string(),
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
