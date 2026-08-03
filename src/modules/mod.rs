pub mod core;
pub mod io;
pub mod math;
pub mod strings;

use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::{Environment, Interpreter};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

pub type BuiltinFn = Rc<dyn Fn(&mut Interpreter, &[Value], &Span) -> InterpResult<Value>>;

pub trait Module {
    fn get_var(&self, name: &str) -> Option<Value>;
    fn call_function(
        &self,
        name: &str,
        args: &[Value],
        interp: &mut Interpreter,
        span: &Span,
    ) -> InterpResult<Value>;

    fn inline_into(&self, interp: &mut Interpreter) -> InterpResult<()>;
}

pub struct UserModule {
    pub env: Environment,
}

impl Module for UserModule {
    fn get_var(&self, name: &str) -> Option<Value> {
        self.env.vars.get(name).cloned()
    }

    fn call_function(
        &self,
        name: &str,
        args: &[Value],
        interp: &mut Interpreter,
        span: &Span,
    ) -> InterpResult<Value> {
        if let Some((params, body)) = self.env.functions.get(name).cloned() {
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

            let mut child_env = self.env.child();
            for (param, arg) in params.iter().zip(args) {
                child_env.declare(param.clone(), arg.clone());
            }

            let old_env = std::mem::replace(&mut interp.env, child_env);
            let old_return = interp.return_value.take();

            for stmt in body {
                interp.execute_stmt(&stmt)?;
                if interp.return_value.is_some() {
                    break;
                }
            }
            let result = interp.return_value.take().unwrap_or(Value::Nil);

            interp.env = old_env;
            interp.return_value = old_return;
            Ok(result)
        } else {
            Err(InterpError::Runtime {
                span: *span,
                message: format!("Function '{}' not found", name),
            })
        }
    }

    fn inline_into(&self, interp: &mut Interpreter) -> InterpResult<()> {
        for (k, v) in &self.env.vars {
            interp.env.declare(k.clone(), v.clone());
        }
        for (k, v) in &self.env.functions {
            interp.env.functions.insert(k.clone(), v.clone());
        }
        Ok(())
    }
}

pub struct BuiltinModule {
    pub vars: HashMap<String, Value>,
    pub funcs: HashMap<String, BuiltinFn>,
}

impl Module for BuiltinModule {
    fn get_var(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned()
    }

    fn call_function(
        &self,
        name: &str,
        args: &[Value],
        interp: &mut Interpreter,
        span: &Span,
    ) -> InterpResult<Value> {
        let func = self.funcs.get(name).ok_or_else(|| InterpError::Runtime {
            span: *span,
            message: format!("Function '{}' not found in module", name),
        })?;
        func(interp, args, span)
    }

    fn inline_into(&self, interp: &mut Interpreter) -> InterpResult<()> {
        for (k, v) in &self.vars {
            interp.env.declare(k.clone(), v.clone());
        }
        for (k, f) in &self.funcs {
            interp.global_functions.insert(k.clone(), f.clone());
        }
        Ok(())
    }
}
