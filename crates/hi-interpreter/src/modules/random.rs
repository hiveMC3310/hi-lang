use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use rand::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// ---------- randint ----------
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
inventory::submit! {
    ModuleFunction {
        module: "random",
        name: "randint",
        params: &["start", "end"],
        doc: "Returns a random integer between start and end (inclusive).",
        func: randint_fn,
    }
}

// ---------- randfloat ----------
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
inventory::submit! {
    ModuleFunction {
        module: "random",
        name: "randfloat",
        params: &[],
        doc: "Returns a random float in the range [0.0, 1.0).",
        func: randfloat_fn,
    }
}

// ---------- randbytes ----------
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
inventory::submit! {
    ModuleFunction {
        module: "random",
        name: "randbytes",
        params: &["n"],
        doc: "Returns a list of n random bytes (each as an integer 0-255).",
        func: randbytes_fn,
    }
}

// ---------- shuffle ----------
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

    let new_list_rc = if Rc::strong_count(list_rc) == 1 {
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
inventory::submit! {
    ModuleFunction {
        module: "random",
        name: "shuffle",
        params: &["list"],
        doc: "Shuffles the elements of a list randomly and returns the shuffled list.",
        func: shuffle_fn,
    }
}

// ---------- choice ----------
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
inventory::submit! {
    ModuleFunction {
        module: "random",
        name: "choice",
        params: &["list"],
        doc: "Selects and returns a random element from the list, or nil if the list is empty.",
        func: choice_fn,
    }
}
