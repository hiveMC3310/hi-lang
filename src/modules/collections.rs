use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Creates a new built-in module containing collection helper functions.
pub fn new_collections_module() -> BuiltinModule {
    let vars = HashMap::new();
    let mut funcs: HashMap<String, BuiltinFn> = HashMap::new();

    funcs.insert("map".to_string(), Rc::new(map_fn));
    funcs.insert("sort".to_string(), Rc::new(sort_fn));
    funcs.insert("filter".to_string(), Rc::new(filter_fn));
    funcs.insert("reduce".to_string(), Rc::new(reduce_fn));
    funcs.insert("any".to_string(), Rc::new(any_fn));
    funcs.insert("all".to_string(), Rc::new(all_fn));
    funcs.insert("find".to_string(), Rc::new(find_fn));

    BuiltinModule { funcs, vars }
}

/// Helper function to call a given function value with arguments.
/// It uses the global `call` built-in to perform the actual invocation.
fn call_func(
    interp: &mut Interpreter,
    func: &Value,
    args: &[Value],
    span: &Span,
) -> InterpResult<Value> {
    let call_fn = interp
        .global_functions
        .get("call")
        .ok_or_else(|| InterpError::Internal("call function not found".to_string()))?
        .clone();

    let mut call_args = vec![func.clone()];
    call_args.extend_from_slice(args);
    call_fn(interp, &call_args, span)
}

/// map(func, list) – applies `func` to each element and returns a new list.
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

/// sort(list) – returns a sorted copy of the list (ascending order).
/// Elements must all be of the same type: integer, float, or string.
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

    // Always create a copy (COW is not used here to simplify borrowing)
    let mut vec = list_rc.borrow().clone();

    if vec.is_empty() {
        return Ok(Value::List(Rc::new(RefCell::new(vec))));
    }

    // Verify that all elements have the same type
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

/// filter(pred, list) – returns a new list containing elements for which `pred` returns TRUE.
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

/// reduce(func, list, initial) – folds the list by applying `func(acc, elem)` and returns the final accumulator.
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

/// any(pred, list) – returns TRUE if at least one element satisfies `pred`.
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

/// all(pred, list) – returns TRUE if all elements satisfy `pred`.
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

/// find(pred, list) – returns the first element that satisfies `pred`, or nil if none found.
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
