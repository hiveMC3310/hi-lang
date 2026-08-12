use crate::ast::Span;
use crate::builtins::{ModuleFunction, ModuleVariable};
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use std::f64::consts::PI;

// ---------- Helper functions ----------
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
                crate::utils::type_name(val)
            ),
        }),
    }
}

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
        let ai = a.0 as i64;
        let bi = b.0 as i64;
        Value::Int(ai.min(bi))
    } else {
        Value::Float(a.0.min(b.0))
    }
}

fn max_of_two(a: (f64, bool), b: (f64, bool)) -> Value {
    if a.1 && b.1 {
        let ai = a.0 as i64;
        let bi = b.0 as i64;
        Value::Int(ai.max(bi))
    } else {
        Value::Float(a.0.max(b.0))
    }
}

// ---------- Consts ----------
inventory::submit! {
    ModuleVariable {
        module: "math",
        name: "PI",
    }
}
// PI is a constant: π ≈ 3.141592653589793

inventory::submit! {
    ModuleVariable {
        module: "math",
        name: "E",
    }
}
// E is a constant: e ≈ 2.718281828459045

// ---------- Trigonometric functions ----------
// sin
fn sin_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "sin")?;
    Ok(Value::Float(num.sin()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "sin",
        params: &["x"],
        doc: "Returns the sine of x (in radians) as Float.",
        func: sin_fn,
    }
}

// cos
fn cos_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "cos")?;
    Ok(Value::Float(num.cos()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "cos",
        params: &["x"],
        doc: "Returns the cosine of x (in radians) as Float.",
        func: cos_fn,
    }
}

// tan
fn tan_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "tan")?;
    Ok(Value::Float(num.tan()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "tan",
        params: &["x"],
        doc: "Returns the tangent of x (in radians) as Float.",
        func: tan_fn,
    }
}

// asin
fn asin_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "asin")?;
    if !(-1.0..=1.0).contains(&num) {
        return Err(InterpError::Runtime {
            span: *span,
            message: "asin() argument must be between -1 and 1".to_string(),
        });
    }
    Ok(Value::Float(num.asin()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "asin",
        params: &["x"],
        doc: "Returns the arc sine of x (in radians) as Float. x must be in [-1, 1].",
        func: asin_fn,
    }
}

// acos
fn acos_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "acos")?;
    if !(-1.0..=1.0).contains(&num) {
        return Err(InterpError::Runtime {
            span: *span,
            message: "acos() argument must be between -1 and 1".to_string(),
        });
    }
    Ok(Value::Float(num.acos()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "acos",
        params: &["x"],
        doc: "Returns the arc cosine of x (in radians) as Float. x must be in [-1, 1].",
        func: acos_fn,
    }
}

// atan
fn atan_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "atan")?;
    Ok(Value::Float(num.atan()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "atan",
        params: &["x"],
        doc: "Returns the arc tangent of x (in radians) as Float.",
        func: atan_fn,
    }
}

// ---------- Other ----------
// sqrt
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
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "sqrt",
        params: &["x"],
        doc: "Returns the square root of x (non‑negative) as Float.",
        func: sqrt_fn,
    }
}

// torad
fn torad_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "torad")?;
    Ok(Value::Float(num * PI / 180.0))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "torad",
        params: &["degrees"],
        doc: "Converts degrees to radians and returns Float.",
        func: torad_fn,
    }
}

// todeg
fn todeg_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "todeg")?;
    Ok(Value::Float(num * 180.0 / PI))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "todeg",
        params: &["radians"],
        doc: "Converts radians to degrees and returns Float.",
        func: todeg_fn,
    }
}

// exp
fn exp_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "exp")?;
    Ok(Value::Float(num.exp()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "exp",
        params: &["x"],
        doc: "Returns e^x (exponential) as Float.",
        func: exp_fn,
    }
}

// log (natural log)
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
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "log",
        params: &["x"],
        doc: "Returns the natural logarithm of x (x > 0) as Float.",
        func: log_fn,
    }
}

// log2
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
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "log2",
        params: &["x"],
        doc: "Returns the base-2 logarithm of x (x > 0) as Float.",
        func: log2_fn,
    }
}

// log10
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
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "log10",
        params: &["x"],
        doc: "Returns the base-10 logarithm of x (x > 0) as Float.",
        func: log10_fn,
    }
}

// ceil
fn ceil_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "ceil")?;
    Ok(Value::Float(num.ceil()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "ceil",
        params: &["x"],
        doc: "Returns the smallest integer ≥ x, as Float.",
        func: ceil_fn,
    }
}

// floor
fn floor_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "floor")?;
    Ok(Value::Float(num.floor()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "floor",
        params: &["x"],
        doc: "Returns the largest integer ≤ x, as Float.",
        func: floor_fn,
    }
}

// round
fn round_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "round")?;
    Ok(Value::Float(num.round()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "round",
        params: &["x"],
        doc: "Returns the nearest integer to x, rounding half away from zero, as Float.",
        func: round_fn,
    }
}

// abs
fn abs_fn(interp: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    let num = get_number_arg(interp, args, span, "abs")?;
    Ok(Value::Float(num.abs()))
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "abs",
        params: &["x"],
        doc: "Returns the absolute value of x as Float.",
        func: abs_fn,
    }
}

// ---------- min, max, clamp (returns Int or Float) ----------
// min
fn min_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() == 2 {
        let a = get_number_arg_opt(&args[0], span)?;
        let b = get_number_arg_opt(&args[1], span)?;
        Ok(min_of_two(a, b))
    } else if args.len() == 1 {
        match &args[0] {
            Value::List(list_rc) => {
                let list = list_rc.borrow();
                if list.is_empty() {
                    return Err(InterpError::Runtime {
                        span: *span,
                        message: "min() on empty list".to_string(),
                    });
                }
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
                Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "min() expects either two numbers or a list of numbers, got {}",
                        crate::utils::type_name(&args[0])
                    ),
                })
            }
        }
    } else {
        Err(InterpError::Runtime {
            span: *span,
            message: format!("min() expects 1 or 2 arguments, got {}", args.len()),
        })
    }
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "min",
        params: &["..."],
        doc: "Returns the minimum of two numbers or of a list of numbers. Returns Int if all inputs are integers, else Float.",
        func: min_fn,
    }
}

// max
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
                Err(InterpError::Runtime {
                    span: *span,
                    message: format!(
                        "max() expects either two numbers or a list of numbers, got {}",
                        crate::utils::type_name(&args[0])
                    ),
                })
            }
        }
    } else {
        Err(InterpError::Runtime {
            span: *span,
            message: format!("max() expects 1 or 2 arguments, got {}", args.len()),
        })
    }
}
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "max",
        params: &["..."],
        doc: "Returns the maximum of two numbers or of a list of numbers. Returns Int if all inputs are integers, else Float.",
        func: max_fn,
    }
}

// clamp
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
inventory::submit! {
    ModuleFunction {
        module: "math",
        name: "clamp",
        params: &["value", "min", "max"],
        doc: "Clamps a number between min and max (inclusive). Returns Int if all arguments are integers, else Float.",
        func: clamp_fn,
    }
}
