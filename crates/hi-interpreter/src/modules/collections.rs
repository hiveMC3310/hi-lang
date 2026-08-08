use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::{Binding, Interpreter};
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

// Вспомогательная функция для вызова функции через глобальный `call`.
fn call_func(
    interp: &mut Interpreter,
    func: &Value,
    args: &[Value],
    span: &Span,
) -> InterpResult<Value> {
    let call_sym = hi_common::intern("call");
    let binding = interp
        .env
        .borrow()
        .lookup(call_sym)
        .ok_or_else(|| InterpError::Internal("call function not found".to_string()))?;
    match binding {
        Binding::BuiltinFunction(f) => {
            let mut call_args = vec![func.clone()];
            call_args.extend_from_slice(args);
            f(interp, &call_args, span)
        }
        _ => Err(InterpError::Internal(
            "call is not a builtin function".to_string(),
        )),
    }
}

// ---------- map ----------
fn map_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "map() expects 2 arguments (function, list), got {}",
                args.len()
            ),
        });
    }
    let func = &args[0];
    let list_val = &args[1];

    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "map() second argument must be a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    let list = list_rc.borrow();
    let mut result = Vec::with_capacity(list.len());

    for elem in list.iter() {
        let res = call_func(interp, func, &[elem.clone()], span)?;
        result.push(res);
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}
inventory::submit! {
    ModuleFunction {
        module: "collections",
        name: "map",
        params: &["func", "list"],
        doc: "Applies a function to each element of a list and returns a new list. The function must accept one argument and return a value.",
        func: map_fn,
    }
}

// ---------- sort ----------
fn sort_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("sort() expects 1 argument (list), got {}", args.len()),
        });
    }
    let list_val = &args[0];

    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "sort() expects a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    let mut vec = list_rc.borrow().clone();

    if vec.is_empty() {
        return Ok(Value::List(Rc::new(RefCell::new(vec))));
    }

    let first_type = crate::utils::type_name(&vec[0]);
    for v in &vec {
        if crate::utils::type_name(v) != first_type {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "sort() list must contain elements of the same type, found {} and {}",
                    first_type,
                    crate::utils::type_name(v)
                ),
            });
        }
    }

    match first_type {
        "integer" => vec.sort_by(|a, b| {
            if let (Value::Int(ai), Value::Int(bi)) = (a, b) {
                ai.cmp(bi)
            } else {
                std::cmp::Ordering::Equal
            }
        }),
        "float" => vec.sort_by(|a, b| {
            if let (Value::Float(af), Value::Float(bf)) = (a, b) {
                af.partial_cmp(bf).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                std::cmp::Ordering::Equal
            }
        }),
        "string" => vec.sort_by(|a, b| {
            if let (Value::String(as_), Value::String(bs_)) = (a, b) {
                as_.cmp(bs_)
            } else {
                std::cmp::Ordering::Equal
            }
        }),
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("sort() cannot sort elements of type {}", first_type),
            });
        }
    }

    Ok(Value::List(Rc::new(RefCell::new(vec))))
}
inventory::submit! {
    ModuleFunction {
        module: "collections",
        name: "sort",
        params: &["list"],
        doc: "Returns a new sorted list. The list must contain only integers, floats, or strings (all of the same type).",
        func: sort_fn,
    }
}

// ---------- filter ----------
fn filter_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "filter() expects 2 arguments (predicate, list), got {}",
                args.len()
            ),
        });
    }
    let pred = &args[0];
    let list_val = &args[1];

    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "filter() second argument must be a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    let list = list_rc.borrow();
    let mut result = Vec::new();

    for elem in list.iter() {
        let res = call_func(interp, pred, &[elem.clone()], span)?;
        if res.as_bool() {
            result.push(elem.clone());
        }
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}
inventory::submit! {
    ModuleFunction {
        module: "collections",
        name: "filter",
        params: &["predicate", "list"],
        doc: "Filters a list by a predicate function. The predicate must accept one argument and return a boolean.",
        func: filter_fn,
    }
}

// ---------- reduce ----------
fn reduce_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "reduce() expects 3 arguments (function, list, initial), got {}",
                args.len()
            ),
        });
    }
    let func = &args[0];
    let list_val = &args[1];
    let initial = &args[2];

    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "reduce() second argument must be a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    let list = list_rc.borrow();
    let mut acc = initial.clone();

    for elem in list.iter() {
        acc = call_func(interp, func, &[acc, elem.clone()], span)?;
    }

    Ok(acc)
}
inventory::submit! {
    ModuleFunction {
        module: "collections",
        name: "reduce",
        params: &["func", "list", "initial"],
        doc: "Reduces a list to a single value using a binary function. The function must accept two arguments (accumulator, element) and return a new accumulator.",
        func: reduce_fn,
    }
}

// ---------- any ----------
fn any_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "any() expects 2 arguments (predicate, list), got {}",
                args.len()
            ),
        });
    }
    let pred = &args[0];
    let list_val = &args[1];

    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "any() second argument must be a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    let list = list_rc.borrow();
    for elem in list.iter() {
        let res = call_func(interp, pred, &[elem.clone()], span)?;
        if res.as_bool() {
            return Ok(Value::Bool(true));
        }
    }
    Ok(Value::Bool(false))
}
inventory::submit! {
    ModuleFunction {
        module: "collections",
        name: "any",
        params: &["predicate", "list"],
        doc: "Returns true if the predicate returns true for at least one element of the list. The predicate must accept one argument and return a boolean.",
        func: any_fn,
    }
}

// ---------- all ----------
fn all_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "all() expects 2 arguments (predicate, list), got {}",
                args.len()
            ),
        });
    }
    let pred = &args[0];
    let list_val = &args[1];

    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "all() second argument must be a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    let list = list_rc.borrow();
    for elem in list.iter() {
        let res = call_func(interp, pred, &[elem.clone()], span)?;
        if !res.as_bool() {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}
inventory::submit! {
    ModuleFunction {
        module: "collections",
        name: "all",
        params: &["predicate", "list"],
        doc: "Returns true if the predicate returns true for every element of the list. The predicate must accept one argument and return a boolean.",
        func: all_fn,
    }
}

// ---------- find ----------
fn find_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "find() expects 2 arguments (predicate, list), got {}",
                args.len()
            ),
        });
    }
    let pred = &args[0];
    let list_val = &args[1];

    let list_rc = match list_val {
        Value::List(l) => l,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "find() second argument must be a list, got {}",
                    crate::utils::type_name(list_val)
                ),
            });
        }
    };

    let list = list_rc.borrow();
    for elem in list.iter() {
        let res = call_func(interp, pred, &[elem.clone()], span)?;
        if res.as_bool() {
            return Ok(elem.clone());
        }
    }
    Ok(Value::Nil)
}
inventory::submit! {
    ModuleFunction {
        module: "collections",
        name: "find",
        params: &["predicate", "list"],
        doc: "Returns the first element for which the predicate returns true, or nil if none found. The predicate must accept one argument and return a boolean.",
        func: find_fn,
    }
}
