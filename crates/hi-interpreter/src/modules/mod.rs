//! Defines the module system for the Hi language.
//!
//! This module provides the core abstractions for built-in and user-defined modules.
//! It includes the `Module` trait, which all modules must implement, along with
//! concrete implementations for user modules (loaded from `.hi` files) and
//! built-in modules (implemented in Rust).
#![allow(clippy::mutable_key_type)]

pub mod collections;
pub mod core;
pub mod datetime;
pub mod io;
pub mod json;
pub mod math;
pub mod os;
pub mod path;
pub mod random;
pub mod regex;
pub mod strings;

use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::{Binding, Environment, Interpreter};
use crate::value::Value;
use hi_common::Symbol;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Type alias for a built-in function implementation.
///
/// Built-in functions are Rust closures that take:
/// - a mutable reference to the interpreter,
/// - a slice of argument values,
/// - a span for error reporting,
///
/// and return a `Value` or an error.
pub type BuiltinFn = Rc<dyn Fn(&mut Interpreter, &[Value], &Span) -> InterpResult<Value>>;

/// Trait that all modules (built-in or user-defined) must implement.
pub trait Module {
    /// Retrieves the value of a variable exported by the module.
    ///
    /// Returns `None` if the variable does not exist.
    fn get_var(&self, name: Symbol) -> Option<Value>;

    /// Calls a function exported by the module.
    ///
    /// # Arguments
    /// * `name` – the name of the function.
    /// * `args` – the arguments passed to the function.
    /// * `interp` – mutable reference to the interpreter (for nested calls).
    /// * `span` – source location for error reporting.
    ///
    /// # Returns
    /// The function's return value, or an error.
    fn call_function(
        &self,
        name: Symbol,
        args: &[Value],
        interp: &mut Interpreter,
        span: &Span,
    ) -> InterpResult<Value>;

    /// Inlines the module's exports into the current interpreter environment.
    ///
    /// This is used when a module is imported without an alias (e.g., `IMPORT "math"`),
    /// making its variables and functions directly accessible in the global scope.
    fn inline_into(&self, env: &mut Environment) -> InterpResult<()>;
}

/// A module loaded from a user-provided `.hi` file.
///
/// It contains an environment with variables and functions defined in the file.
pub struct UserModule {
    /// The environment captured from the module file.
    pub env: Rc<RefCell<Environment>>,
}

impl Module for UserModule {
    fn get_var(&self, name: Symbol) -> Option<Value> {
        self.env.borrow().lookup(name).and_then(|b| {
            if let Binding::Variable(v) = b {
                Some(v)
            } else {
                None
            }
        })
    }

    fn call_function(
        &self,
        name: Symbol,
        args: &[Value],
        interp: &mut Interpreter,
        span: &Span,
    ) -> InterpResult<Value> {
        let binding = self
            .env
            .borrow()
            .lookup(name)
            .ok_or_else(|| InterpError::Runtime {
                span: *span,
                message: format!("Function '{}' not found in module", name),
            })?;
        match binding {
            Binding::UserFunction(params, body) => {
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
                let mut child = Environment::child(self.env.clone());
                for (p, v) in params.iter().zip(args) {
                    child.define(*p, Binding::Variable(v.clone()));
                }
                let old_env = std::mem::replace(&mut interp.env, Rc::new(RefCell::new(child)));
                let old_ret = interp.return_value.take();
                for stmt in body {
                    interp.execute_stmt(&stmt)?;
                    if interp.return_value.is_some() {
                        break;
                    }
                }
                let res = interp.return_value.take().unwrap_or(Value::Nil);
                interp.env = old_env;
                interp.return_value = old_ret;
                Ok(res)
            }
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!("'{}' is not a user function", name),
            }),
        }
    }

    fn inline_into(&self, env: &mut Environment) -> InterpResult<()> {
        for (sym, binding) in self.env.borrow().bindings() {
            env.define(sym, binding.clone());
        }
        Ok(())
    }
}

/// A module implemented in Rust, providing built-in functionality.
///
/// It contains a map of exported variables and a map of exported functions.
pub struct BuiltinModule {
    /// Variables exported by the module (e.g., constants like `PI`).
    pub vars: HashMap<Symbol, Value>,
    /// Functions exported by the module (e.g., `sin`, `cos`).
    pub funcs: HashMap<Symbol, BuiltinFn>,
}

impl Module for BuiltinModule {
    fn get_var(&self, name: Symbol) -> Option<Value> {
        self.vars.get(&name).cloned()
    }

    fn call_function(
        &self,
        name: Symbol,
        args: &[Value],
        interp: &mut Interpreter,
        span: &Span,
    ) -> InterpResult<Value> {
        let func = self.funcs.get(&name).ok_or_else(|| InterpError::Runtime {
            span: *span,
            message: format!("Function '{}' not found in module", name),
        })?;
        func(interp, args, span)
    }

    fn inline_into(&self, env: &mut Environment) -> InterpResult<()> {
        for (sym, v) in &self.vars {
            env.define(*sym, Binding::Variable(v.clone()));
        }
        for (sym, f) in &self.funcs {
            env.define(*sym, Binding::BuiltinFunction(f.clone()));
        }
        Ok(())
    }
}
