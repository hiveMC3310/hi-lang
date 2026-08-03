use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

// ---------- Just Hello ----------
pub fn hello_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("hello() doesn't expect args, got {}", args.len()),
        });
    }
    println!("Hello, World!");
    Ok(Value::Nil)
}

// ---------- Lambda ----------
pub fn call_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.is_empty() {
        return Err(InterpError::Runtime {
            span: *span,
            message: "call() expects at least 1 argument (function)".to_string(),
        });
    }

    let func_val = &args[0];
    let func_name = match func_val {
        Value::Function(name) => name,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "call() expects a function, got {}",
                    crate::utils::type_name(&func_val)
                ),
            });
        }
    };

    let (params, body) =
        interp
            .env
            .get_function(&func_name)
            .ok_or_else(|| InterpError::Runtime {
                span: *span,
                message: format!("Function '{}' not found", func_name),
            })?;
    let call_args = &args[1..];
    if call_args.len() != params.len() {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "Function '{}' expects {} arguments, got {}",
                func_name,
                params.len(),
                call_args.len()
            ),
        });
    }

    let arg_values: Vec<Value> = call_args.iter().cloned().collect();

    let mut child_env = interp.env.child();
    for (param, arg_val) in params.iter().zip(arg_values) {
        child_env.declare(param.clone(), arg_val);
    }
    let old_env = std::mem::replace(&mut interp.env, child_env);
    let old_return = interp.return_value.take();

    for stmt in body {
        interp.execute_stmt(&stmt)?;
        if interp.return_value.is_some() || interp.break_flag {
            break;
        }
    }
    let result = interp.return_value.take().unwrap_or(Value::Nil);
    interp.env = old_env;
    interp.return_value = old_return;
    Ok(result)
}

// ---------- String, List and Dict methods ----------
pub fn len_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("len() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    let len = match val {
        Value::String(s) => s.len(),
        Value::List(l) => l.borrow().len(),
        Value::Dict(d) => d.borrow().len(),
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "len() expects string, list, or dict, got {}",
                    crate::utils::type_name(&val)
                ),
            });
        }
    };
    Ok(Value::Int(len as i64))
}

pub fn keys_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("keys() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::Dict(dict) => {
            let keys = dict.borrow().keys().cloned().collect::<Vec<_>>();
            Ok(Value::List(Rc::new(RefCell::new(keys))))
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "keys() expects a dict, got {}",
                crate::utils::type_name(&val)
            ),
        }),
    }
}

pub fn values_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("values() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::Dict(dict) => {
            let keys = dict.borrow().values().cloned().collect::<Vec<_>>();
            Ok(Value::List(Rc::new(RefCell::new(keys))))
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "values() expects a dict, got {}",
                crate::utils::type_name(&val)
            ),
        }),
    }
}

pub fn append_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("append() expects 2 arguments, got {}", args.len()),
        });
    }
    let list_val = &args[0];
    let elem_val = &args[1];
    match list_val {
        Value::List(list_rc) => {
            // COW
            let new_list_rc = if Rc::strong_count(&list_rc) == 1 {
                list_rc.borrow_mut().push(elem_val.clone());
                list_rc.clone()
            } else {
                let mut new_vec = list_rc.borrow().clone();
                new_vec.push(elem_val.clone());
                Rc::new(RefCell::new(new_vec))
            };
            Ok(Value::List(new_list_rc))
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "append() expects a list, got {}",
                crate::utils::type_name(&list_val)
            ),
        }),
    }
}

pub fn insert_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("insert() expects 3 arguments, got {}", args.len()),
        });
    }
    let list_val = &args[0];
    let idx_val = &args[1];
    let elem_val = &args[2];
    let idx = match idx_val {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "insert() index must be an integer".to_string(),
            });
        }
    };
    match list_val {
        Value::List(list_rc) => {
            let len = list_rc.borrow().len();
            if idx < 0 || idx as usize > len {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!("insert() index {} out of bounds (len={})", idx, len),
                });
            }
            let new_list_rc = if Rc::strong_count(&list_rc) == 1 {
                list_rc.borrow_mut().insert(idx as usize, elem_val.clone());
                list_rc.clone()
            } else {
                let mut new_vec = list_rc.borrow().clone();
                new_vec.insert(idx as usize, elem_val.clone());
                Rc::new(RefCell::new(new_vec))
            };
            Ok(Value::List(new_list_rc))
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "insert() expects a list, got {}",
                crate::utils::type_name(&list_val)
            ),
        }),
    }
}

pub fn remove_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("remove() expects 2 arguments, got {}", args.len()),
        });
    }
    let base_val = &args[0];
    let idx_val = &args[1];
    match base_val {
        Value::List(list_rc) => {
            let idx = match idx_val {
                Value::Int(i) => *i,
                _ => {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "remove() index must be an integer".to_string(),
                    });
                }
            };
            let len = list_rc.borrow().len();
            if idx < 0 || idx as usize >= len {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!("remove() index {} out of bounds (len={})", idx, len),
                });
            }
            let new_list_rc = if Rc::strong_count(&list_rc) == 1 {
                list_rc.borrow_mut().remove(idx as usize);
                list_rc.clone()
            } else {
                let mut new_vec = list_rc.borrow().clone();
                new_vec.remove(idx as usize);
                Rc::new(RefCell::new(new_vec))
            };
            Ok(Value::List(new_list_rc))
        }
        Value::Dict(dict_rc) => {
            if !idx_val.is_hashable() {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: "remove() key must be hashable".to_string(),
                });
            }
            let mut dict_ref = dict_rc.borrow_mut();
            if dict_ref.remove(&idx_val).is_none() {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!("remove() key {:?} not found", idx_val),
                });
            }
            Ok(Value::Nil)
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "remove() expects a list or dict, got {}",
                crate::utils::type_name(&base_val)
            ),
        }),
    }
}

pub fn contains_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("contains() expects 2 arguments, got {}", args.len()),
        });
    }
    let base = &args[0];
    let elem = &args[1];
    let result = match (&base, &elem) {
        (Value::String(s), Value::String(sub)) => s.contains(sub),
        (Value::List(l_rc), val) => l_rc.borrow().contains(&val),
        (Value::Dict(d_rc), key) => {
            if !key.is_hashable() {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: "Dictionary key must be hashable".to_string(),
                });
            }
            d_rc.borrow().contains_key(key)
        }
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "contains() expects string/string or list/element, or dict/key, got {} and {}",
                    crate::utils::type_name(&base),
                    crate::utils::type_name(&elem)
                ),
            });
        }
    };
    Ok(Value::Bool(result))
}

pub fn indexof_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("indexof() expects 2 arguments, got {}", args.len()),
        });
    }
    let base = &args[0];
    let elem = &args[1];
    let result = match (base, elem) {
        (Value::String(s), Value::String(sub)) => match s.find(sub) {
            Some(pos) => Value::Int(pos as i64),
            None => Value::Int(-1),
        },
        (Value::List(list_rc), val) => match list_rc.borrow().iter().position(|v| v == val) {
            Some(pos) => Value::Int(pos as i64),
            None => Value::Int(-1),
        },
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "indexof() expects string/string or list/element, got {} and {}",
                    crate::utils::type_name(&base),
                    crate::utils::type_name(&elem)
                ),
            });
        }
    };
    Ok(result)
}

pub fn concat_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }

    let first_type = match &args[0] {
        Value::String(_) => "string",
        Value::List(_) => "list",
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "concat() expects all arguments to be strings or all to be lists, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };

    match first_type {
        "string" => {
            let mut result = String::new();
            for val in args {
                match val {
                    Value::String(s) => result.push_str(s),
                    _ => {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: "concat() expects all arguments to be strings".to_string(),
                        });
                    }
                }
            }
            Ok(Value::String(result))
        }
        "list" => {
            let mut result_vec = Vec::new();
            for val in args {
                match val {
                    Value::List(l) => {
                        result_vec.extend(l.borrow().iter().cloned());
                    }
                    _ => {
                        return Err(InterpError::Runtime {
                            span: *span,
                            message: "concat() expects all arguments to be lists".to_string(),
                        });
                    }
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(result_vec))))
        }
        _ => unreachable!(),
    }
}

pub fn slice_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("slice() expects 3 arguments, got {}", args.len()),
        });
    }
    let list_val = &args[0];
    let start_val = &args[1];
    let len_val = &args[2];
    let list_rc = match list_val {
        Value::List(l) => l.clone(),
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "slice() expects a list, got {}",
                    crate::utils::type_name(&list_val)
                ),
            });
        }
    };
    let start = match start_val {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "slice() start must be integer".to_string(),
            });
        }
    };
    let len = match len_val {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "slice() length must be integer".to_string(),
            });
        }
    };
    if start < 0 || len < 0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: "slice() start and length must be non-negative".to_string(),
        });
    }
    let list = list_rc.borrow();
    let start = start as usize;
    let len = len as usize;
    let end = start + len;
    let result = if start >= list.len() {
        Vec::new()
    } else if end > list.len() {
        list[start..].to_vec()
    } else {
        list[start..end].to_vec()
    };
    Ok(Value::List(Rc::new(RefCell::new(result))))
}

pub fn reverse_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("reverse() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
        Value::List(list_rc) => {
            let mut vec = list_rc.borrow().clone();
            vec.reverse();
            Ok(Value::List(Rc::new(RefCell::new(vec))))
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "reverse() expects a string or list, got {}",
                crate::utils::type_name(&val)
            ),
        }),
    }
}

pub fn put_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "put() expects 3 arguments (dict, key, value), got {}",
                args.len()
            ),
        });
    }
    let dict_val = &args[0];
    let key = &args[1];
    let value = &args[2];
    let dict_rc = match dict_val {
        Value::Dict(d) => d.clone(),
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "put() expects a dict, got {}",
                    crate::utils::type_name(&dict_val)
                ),
            });
        }
    };
    if !key.is_hashable() {
        return Err(InterpError::Runtime {
            span: *span,
            message: "put() key must be hashable".to_string(),
        });
    }
    dict_rc.borrow_mut().insert(key.clone(), value.clone());
    Ok(Value::Nil)
}

pub fn get_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("get() expects 2 arguments (dict, key), got {}", args.len()),
        });
    }
    let dict_val = &args[0];
    let key = &args[1];
    let dict_rc = match dict_val {
        Value::Dict(d) => d,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "get() expects a dict, got {}",
                    crate::utils::type_name(&dict_val)
                ),
            });
        }
    };
    if !key.is_hashable() {
        return Err(InterpError::Runtime {
            span: *span,
            message: "get() key must be hashable".to_string(),
        });
    }
    match dict_rc.borrow().get(&key) {
        Some(v) => Ok(v.clone()),
        None => Ok(Value::Nil),
    }
}

// ---------- Converts ----------
pub fn tostring_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("tostring() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    Ok(Value::String(val.to_string()))
}

pub fn toint_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("toint() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => {
            let trimmed = s.trim();
            if let Ok(i) = trimmed.parse::<i64>() {
                Ok(Value::Int(i))
            } else {
                Err(InterpError::Runtime {
                    span: *span,
                    message: format!("Cannot convert '{}' to integer", s),
                })
            }
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "toint() expects number or string, got {}",
                crate::utils::type_name(&val)
            ),
        }),
    }
}

pub fn tofloat_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("tofloat() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => {
            let trimmed = s.trim();
            if let Ok(f) = trimmed.parse::<f64>() {
                Ok(Value::Float(f))
            } else {
                Err(InterpError::Runtime {
                    span: *span,
                    message: format!("Cannot convert '{}' to float", s),
                })
            }
        }
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "tofloat() expects number or string, got {}",
                crate::utils::type_name(&val)
            ),
        }),
    }
}

pub fn typeof_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("typeof() expects 1 argument, got {}", args.len()),
        });
    }
    let val = &args[0];
    let type_name = crate::utils::type_name(&val);
    Ok(Value::String(type_name.to_string()))
}
