use crate::ast::{Expr, Span};
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::{FileHandle, Value};
use std::cell::RefCell;
use std::io::{BufRead, Read, Write};
use std::rc::Rc;

pub type BuiltinFn = Rc<dyn Fn(&mut Interpreter, &[Expr], &Span) -> InterpResult<Value>>;

pub struct Builtin {}

impl Builtin {
    pub fn hello_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 0 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("hello() doesn't expect args, got {}", args.len()),
            });
        }
        println!("Hello, World!");
        Ok(Value::Nil)
    }

    // ---------- String, List and Dict methods ----------
    pub fn len_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("len() expects 1 argument, got {}", args.len()),
            });
        }
        let val = interp.eval_expr(&args[0])?;
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

    pub fn keys_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("keys() expects 1 argument, got {}", args.len()),
            });
        }
        let val = interp.eval_expr(&args[0])?;
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

    pub fn values_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("values() expects 1 argument, got {}", args.len()),
            });
        }
        let val = interp.eval_expr(&args[0])?;
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

    pub fn append_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("append() expects 2 arguments, got {}", args.len()),
            });
        }
        let list_val = interp.eval_expr(&args[0])?;
        let elem_val = interp.eval_expr(&args[1])?;
        match list_val {
            Value::List(list_rc) => {
                // COW
                let new_list_rc = if Rc::strong_count(&list_rc) == 1 {
                    list_rc.borrow_mut().push(elem_val);
                    list_rc.clone()
                } else {
                    let mut new_vec = list_rc.borrow().clone();
                    new_vec.push(elem_val);
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

    pub fn insert_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 3 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("insert() expects 3 arguments, got {}", args.len()),
            });
        }
        let list_val = interp.eval_expr(&args[0])?;
        let idx_val = interp.eval_expr(&args[1])?;
        let elem_val = interp.eval_expr(&args[2])?;
        let idx = match idx_val {
            Value::Int(i) => i,
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
                    list_rc.borrow_mut().insert(idx as usize, elem_val);
                    list_rc.clone()
                } else {
                    let mut new_vec = list_rc.borrow().clone();
                    new_vec.insert(idx as usize, elem_val);
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

    pub fn remove_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("remove() expects 2 arguments, got {}", args.len()),
            });
        }
        let base_val = interp.eval_expr(&args[0])?;
        let idx_val = interp.eval_expr(&args[1])?;
        match base_val {
            Value::List(list_rc) => {
                let idx = match idx_val {
                    Value::Int(i) => i,
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

    pub fn contains_fn(
        interp: &mut Interpreter,
        args: &[Expr],
        span: &Span,
    ) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("contains() expects 2 arguments, got {}", args.len()),
            });
        }
        let base = interp.eval_expr(&args[0])?;
        let elem = interp.eval_expr(&args[1])?;
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

    pub fn indexof_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("indexof() expects 2 arguments, got {}", args.len()),
            });
        }
        let base = interp.eval_expr(&args[0])?;
        let elem = interp.eval_expr(&args[1])?;
        let result = match (&base, &elem) {
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

    pub fn split_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("split() expects 2 arguments, got {}", args.len()),
            });
        }
        let base = interp.eval_expr(&args[0])?;
        let delim = interp.eval_expr(&args[1])?;
        match (&base, &delim) {
            (Value::String(s), Value::String(d)) => {
                let parts: Vec<Value> = s
                    .split(d)
                    .map(|part| Value::String(part.to_string()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(parts))))
            }
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "split() expects two strings, got {} and {}",
                    crate::utils::type_name(&base),
                    crate::utils::type_name(&delim)
                ),
            }),
        }
    }

    pub fn concat_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("concat() expects 2 arguments, got {}", args.len()),
            });
        }
        let left = interp.eval_expr(&args[0])?;
        let right = interp.eval_expr(&args[1])?;
        match (&left, &right) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::String(format!("{}{}", s1, s2))),
            (Value::List(l1), Value::List(l2)) => {
                let mut new_vec = l1.borrow().clone();
                new_vec.extend(l2.borrow().iter().cloned());
                Ok(Value::List(Rc::new(RefCell::new(new_vec))))
            }
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "concat() expects two strings or two lists, got {} and {}",
                    crate::utils::type_name(&left),
                    crate::utils::type_name(&right)
                ),
            }),
        }
    }

    pub fn replace_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 3 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("replace() expects 3 arguments, got {}", args.len()),
            });
        }
        let base = interp.eval_expr(&args[0])?;
        let old = interp.eval_expr(&args[1])?;
        let new = interp.eval_expr(&args[2])?;
        match (&base, &old, &new) {
            (Value::String(s), Value::String(old_str), Value::String(new_str)) => {
                Ok(Value::String(s.replace(old_str, &new_str)))
            }
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "replace() expects three strings, got {}, {}, {}",
                    crate::utils::type_name(&base),
                    crate::utils::type_name(&old),
                    crate::utils::type_name(&new)
                ),
            }),
        }
    }

    pub fn substr_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 3 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("substr() expects 3 arguments, got {}", args.len()),
            });
        }
        let s_val = interp.eval_expr(&args[0])?;
        let start_val = interp.eval_expr(&args[1])?;
        let len_val = interp.eval_expr(&args[2])?;
        let s = match s_val {
            Value::String(s) => s,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "substr() expects a string, got {}",
                        crate::utils::type_name(&s_val)
                    ),
                });
            }
        };
        let start = match start_val {
            Value::Int(i) => i,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: "substr() start must be integer".to_string(),
                });
            }
        };
        let len = match len_val {
            Value::Int(i) => i,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: "substr() length must be integer".to_string(),
                });
            }
        };
        if start < 0 || len < 0 {
            return Err(InterpError::Runtime {
                span: *span,
                message: "substr() start and length must be non-negative".to_string(),
            });
        }
        let start = start as usize;
        let len = len as usize;
        let end = start + len;
        let result = if start >= s.len() {
            String::new()
        } else if end > s.len() {
            s[start..].to_string()
        } else {
            s[start..end].to_string()
        };
        Ok(Value::String(result))
    }

    pub fn slice_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 3 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("slice() expects 3 arguments, got {}", args.len()),
            });
        }
        let list_val = interp.eval_expr(&args[0])?;
        let start_val = interp.eval_expr(&args[1])?;
        let len_val = interp.eval_expr(&args[2])?;
        let list_rc = match list_val {
            Value::List(l) => l,
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
            Value::Int(i) => i,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: "slice() start must be integer".to_string(),
                });
            }
        };
        let len = match len_val {
            Value::Int(i) => i,
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

    pub fn reverse_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("reverse() expects 1 argument, got {}", args.len()),
            });
        }
        let val = interp.eval_expr(&args[0])?;
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

    pub fn starts_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("starts() expects 2 arguments, got {}", args.len()),
            });
        }
        let base = interp.eval_expr(&args[0])?;
        let prefix = interp.eval_expr(&args[1])?;
        match (&base, &prefix) {
            (Value::String(s), Value::String(p)) => Ok(Value::Bool(s.starts_with(p))),
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "starts() expects two strings, got {} and {}",
                    crate::utils::type_name(&base),
                    crate::utils::type_name(&prefix)
                ),
            }),
        }
    }

    pub fn ends_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("ends() expects 2 arguments, got {}", args.len()),
            });
        }
        let base = interp.eval_expr(&args[0])?;
        let suffix = interp.eval_expr(&args[1])?;
        match (&base, &suffix) {
            (Value::String(s), Value::String(suf)) => Ok(Value::Bool(s.ends_with(suf))),
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "ends() expects two strings, got {} and {}",
                    crate::utils::type_name(&base),
                    crate::utils::type_name(&suffix)
                ),
            }),
        }
    }

    pub fn upper_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("upper() expects 1 argument, got {}", args.len()),
            });
        }
        let val = interp.eval_expr(&args[0])?;
        match val {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "upper() expects a string, got {}",
                    crate::utils::type_name(&val)
                ),
            }),
        }
    }

    pub fn lower_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("lower() expects 1 argument, got {}", args.len()),
            });
        }
        let val = interp.eval_expr(&args[0])?;
        match val {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "lower() expects a string, got {}",
                    crate::utils::type_name(&val)
                ),
            }),
        }
    }

    pub fn trim_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("trim() expects 1 argument, got {}", args.len()),
            });
        }
        let val = interp.eval_expr(&args[0])?;
        match val {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "trim() expects a string, got {}",
                    crate::utils::type_name(&val)
                ),
            }),
        }
    }

    pub fn put_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 3 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "put() expects 3 arguments (dict, key, value), got {}",
                    args.len()
                ),
            });
        }
        let dict_val = interp.eval_expr(&args[0])?;
        let key = interp.eval_expr(&args[1])?;
        let value = interp.eval_expr(&args[2])?;
        let dict_rc = match dict_val {
            Value::Dict(d) => d,
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
        dict_rc.borrow_mut().insert(key, value);
        Ok(Value::Nil)
    }

    pub fn get_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("get() expects 2 arguments (dict, key), got {}", args.len()),
            });
        }
        let dict_val = interp.eval_expr(&args[0])?;
        let key = interp.eval_expr(&args[1])?;
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

    // ---------- Files methods ----------
    pub fn open_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "open() expects 2 arguments (path, mode), got {}",
                    args.len()
                ),
            });
        }
        let path_val = interp.eval_expr(&args[0])?;
        let mode_val = interp.eval_expr(&args[1])?;
        let path = match path_val {
            Value::String(s) => s,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: "open() path must be a string".to_string(),
                });
            }
        };
        let mode = match mode_val {
            Value::String(s) => s,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: "open() mode must be a string".to_string(),
                });
            }
        };

        let handle = match mode.as_str() {
            "r" => {
                let file = std::fs::File::open(&path).map_err(|e| InterpError::Runtime {
                    span: *span,
                    message: format!("Cannot open file '{}' for reading: {}", path, e),
                })?;
                FileHandle::new_reader(path, file)
            }
            "w" => {
                let file = std::fs::File::create(&path).map_err(|e| InterpError::Runtime {
                    span: *span,
                    message: format!("Cannot create file '{}' for writing: {}", path, e),
                })?;
                FileHandle::new_writer(path, file)
            }
            "a" => {
                let file = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&path)
                    .map_err(|e| InterpError::Runtime {
                        span: *span,
                        message: format!("Cannot open file '{}' for appending: {}", path, e),
                    })?;
                FileHandle::new_writer(path, file)
            }
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!("Invalid file mode '{}', use 'r', 'w', or 'a'", mode),
                });
            }
        };
        Ok(Value::File(Rc::new(RefCell::new(handle))))
    }

    pub fn close_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("close() expects 1 argument (file), got {}", args.len()),
            });
        }
        let file_val = interp.eval_expr(&args[0])?;
        let fh = match file_val {
            Value::File(fh) => fh,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "close() expects a file, got {}",
                        crate::utils::type_name(&file_val)
                    ),
                });
            }
        };
        let mut handle = fh.borrow_mut();
        if let Some(ref mut writer) = handle.writer {
            writer.flush().map_err(InterpError::Io)?;
        }
        handle.reader = None;
        handle.writer = None;
        Ok(Value::Nil)
    }

    pub fn read_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("read() expects 1 argument (file), got {}", args.len()),
            });
        }
        let file_val = interp.eval_expr(&args[0])?;
        let fh = match file_val {
            Value::File(fh) => fh,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "read() expects a file, got {}",
                        crate::utils::type_name(&file_val)
                    ),
                });
            }
        };
        let mut handle = fh.borrow_mut();
        let reader = handle.reader.as_mut().ok_or_else(|| InterpError::Runtime {
            span: *span,
            message: "File is not open for reading".to_string(),
        })?;
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(InterpError::Io)?;
        handle.eof = true;
        Ok(Value::String(content))
    }

    pub fn readln_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("readln() expects 1 argument (file), got {}", args.len()),
            });
        }
        let file_val = interp.eval_expr(&args[0])?;
        let fh = match file_val {
            Value::File(fh) => fh,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "readln() expects a file, got {}",
                        crate::utils::type_name(&file_val)
                    ),
                });
            }
        };
        let mut handle = fh.borrow_mut();
        let reader = handle.reader.as_mut().ok_or_else(|| InterpError::Runtime {
            span: *span,
            message: "File is not open for reading".to_string(),
        })?;
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).map_err(InterpError::Io)?;
        if bytes == 0 {
            handle.eof = true;
        }
        Ok(Value::String(line))
    }

    pub fn write_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "write() expects 2 arguments (file, value), got {}",
                    args.len()
                ),
            });
        }
        let file_val = interp.eval_expr(&args[0])?;
        let value = interp.eval_expr(&args[1])?;
        let fh = match file_val {
            Value::File(fh) => fh,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "write() expects a file, got {}",
                        crate::utils::type_name(&file_val)
                    ),
                });
            }
        };
        let mut handle = fh.borrow_mut();
        let writer = handle.writer.as_mut().ok_or_else(|| InterpError::Runtime {
            span: *span,
            message: "File is not open for writing".to_string(),
        })?;
        write!(writer, "{}", value).map_err(InterpError::Io)?;
        Ok(Value::Nil)
    }

    pub fn writeln_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 2 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "writeln() expects 2 arguments (file, value), got {}",
                    args.len()
                ),
            });
        }
        let file_val = interp.eval_expr(&args[0])?;
        let value = interp.eval_expr(&args[1])?;
        let fh = match file_val {
            Value::File(fh) => fh,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "writeln() expects a file, got {}",
                        crate::utils::type_name(&file_val)
                    ),
                });
            }
        };
        let mut handle = fh.borrow_mut();
        let writer = handle.writer.as_mut().ok_or_else(|| InterpError::Runtime {
            span: *span,
            message: "File is not open for writing".to_string(),
        })?;
        writeln!(writer, "{}", value).map_err(InterpError::Io)?;
        Ok(Value::Nil)
    }

    pub fn eof_fn(interp: &mut Interpreter, args: &[Expr], span: &Span) -> InterpResult<Value> {
        if args.len() != 1 {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!("eof() expects 1 argument (file), got {}", args.len()),
            });
        }
        let file_val = interp.eval_expr(&args[0])?;
        let fh = match file_val {
            Value::File(fh) => fh,
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "eof() expects a file, got {}",
                        crate::utils::type_name(&file_val)
                    ),
                });
            }
        };
        let handle = fh.borrow();
        let is_eof = handle.eof || handle.reader.is_none() && handle.writer.is_none();
        Ok(Value::Bool(is_eof))
    }
}
