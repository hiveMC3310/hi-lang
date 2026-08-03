use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::Value;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;

pub fn new_math_module() -> BuiltinModule {
    let mut vars = HashMap::new();
    vars.insert("PI".to_string(), Value::Float(PI));
    vars.insert("E".to_string(), Value::Float(std::f64::consts::E));

    let mut funcs: HashMap<String, BuiltinFn> = HashMap::new();
    funcs.insert("sin".to_string(), Rc::new(sin_fn));
    funcs.insert("cos".to_string(), Rc::new(cos_fn));
    funcs.insert("tan".to_string(), Rc::new(tan_fn));
    funcs.insert("asin".to_string(), Rc::new(asin_fn));
    funcs.insert("acos".to_string(), Rc::new(acos_fn));
    funcs.insert("atan".to_string(), Rc::new(atan_fn));
    funcs.insert("sqrt".to_string(), Rc::new(sqrt_fn));
    funcs.insert("torad".to_string(), Rc::new(torad_fn));
    funcs.insert("todeg".to_string(), Rc::new(todeg_fn));
    funcs.insert("exp".to_string(), Rc::new(exp_fn));
    funcs.insert("log".to_string(), Rc::new(log_fn));
    funcs.insert("log2".to_string(), Rc::new(log2_fn));
    funcs.insert("log10".to_string(), Rc::new(log10_fn));
    funcs.insert("ceil".to_string(), Rc::new(ceil_fn));
    funcs.insert("floor".to_string(), Rc::new(floor_fn));
    funcs.insert("round".to_string(), Rc::new(round_fn));
    funcs.insert("abs".to_string(), Rc::new(abs_fn));
    funcs.insert("min".to_string(), Rc::new(min_fn));
    funcs.insert("max".to_string(), Rc::new(max_fn));
    funcs.insert("clamp".to_string(), Rc::new(clamp_fn));

    BuiltinModule { vars, funcs }
}

fn get_number_arg(
    _: &mut Interpreter,
    args: &[Value],
    span: &Span,
    name: &str,
) -> InterpResult<f64> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("{}() expects 1 argument, got {}", name, args.len()),
        });
    }
    let val = &args[0];
    match val {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(f) => Ok(*f),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "{}() expects a number, got {}",
                name,
                crate::utils::type_name(&val)
            ),
        }),
    }
}

// ---------- Math ----------
fn sin_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "sin")?;
    Ok(Value::Float(num.sin()))
}

fn cos_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "cos")?;
    Ok(Value::Float(num.cos()))
}

fn tan_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "tan")?;
    Ok(Value::Float(num.tan()))
}

fn asin_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "asin")?;
    if num < -1.0 || num > 1.0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: "asin() argument must be between -1 and 1".to_string(),
        });
    }
    Ok(Value::Float(num.asin()))
}

fn acos_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "acos")?;
    if num < -1.0 || num > 1.0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: "acos() argument must be between -1 and 1".to_string(),
        });
    }
    Ok(Value::Float(num.acos()))
}

fn atan_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "atan")?;
    Ok(Value::Float(num.atan()))
}

fn sqrt_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "sqrt")?;
    if num < 0.0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: "sqrt() argument must be non-negative".to_string(),
        });
    }
    Ok(Value::Float(num.sqrt()))
}

fn torad_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "torad")?;
    Ok(Value::Float(num * PI / 180.0))
}

fn todeg_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "todeg")?;
    Ok(Value::Float(num * 180.0 / PI))
}

fn exp_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "exp")?;
    Ok(Value::Float(num.exp()))
}

fn log_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "log")?;
    if num <= 0.0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: "log() argument must be positive".to_string(),
        });
    }
    Ok(Value::Float(num.ln()))
}

fn log2_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "log2")?;
    if num <= 0.0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: "log2() argument must be positive".to_string(),
        });
    }
    Ok(Value::Float(num.log2()))
}

fn log10_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "log10")?;
    if num <= 0.0 {
        return Err(InterpError::Runtime {
            span: *span,
            message: "log10() argument must be positive".to_string(),
        });
    }
    Ok(Value::Float(num.log10()))
}

fn ceil_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "ceil")?;
    Ok(Value::Float(num.ceil()))
}

fn floor_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "floor")?;
    Ok(Value::Float(num.floor()))
}

fn round_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "round")?;
    Ok(Value::Float(num.round()))
}

fn abs_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "abs")?;
    Ok(Value::Float(num.abs()))
}

fn min_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() == 2 {
        // Two numbers
        let a = get_number_arg_opt(&args[0], span)?;
        let b = get_number_arg_opt(&args[1], span)?;
        return Ok(min_of_two(a, b));
    } else if args.len() == 1 {
        // One argument: must be a list
        match &args[0] {
            Value::List(list_rc) => {
                let list = list_rc.borrow();
                if list.is_empty() {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "min() on empty list".to_string(),
                    });
                }
                // Convert all elements to f64, track if any is float
                let mut all_int = true;
                let mut values = Vec::with_capacity(list.len());
                for v in list.iter() {
                    match v {
                        Value::Int(i) => values.push((*i as f64, true)),
                        Value::Float(f) => {
                            values.push((*f, false));
                            all_int = false;
                        }
                        _ => {
                            return Err(InterpError::Runtime {
                                span: *span,
                                message: format!(
                                    "min() list must contain numbers, got {}",
                                    crate::utils::type_name(v)
                                ),
                            });
                        }
                    }
                }
                let min_val = values.iter().map(|(f, _)| *f).fold(f64::INFINITY, f64::min);
                if all_int {
                    Ok(Value::Int(min_val as i64))
                } else {
                    Ok(Value::Float(min_val))
                }
            }
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "min() expects either two numbers or a list of numbers, got {}",
                        crate::utils::type_name(&args[0])
                    ),
                });
            }
        }
    } else {
        Err(InterpError::Runtime {
            span: *span,
            message: format!("min() expects 1 or 2 arguments, got {}", args.len()),
        })
    }
}

fn max_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() == 2 {
        let a = get_number_arg_opt(&args[0], span)?;
        let b = get_number_arg_opt(&args[1], span)?;
        Ok(max_of_two(a, b))
    } else if args.len() == 1 {
        match &args[0] {
            Value::List(list_rc) => {
                let list = list_rc.borrow();
                if list.is_empty() {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "max() on empty list".to_string(),
                    });
                }
                let mut all_int = true;
                let mut values = Vec::with_capacity(list.len());
                for v in list.iter() {
                    match v {
                        Value::Int(i) => {
                            values.push((*i as f64, true));
                        }
                        Value::Float(f) => {
                            values.push((*f, false));
                            all_int = false;
                        }
                        _ => {
                            return Err(InterpError::Runtime {
                                span: *span,
                                message: format!(
                                    "max() list must contain numbers, got {}",
                                    crate::utils::type_name(v)
                                ),
                            });
                        }
                    }
                }
                let max_val = values
                    .iter()
                    .map(|(f, _)| *f)
                    .fold(f64::NEG_INFINITY, f64::max);
                if all_int {
                    Ok(Value::Int(max_val as i64))
                } else {
                    Ok(Value::Float(max_val))
                }
            }
            _ => {
                return Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "max() expects either two numbers or a list of numbers, got {}",
                        crate::utils::type_name(&args[0])
                    ),
                });
            }
        }
    } else {
        Err(InterpError::Runtime {
            span: *span,
            message: format!("max() expects 1 or 2 arguments, got {}", args.len()),
        })
    }
}

fn clamp_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "clamp() expects 3 arguments (value, min, max), got {}",
                args.len()
            ),
        });
    }
    let val = get_number_arg_opt(&args[0], span)?;
    let a = get_number_arg_opt(&args[1], span)?;
    let b = get_number_arg_opt(&args[2], span)?;

    // Ensure low <= high
    let (low, high) = if a.0 <= b.0 { (a, b) } else { (b, a) };

    let result = if val.0 < low.0 {
        low
    } else if val.0 > high.0 {
        high
    } else {
        val
    };

    if result.1 {
        Ok(Value::Int(result.0 as i64))
    } else {
        Ok(Value::Float(result.0))
    }
}

// Helper to extract a number as f64, regardless of int or float.
fn get_number_arg_opt(val: &Value, span: &Span) -> InterpResult<(f64, bool)> {
    match val {
        Value::Int(i) => Ok((*i as f64, true)),
        Value::Float(f) => Ok((*f, false)),
        _ => Err(InterpError::Runtime {
            span: *span,
            message: format!("Expected number, got {}", crate::utils::type_name(val)),
        }),
    }
}

fn min_of_two(a: (f64, bool), b: (f64, bool)) -> Value {
    if a.1 && b.1 {
        // both ints
        let ai = a.0 as i64;
        let bi = b.0 as i64;
        Value::Int(ai.min(bi))
    } else {
        let af = if a.1 { a.0 } else { a.0 };
        let bf = if b.1 { b.0 } else { b.0 };
        Value::Float(af.min(bf))
    }
}

fn max_of_two(a: (f64, bool), b: (f64, bool)) -> Value {
    if a.1 && b.1 {
        let ai = a.0 as i64;
        let bi = b.0 as i64;
        Value::Int(ai.max(bi))
    } else {
        let af = if a.1 { a.0 } else { a.0 };
        let bf = if b.1 { b.0 } else { b.0 };
        Value::Float(af.max(bf))
    }
}
