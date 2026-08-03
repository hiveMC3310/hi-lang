use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::Value;
use rand::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Creates a new built-in module for random operations.
pub fn new_random_module() -> BuiltinModule {
    let vars = HashMap::new();
    let mut funcs: HashMap<String, BuiltinFn> = HashMap::new();

    funcs.insert("randint".to_string(), Rc::new(randint_fn));
    funcs.insert("randfloat".to_string(), Rc::new(randfloat_fn));
    funcs.insert("randbytes".to_string(), Rc::new(randbytes_fn));
    funcs.insert("shuffle".to_string(), Rc::new(shuffle_fn));
    funcs.insert("choice".to_string(), Rc::new(choice_fn));

    BuiltinModule { funcs, vars }
}

/// randint(start, end) – returns a random integer in the inclusive range [start, end].
fn randint_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "randint() expects 2 arguments (start, end), got {}",
                args.len()
            ),
        });
    }
    let start = match &args[0] {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "randint() start must be an integer, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let end = match &args[1] {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "randint() end must be an integer, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };
    if start > end {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("randint() start ({}) must be <= end ({})", start, end),
        });
    }
    let mut rng = rand::rng();
    let val = rng.random_range(start..=end);
    Ok(Value::Int(val))
}

/// randfloat() – returns a random float in the range [0.0, 1.0).
fn randfloat_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if !args.is_empty() {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("randfloat() expects no arguments, got {}", args.len()),
        });
    }
    let mut rng = rand::rng();
    let val: f64 = rng.random();
    Ok(Value::Float(val))
}

/// randbytes(n) – returns a list of n random bytes (integers 0..255).
fn randbytes_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("randbytes() expects 1 argument (n), got {}", args.len()),
        });
    }
    let n = match &args[0] {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "randbytes() n must be an integer, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    if n < 0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("randbytes() n must be non-negative, got {}", n),
        });
    }
    let mut rng = rand::rng();
    let mut bytes = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let byte: u8 = rng.random();
        bytes.push(Value::Int(byte as i64));
    }
    Ok(Value::List(Rc::new(RefCell::new(bytes))))
}

/// shuffle(list) – returns a new list with elements shuffled (COW).
fn shuffle_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("shuffle() expects 1 argument (list), got {}", args.len()),
        });
    }
    let list_val = &args[0];
    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "shuffle() expects a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    // COW: if reference count is 1, mutate in place; otherwise copy.
    let new_list_rc = if Rc::strong_count(&list_rc) == 1 {
        let mut list = list_rc.borrow_mut();
        let mut rng = rand::rng();
        list.shuffle(&mut rng);
        list_rc.clone()
    } else {
        let mut new_vec = list_rc.borrow().clone();
        let mut rng = rand::rng();
        new_vec.shuffle(&mut rng);
        Rc::new(RefCell::new(new_vec))
    };
    Ok(Value::List(new_list_rc))
}

/// choice(list) – returns a random element from the list, or nil if empty.
fn choice_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("choice() expects 1 argument (list), got {}", args.len()),
        });
    }
    let list_val = &args[0];
    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "choice() expects a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };
    let list = list_rc.borrow();
    if list.is_empty() {
        return Ok(Value::Nil);
    }
    let mut rng = rand::rng();
    let idx = rng.random_range(0..list.len());
    Ok(list[idx].clone())
}
