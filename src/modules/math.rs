use crate::ast::Span;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::modules::{BuiltinFn, BuiltinModule};
use crate::value::Value;
use rand::RngExt;
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
    funcs.insert("rand".to_string(), Rc::new(rand_fn));

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

fn rand_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "rand() expects 2 arguments (start, end), got {}",
                args.len()
            ),
        });
    }
    let start_val = &args[0];
    let end_val = &args[1];
    let start = match start_val {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "rand() start must be integer".to_string(),
            });
        }
    };
    let end = match end_val {
        Value::Int(i) => *i,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: "rand() end must be integer".to_string(),
            });
        }
    };
    if start > end {
        return Err(InterpError::Runtime {
            span: *span,
            message: "rand() start must be <= end".to_string(),
        });
    }
    let mut rng = rand::rng();
    let value = rng.random_range(start..=end);
    Ok(Value::Int(value))
}
