use crate::ast::Span;
use crate::builtins::ModuleFunction;
use crate::error::{InterpError, InterpResult};
use crate::interpreter::Interpreter;
use crate::value::Value;
use chrono::{Datelike, Duration, Local, NaiveDateTime, Timelike, Utc};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ---------- Helper functions for conversion ----------

/// Converts a NaiveDateTime to a Hi dictionary.
/// Dictionary fields: year, month, day, hour, minute, second, millisecond, timestamp (Unix ms).
fn datetime_to_dict(dt: NaiveDateTime) -> Value {
    let mut map = HashMap::new();
    map.insert(
        Value::String("year".to_string()),
        Value::Int(dt.year() as i64),
    );
    map.insert(
        Value::String("month".to_string()),
        Value::Int(dt.month() as i64),
    );
    map.insert(
        Value::String("day".to_string()),
        Value::Int(dt.day() as i64),
    );
    map.insert(
        Value::String("hour".to_string()),
        Value::Int(dt.hour() as i64),
    );
    map.insert(
        Value::String("minute".to_string()),
        Value::Int(dt.minute() as i64),
    );
    map.insert(
        Value::String("second".to_string()),
        Value::Int(dt.second() as i64),
    );
    map.insert(
        Value::String("millisecond".to_string()),
        Value::Int(dt.timestamp_subsec_millis() as i64),
    );
    let timestamp_ms = dt.timestamp_millis();
    map.insert(
        Value::String("timestamp".to_string()),
        Value::Int(timestamp_ms),
    );
    Value::Dict(Rc::new(RefCell::new(map)))
}

/// Extracts a NaiveDateTime from a Hi dictionary.
fn dict_to_datetime(dict_val: &Value, span: &Span) -> InterpResult<NaiveDateTime> {
    let map = match dict_val {
        Value::Dict(m) => m,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "Expected a dictionary, got {}",
                    crate::utils::type_name(dict_val)
                ),
            });
        }
    };
    let map_ref = map.borrow();

    let get_int = |key: &str| -> InterpResult<i64> {
        if let Some(Value::Int(v)) = map_ref.get(&Value::String(key.to_string())) {
            Ok(*v)
        } else {
            Err(InterpError::Runtime {
                span: *span,
                message: format!("Missing or invalid field '{}' in datetime dictionary", key),
            })
        }
    };

    let year = get_int("year")?;
    let month = get_int("month")?;
    let day = get_int("day")?;
    let hour = get_int("hour")?;
    let minute = get_int("minute")?;
    let second = get_int("second")?;
    let millisecond = get_int("millisecond")?;

    // Validate ranges (chrono will panic if out of range, so we check)
    if month < 1 || month > 12 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("Month must be 1..12, got {}", month),
        });
    }
    if day < 1 || day > 31 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("Day must be 1..31, got {}", day),
        });
    }
    if hour < 0 || hour > 23 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("Hour must be 0..23, got {}", hour),
        });
    }
    if minute < 0 || minute > 59 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("Minute must be 0..59, got {}", minute),
        });
    }
    if second < 0 || second > 59 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("Second must be 0..59, got {}", second),
        });
    }
    if millisecond < 0 || millisecond > 999 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("Millisecond must be 0..999, got {}", millisecond),
        });
    }

    // Build NaiveDateTime
    let dt = NaiveDateTime::from_timestamp_opt(
        0, // we'll set via with_* methods
        0,
    )
    .unwrap()
    .with_year(year as i32)
    .and_then(|d| d.with_month(month as u32))
    .and_then(|d| d.with_day(day as u32))
    .and_then(|d| d.with_hour(hour as u32))
    .and_then(|d| d.with_minute(minute as u32))
    .and_then(|d| d.with_second(second as u32))
    .and_then(|d| d.with_nanosecond((millisecond as u32) * 1_000_000))
    .ok_or_else(|| InterpError::Runtime {
        span: *span,
        message: "Invalid date/time values".to_string(),
    })?;

    Ok(dt)
}

/// Converts a chrono::Duration to a Hi dictionary.
/// Fields: days, hours, minutes, seconds, milliseconds (all integers, can be negative).
fn duration_to_dict(dur: Duration) -> Value {
    let total_ms = dur.num_milliseconds();
    let sign = if total_ms < 0 { -1 } else { 1 };
    let abs_ms = total_ms.abs();

    let days = abs_ms / 86_400_000;
    let rem = abs_ms % 86_400_000;
    let hours = rem / 3_600_000;
    let rem = rem % 3_600_000;
    let minutes = rem / 60_000;
    let rem = rem % 60_000;
    let seconds = rem / 1_000;
    let milliseconds = rem % 1_000;

    // Apply sign to all components (negative duration)
    let (d, h, m, s, ms) = if sign < 0 {
        (-days, -hours, -minutes, -seconds, -milliseconds)
    } else {
        (days, hours, minutes, seconds, milliseconds)
    };

    let mut map = HashMap::new();
    map.insert(Value::String("days".to_string()), Value::Int(d));
    map.insert(Value::String("hours".to_string()), Value::Int(h));
    map.insert(Value::String("minutes".to_string()), Value::Int(m));
    map.insert(Value::String("seconds".to_string()), Value::Int(s));
    map.insert(Value::String("milliseconds".to_string()), Value::Int(ms));
    Value::Dict(Rc::new(RefCell::new(map)))
}

/// Extracts a chrono::Duration from a Hi dictionary (as produced by duration_to_dict).
fn dict_to_duration(dict_val: &Value, span: &Span) -> InterpResult<Duration> {
    let map = match dict_val {
        Value::Dict(m) => m,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "Expected a duration dictionary, got {}",
                    crate::utils::type_name(dict_val)
                ),
            });
        }
    };
    let map_ref = map.borrow();

    let get_int = |key: &str| -> InterpResult<i64> {
        if let Some(Value::Int(v)) = map_ref.get(&Value::String(key.to_string())) {
            Ok(*v)
        } else {
            Err(InterpError::Runtime {
                span: *span,
                message: format!("Missing or invalid field '{}' in duration dictionary", key),
            })
        }
    };

    let days = get_int("days")?;
    let hours = get_int("hours")?;
    let minutes = get_int("minutes")?;
    let seconds = get_int("seconds")?;
    let milliseconds = get_int("milliseconds")?;

    // Total milliseconds (may be negative)
    let total_ms =
        days * 86_400_000 + hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + milliseconds;
    Ok(Duration::milliseconds(total_ms))
}

// ---------- Public functions ----------

/// now() – returns a dictionary with the current local date and time.
fn now_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if !args.is_empty() {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("now() expects no arguments, got {}", args.len()),
        });
    }
    let dt = Local::now().naive_local();
    Ok(datetime_to_dict(dt))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "now",
        params: &[],
        doc: "Returns a dictionary with the current local date and time (fields: year, month, day, hour, minute, second, millisecond, timestamp).",
        func: now_fn,
    }
}

/// utcnow() – returns a dictionary with the current UTC date and time.
fn utcnow_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if !args.is_empty() {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("utcnow() expects no arguments, got {}", args.len()),
        });
    }
    let dt = Utc::now().naive_utc();
    Ok(datetime_to_dict(dt))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "utcnow",
        params: &[],
        doc: "Returns a dictionary with the current UTC date and time (same fields as now()).",
        func: utcnow_fn,
    }
}

/// fromstring(str, format) – parses a string according to the format and returns a datetime dictionary.
fn fromstring_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "fromstring() expects 2 arguments (string, format), got {}",
                args.len()
            ),
        });
    }
    let s = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "fromstring() first argument must be a string, got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let fmt = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "fromstring() second argument must be a string, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };

    let dt = NaiveDateTime::parse_from_str(s, fmt).map_err(|e| InterpError::Runtime {
        span: *span,
        message: format!("Failed to parse datetime: {}", e),
    })?;
    Ok(datetime_to_dict(dt))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "fromstring",
        params: &["string", "format"],
        doc: "Parses a string using the given format (see chrono format specifiers) and returns a datetime dictionary.",
        func: fromstring_fn,
    }
}

/// tostring(dt, format) – formats a datetime dictionary into a string.
fn tostring_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "tostring() expects 2 arguments (datetime, format), got {}",
                args.len()
            ),
        });
    }
    let dt_val = &args[0];
    let fmt = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "tostring() second argument must be a string, got {}",
                    crate::utils::type_name(&args[1])
                ),
            });
        }
    };

    let dt = dict_to_datetime(dt_val, span)?;
    let s = dt.format(fmt).to_string();
    Ok(Value::String(s))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "tostring",
        params: &["datetime", "format"],
        doc: "Formats a datetime dictionary as a string according to the given format.",
        func: tostring_fn,
    }
}

/// add(dt, duration) – adds a duration to a datetime and returns a new datetime dictionary.
fn add_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "add() expects 2 arguments (datetime, duration), got {}",
                args.len()
            ),
        });
    }
    let dt_val = &args[0];
    let dur_val = &args[1];

    let dt = dict_to_datetime(dt_val, span)?;
    let dur = dict_to_duration(dur_val, span)?;
    let new_dt = dt + dur;
    Ok(datetime_to_dict(new_dt))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "add",
        params: &["datetime", "duration"],
        doc: "Adds a duration dictionary to a datetime dictionary and returns a new datetime dictionary.",
        func: add_fn,
    }
}

/// diff(dt1, dt2) – returns the difference (dt1 - dt2) as a duration dictionary.
fn diff_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("diff() expects 2 arguments (dt1, dt2), got {}", args.len()),
        });
    }
    let dt1 = dict_to_datetime(&args[0], span)?;
    let dt2 = dict_to_datetime(&args[1], span)?;
    let dur = dt1 - dt2;
    Ok(duration_to_dict(dur))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "diff",
        params: &["dt1", "dt2"],
        doc: "Returns the difference (dt1 - dt2) as a duration dictionary (fields: days, hours, minutes, seconds, milliseconds).",
        func: diff_fn,
    }
}

/// year(dt) – returns the year component of a datetime.
fn year_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("year() expects 1 argument (datetime), got {}", args.len()),
        });
    }
    let dt = dict_to_datetime(&args[0], span)?;
    Ok(Value::Int(dt.year() as i64))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "year",
        params: &["datetime"],
        doc: "Returns the year field from a datetime dictionary as an integer.",
        func: year_fn,
    }
}

/// month(dt) – returns the month (1-12).
fn month_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("month() expects 1 argument (datetime), got {}", args.len()),
        });
    }
    let dt = dict_to_datetime(&args[0], span)?;
    Ok(Value::Int(dt.month() as i64))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "month",
        params: &["datetime"],
        doc: "Returns the month field (1‑12) from a datetime dictionary.",
        func: month_fn,
    }
}

/// day(dt) – returns the day of month (1-31).
fn day_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("day() expects 1 argument (datetime), got {}", args.len()),
        });
    }
    let dt = dict_to_datetime(&args[0], span)?;
    Ok(Value::Int(dt.day() as i64))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "day",
        params: &["datetime"],
        doc: "Returns the day of month (1‑31) from a datetime dictionary.",
        func: day_fn,
    }
}

/// hour(dt) – returns the hour (0-23).
fn hour_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("hour() expects 1 argument (datetime), got {}", args.len()),
        });
    }
    let dt = dict_to_datetime(&args[0], span)?;
    Ok(Value::Int(dt.hour() as i64))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "hour",
        params: &["datetime"],
        doc: "Returns the hour (0‑23) from a datetime dictionary.",
        func: hour_fn,
    }
}

/// minute(dt) – returns the minute (0-59).
fn minute_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("minute() expects 1 argument (datetime), got {}", args.len()),
        });
    }
    let dt = dict_to_datetime(&args[0], span)?;
    Ok(Value::Int(dt.minute() as i64))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "minute",
        params: &["datetime"],
        doc: "Returns the minute (0‑59) from a datetime dictionary.",
        func: minute_fn,
    }
}

/// second(dt) – returns the second (0-59).
fn second_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!("second() expects 1 argument (datetime), got {}", args.len()),
        });
    }
    let dt = dict_to_datetime(&args[0], span)?;
    Ok(Value::Int(dt.second() as i64))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "second",
        params: &["datetime"],
        doc: "Returns the second (0‑59) from a datetime dictionary.",
        func: second_fn,
    }
}

/// millisecond(dt) – returns the millisecond (0-999).
fn millisecond_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "millisecond() expects 1 argument (datetime), got {}",
                args.len()
            ),
        });
    }
    let dt = dict_to_datetime(&args[0], span)?;
    Ok(Value::Int(dt.timestamp_subsec_millis() as i64))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "millisecond",
        params: &["datetime"],
        doc: "Returns the millisecond (0‑999) from a datetime dictionary.",
        func: millisecond_fn,
    }
}

/// timestamp(dt) – returns the Unix timestamp in milliseconds.
fn timestamp_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "timestamp() expects 1 argument (datetime), got {}",
                args.len()
            ),
        });
    }
    let dt = dict_to_datetime(&args[0], span)?;
    Ok(Value::Int(dt.timestamp_millis()))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "timestamp",
        params: &["datetime"],
        doc: "Returns the Unix timestamp in milliseconds from a datetime dictionary.",
        func: timestamp_fn,
    }
}

/// duration(seconds) – creates a duration dictionary from a number of seconds (can be float or integer).
fn duration_fn(_: &mut Interpreter, args: &[Value], span: &Span) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(InterpError::Runtime {
            span: *span,
            message: format!(
                "duration() expects 1 argument (seconds), got {}",
                args.len()
            ),
        });
    }
    let seconds = match &args[0] {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(InterpError::Runtime {
                span: *span,
                message: format!(
                    "duration() argument must be a number (int or float), got {}",
                    crate::utils::type_name(&args[0])
                ),
            });
        }
    };
    let ms = (seconds * 1000.0).round() as i64;
    let dur = Duration::milliseconds(ms);
    Ok(duration_to_dict(dur))
}
inventory::submit! {
    ModuleFunction {
        module: "datetime",
        name: "duration",
        params: &["seconds"],
        doc: "Creates a duration dictionary from a number of seconds (int or float). The duration dictionary has fields: days, hours, minutes, seconds, milliseconds.",
        func: duration_fn,
    }
}
