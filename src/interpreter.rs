//! Core interpreter: runs Hi programs.

use crate::commands::{BinOp, Command};
use crate::error::{InterpError, InterpResult};
use crate::tokenizer::Tokenizer;
use crate::value::Value;
use std::collections::HashMap;
use std::io::Write;

/// A call frame: holds return address, local variables, and active loops for BREAK.
#[derive(Clone)]
pub struct CallFrame {
    return_line: usize,
    locals: HashMap<String, Value>,
    active_loops: Vec<usize>,
}

impl CallFrame {
    pub fn new(return_line: usize) -> Self {
        Self {
            return_line,
            locals: HashMap::new(),
            active_loops: Vec::new(),
        }
    }
}

/// The interpreter state: lines, stack, variables, jump maps, and call stack.
pub struct Interpreter {
    line_num: usize,
    pub lines: Vec<String>,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub call_stack: Vec<CallFrame>,

    if_jump_map: HashMap<usize, usize>,
    else_jump_map: HashMap<usize, usize>,

    loop_start_map: HashMap<usize, usize>,
    loop_back_map: HashMap<usize, usize>,

    func_start_map: HashMap<String, usize>,
    func_end_map: HashMap<String, usize>,
}

impl Interpreter {
    /// Creates a new interpreter from source lines.
    pub fn new(lines: Vec<String>) -> Self {
        let mut s = Self {
            lines,
            line_num: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            if_jump_map: HashMap::new(),
            else_jump_map: HashMap::new(),
            loop_start_map: HashMap::new(),
            loop_back_map: HashMap::new(),
            func_start_map: HashMap::new(),
            func_end_map: HashMap::new(),
        };
        s.call_stack.push(CallFrame::new(0));
        s
    }

    /// Runs the program starting from a specific line number.
    /// Calls `build_maps()` first to ensure jump maps are up‑to‑date.
    pub fn run_from(&mut self, start_line: usize) -> InterpResult<()> {
        self.build_maps()?;
        self.line_num = start_line;

        while self.line_num < self.lines.len() {
            let raw_line = &self.lines[self.line_num];
            let tokens = Tokenizer::tokenize(raw_line, self.line_num + 1)?;

            if tokens.is_empty() {
                self.line_num += 1;
                continue;
            }

            let cmd = self.parse_command(&tokens)?;
            self.execute_command(&cmd)?;
        }
        Ok(())
    }

    /// Runs the program. Returns an error if any occurs.
    pub fn run(&mut self) -> InterpResult<()> {
        self.run_from(0)
    }

    /// Builds jump maps for IF/WHILE/FUNC structures.
    pub fn build_maps(&mut self) -> InterpResult<()> {
        let mut if_stack: Vec<(usize, Option<usize>)> = Vec::new();
        let mut while_stack = Vec::new();
        let mut func_stack = Vec::new();

        for (i, line) in self.lines.iter().enumerate() {
            let tokens = Tokenizer::tokenize(line, i + 1)?;
            if tokens.is_empty() {
                continue;
            }
            let cmd = tokens[0].to_uppercase();

            match cmd.as_str() {
                "FUNC" => {
                    if tokens.len() < 2 {
                        return Err(InterpError::Syntax {
                            line: i + 1,
                            message: "FUNC requires a name".to_string(),
                        });
                    }
                    let name = tokens[1].clone();
                    self.func_start_map.remove(&name);
                    self.func_end_map.remove(&name);
                    func_stack.push((name, i));
                }
                "ENDF" => {
                    if let Some((name, start)) = func_stack.pop() {
                        self.func_start_map.insert(name.clone(), start);
                        self.func_end_map.insert(name.clone(), i);
                    } else {
                        return Err(InterpError::Syntax {
                            line: i + 1,
                            message: "Unexpected ENDF".to_string(),
                        });
                    }
                }
                "IF" => {
                    if tokens.len() < 2 {
                        return Err(InterpError::Syntax {
                            line: i + 1,
                            message: "IF requires a condition".to_string(),
                        });
                    }
                    if_stack.push((i, None));
                }
                "ELSE" => {
                    if let Some((_, else_line)) = if_stack.last_mut() {
                        *else_line = Some(i);
                    } else {
                        return Err(InterpError::Syntax {
                            line: i + 1,
                            message: "Unexpected ELSE".to_string(),
                        });
                    }
                }
                "ENDIF" => {
                    if let Some((if_line, else_line_opt)) = if_stack.pop() {
                        let target = match &else_line_opt {
                            Some(else_line) => *else_line,
                            None => i,
                        };
                        self.if_jump_map.insert(if_line, target);

                        if let Some(else_line) = else_line_opt {
                            self.else_jump_map.insert(else_line, i);
                        }
                    } else {
                        return Err(InterpError::Syntax {
                            line: i + 1,
                            message: "Unexpected ENDIF".to_string(),
                        });
                    }
                }
                "WHILE" => while_stack.push(i),
                "DO" => {
                    if let Some(start) = while_stack.pop() {
                        self.loop_start_map.insert(start, i);
                        self.loop_back_map.insert(i, start);
                    } else {
                        return Err(InterpError::Syntax {
                            line: i + 1,
                            message: "Unexpected DO".to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        if !if_stack.is_empty() {
            return Err(InterpError::UnclosedBlock {
                block: "IF".to_string(),
            });
        }
        if !while_stack.is_empty() {
            return Err(InterpError::UnclosedBlock {
                block: "WHILE".to_string(),
            });
        }
        if !func_stack.is_empty() {
            return Err(InterpError::UnclosedBlock {
                block: "FUNC".to_string(),
            });
        }

        Ok(())
    }

    /// Returns `true` if the current scope is the global scope (i.e., not inside any function).
    fn is_global_scope(&self) -> bool {
        self.call_stack.len() == 1 && self.call_stack[0].return_line == 0
    }

    /// Returns a reference to the current local variables, if any.
    fn current_locals(&self) -> Option<&HashMap<String, Value>> {
        self.call_stack.last().map(|frame| &frame.locals)
    }

    /// Returns a mutable reference to the current local variables, if any.
    fn current_locals_mut(&mut self) -> Option<&mut HashMap<String, Value>> {
        self.call_stack.last_mut().map(|frame| &mut frame.locals)
    }

    /// Parses a token list into a Command.
    fn parse_command(&self, tokens: &[String]) -> InterpResult<Command> {
        if tokens.is_empty() {
            return Err(InterpError::Internal("Empty token list".to_string()));
        }
        let cmd_str = tokens[0].to_uppercase();

        match cmd_str.as_str() {
            "HELLO" => Ok(Command::Hello),
            "PUSH" => {
                if tokens.len() < 2 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "PUSH requires a value".to_string(),
                    });
                }
                let value = Tokenizer::resolve_value(
                    &tokens[1],
                    &self.stack,
                    &self.globals,
                    self.current_locals(),
                )?;
                Ok(Command::Push(value))
            }
            "POP" => {
                let var = if tokens.len() >= 2 {
                    Some(tokens[1].clone())
                } else {
                    None
                };
                Ok(Command::Pop(var))
            }
            "LET" => {
                if tokens.len() < 3 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "LET requires name and value".to_string(),
                    });
                }
                let name = tokens[1].clone();
                let value = Tokenizer::resolve_value(
                    &tokens[2],
                    &self.stack,
                    &self.globals,
                    self.current_locals(),
                )?;
                Ok(Command::Let(name, value))
            }
            "PRINT" => {
                if tokens.len() < 2 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "PRINT requires at least one argument".to_string(),
                    });
                }
                let mut args = Vec::new();
                for tok in &tokens[1..] {
                    let v = Tokenizer::resolve_value(
                        tok,
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    args.push(v);
                }
                Ok(Command::Print(args))
            }
            "INPUT" => {
                let (prompt, var) = match tokens.len() {
                    2 => (None, tokens[1].clone()),
                    3 => (Some(tokens[1].clone()), tokens[2].clone()),
                    _ => {
                        return Err(InterpError::Syntax {
                            line: self.line_num + 1,
                            message: "INPUT takes 1 or 2 arguments".to_string(),
                        });
                    }
                };
                Ok(Command::Input(prompt, var))
            }
            "ADD" | "SUB" | "MUL" | "DIV" | "EQ" | "NE" | "GT" | "GE" | "LT" | "LE" | "AND"
            | "OR" | "MOD" | "POW" => {
                let op = match cmd_str.as_str() {
                    "ADD" => BinOp::Add,
                    "SUB" => BinOp::Sub,
                    "MUL" => BinOp::Mul,
                    "DIV" => BinOp::Div,
                    "EQ" => BinOp::Eq,
                    "NE" => BinOp::Ne,
                    "GT" => BinOp::Gt,
                    "GE" => BinOp::Ge,
                    "LT" => BinOp::Lt,
                    "LE" => BinOp::Le,
                    "AND" => BinOp::And,
                    "OR" => BinOp::Or,
                    "MOD" => BinOp::Mod,
                    "POW" => BinOp::Pow,
                    _ => {
                        return Err(InterpError::Syntax {
                            line: self.line_num + 1,
                            message: format!("Unknown binary operator '{}'", cmd_str),
                        });
                    }
                };
                let (left, right) = self.parse_binary_args(tokens)?;
                Ok(Command::Binary(op, left, right))
            }
            "NOT" => {
                if tokens.len() == 1 {
                    if self.stack.len() < 1 {
                        return Err(InterpError::Semantic {
                            line: self.line_num + 1,
                            message: "Not enough values on stack for logic operation".to_string(),
                        });
                    }
                    Ok(Command::Not(None))
                } else if tokens.len() >= 2 {
                    let a = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    Ok(Command::Not(Some(a)))
                } else {
                    Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "Invalid arguments for logic operation".to_string(),
                    })
                }
            }
            "IF" => {
                if tokens.len() == 4 {
                    let op_str = tokens[1].to_uppercase();
                    if ["EQ", "NE", "GT", "GE", "LT", "LE", "AND", "OR"].contains(&op_str.as_str())
                    {
                        let left = Tokenizer::resolve_value(
                            &tokens[2],
                            &self.stack,
                            &self.globals,
                            self.current_locals(),
                        )?;
                        let right = Tokenizer::resolve_value(
                            &tokens[3],
                            &self.stack,
                            &self.globals,
                            self.current_locals(),
                        )?;
                        let op = match op_str.as_str() {
                            "EQ" => BinOp::Eq,
                            "NE" => BinOp::Ne,
                            "GT" => BinOp::Gt,
                            "GE" => BinOp::Ge,
                            "LT" => BinOp::Lt,
                            "LE" => BinOp::Le,
                            "AND" => BinOp::And,
                            "OR" => BinOp::Or,
                            _ => {
                                return Err(InterpError::Syntax {
                                    line: self.line_num + 1,
                                    message: format!(
                                        "Unknown comparison/logical operator '{}'",
                                        op_str
                                    ),
                                });
                            }
                        };
                        let bool_result =
                            Self::evaluate_binary_op_bool(op, &left, &right, self.line_num + 1)?;
                        return Ok(Command::If(Value::Bool(bool_result)));
                    }
                }
                if tokens.len() == 2 {
                    let cond = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    return Ok(Command::If(cond));
                }
                Err(InterpError::Syntax {
                    line: self.line_num + 1,
                    message: "IF requires a condition or a comparison expression (e.g., IF EQ x 1)"
                        .to_string(),
                })
            }
            "ELSE" => Ok(Command::Else),
            "ENDIF" => Ok(Command::Endif),
            "WHILE" => {
                if tokens.len() == 4 {
                    let op_str = tokens[1].to_uppercase();
                    if ["EQ", "NE", "GT", "GE", "LT", "LE", "AND", "OR"].contains(&op_str.as_str())
                    {
                        let left = Tokenizer::resolve_value(
                            &tokens[2],
                            &self.stack,
                            &self.globals,
                            self.current_locals(),
                        )?;
                        let right = Tokenizer::resolve_value(
                            &tokens[3],
                            &self.stack,
                            &self.globals,
                            self.current_locals(),
                        )?;
                        let op = match op_str.as_str() {
                            "EQ" => BinOp::Eq,
                            "NE" => BinOp::Ne,
                            "GT" => BinOp::Gt,
                            "GE" => BinOp::Ge,
                            "LT" => BinOp::Lt,
                            "LE" => BinOp::Le,
                            "AND" => BinOp::And,
                            "OR" => BinOp::Or,
                            _ => {
                                return Err(InterpError::Syntax {
                                    line: self.line_num + 1,
                                    message: format!(
                                        "Unknown comparison/logical operator '{}'",
                                        op_str
                                    ),
                                });
                            }
                        };
                        let bool_result =
                            Self::evaluate_binary_op_bool(op, &left, &right, self.line_num + 1)?;
                        return Ok(Command::While(Value::Bool(bool_result)));
                    }
                }
                if tokens.len() == 2 {
                    let cond = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    return Ok(Command::While(cond));
                }
                Err(InterpError::Syntax {
                    line: self.line_num + 1,
                    message: "WHILE requires a condition".to_string(),
                })
            }
            "DO" => Ok(Command::Do),
            "BREAK" => Ok(Command::Break),
            "FUNC" => {
                if tokens.len() < 2 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "FUNC requires a name".to_string(),
                    });
                }
                Ok(Command::Func(tokens[1].clone()))
            }
            "RET" => Ok(Command::Ret),
            "ENDF" => Ok(Command::Endf),
            "CALL" => {
                if tokens.len() < 2 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "CALL requires a function name".to_string(),
                    });
                }
                Ok(Command::Call(tokens[1].clone()))
            }
            "LEN" => {
                if tokens.len() == 2 {
                    let val = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    Ok(Command::Len(Some(val)))
                } else if tokens.len() == 1 {
                    if self.stack.is_empty() {
                        return Err(InterpError::Semantic {
                            line: self.line_num + 1,
                            message: "LEN requires a value on stack".to_string(),
                        });
                    }
                    Ok(Command::Len(None))
                } else {
                    Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "LEN takes 0 or 1 argument".to_string(),
                    })
                }
            }
            "CONCAT" => {
                if tokens.len() == 3 {
                    let a = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    let b = Tokenizer::resolve_value(
                        &tokens[2],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    Ok(Command::Concat(Some(a), Some(b)))
                } else if tokens.len() == 1 {
                    if self.stack.len() < 2 {
                        return Err(InterpError::Semantic {
                            line: self.line_num + 1,
                            message: "CONCAT requires two values on stack".to_string(),
                        });
                    }
                    Ok(Command::Concat(None, None))
                } else {
                    Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "CONCAT takes 0 or 2 arguments".to_string(),
                    })
                }
            }
            "SUBSTR" => {
                if tokens.len() == 4 {
                    let s = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    let start = Tokenizer::resolve_value(
                        &tokens[2],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    let len = Tokenizer::resolve_value(
                        &tokens[3],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    Ok(Command::Substr(Some(s), Some(start), Some(len)))
                } else if tokens.len() == 1 {
                    if self.stack.len() < 3 {
                        return Err(InterpError::Semantic {
                            line: self.line_num + 1,
                            message: "SUBSTR requires three values on stack".to_string(),
                        });
                    }
                    Ok(Command::Substr(None, None, None))
                } else {
                    Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "SUBSTR takes 0 or 3 arguments".to_string(),
                    })
                }
            }
            "UPPER" => {
                if tokens.len() == 2 {
                    let val = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    Ok(Command::Upper(Some(val)))
                } else if tokens.len() == 1 {
                    if self.stack.is_empty() {
                        return Err(InterpError::Semantic {
                            line: self.line_num + 1,
                            message: "UPPER requires a value on stack".to_string(),
                        });
                    }
                    Ok(Command::Upper(None))
                } else {
                    Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "UPPER takes 0 or 1 argument".to_string(),
                    })
                }
            }
            "LOWER" => {
                if tokens.len() == 2 {
                    let val = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    Ok(Command::Lower(Some(val)))
                } else if tokens.len() == 1 {
                    if self.stack.is_empty() {
                        return Err(InterpError::Semantic {
                            line: self.line_num + 1,
                            message: "LOWER requires a value on stack".to_string(),
                        });
                    }
                    Ok(Command::Lower(None))
                } else {
                    Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "LOWER takes 0 or 1 argument".to_string(),
                    })
                }
            }
            "TRIM" => {
                if tokens.len() == 2 {
                    let val = Tokenizer::resolve_value(
                        &tokens[1],
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    Ok(Command::Trim(Some(val)))
                } else if tokens.len() == 1 {
                    if self.stack.is_empty() {
                        return Err(InterpError::Semantic {
                            line: self.line_num + 1,
                            message: "TRIM requires a value on stack".to_string(),
                        });
                    }
                    Ok(Command::Trim(None))
                } else {
                    Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "TRIM takes 0 or 1 argument".to_string(),
                    })
                }
            }
            "LIST" => {
                let mut elements = Vec::new();
                for token in &tokens[1..] {
                    let val = Tokenizer::resolve_value(
                        token,
                        &self.stack,
                        &self.globals,
                        self.current_locals(),
                    )?;
                    elements.push(val);
                }
                Ok(Command::List(elements))
            }
            "INDEX" => {
                if tokens.len() < 3 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "INDEX requires two arguments: list and index".to_string(),
                    });
                }
                let list_val = Tokenizer::resolve_value(
                    &tokens[1],
                    &self.stack,
                    &self.globals,
                    self.current_locals(),
                )?;
                let idx_val = Tokenizer::resolve_value(
                    &tokens[2],
                    &self.stack,
                    &self.globals,
                    self.current_locals(),
                )?;
                Ok(Command::Index(list_val, idx_val))
            }
            "APPEND" => {
                if tokens.len() < 3 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "APPEND requires two arguments: list and element".to_string(),
                    });
                }
                let list_val = Tokenizer::resolve_value(
                    &tokens[1],
                    &self.stack,
                    &self.globals,
                    self.current_locals(),
                )?;
                let el_val = Tokenizer::resolve_value(
                    &tokens[2],
                    &self.stack,
                    &self.globals,
                    self.current_locals(),
                )?;
                Ok(Command::Append(list_val, el_val))
            }
            _ => Err(InterpError::Syntax {
                line: self.line_num + 1,
                message: format!("Unknown command '{}'", cmd_str),
            }),
        }
    }

    /// Resolves two arguments for binary operations (ADD, SUB, etc.).
    fn parse_binary_args(&self, tokens: &[String]) -> InterpResult<(Option<Value>, Option<Value>)> {
        if tokens.len() == 1 {
            if self.stack.len() < 2 {
                return Err(InterpError::Semantic {
                    line: self.line_num + 1,
                    message: "Not enough values on stack for binary operation".to_string(),
                });
            }
            Ok((None, None))
        } else if tokens.len() >= 3 {
            let a = Tokenizer::resolve_value(
                &tokens[1],
                &self.stack,
                &self.globals,
                self.current_locals(),
            )?;
            let b = Tokenizer::resolve_value(
                &tokens[2],
                &self.stack,
                &self.globals,
                self.current_locals(),
            )?;
            Ok((Some(a), Some(b)))
        } else {
            Err(InterpError::Syntax {
                line: self.line_num + 1,
                message: "Invalid arguments for binary operation".to_string(),
            })
        }
    }

    /// Resolves binary operands from Option<Value> or from the stack.
    fn resolve_binary_args(
        &mut self,
        a_opt: &Option<Value>,
        b_opt: &Option<Value>,
        line: usize,
    ) -> InterpResult<(Value, Value)> {
        match (a_opt, b_opt) {
            (Some(l), Some(r)) => Ok((l.clone(), r.clone())),
            (None, None) => {
                if self.stack.len() < 2 {
                    return Err(InterpError::Runtime {
                        line,
                        message: "Not enough values on stack for operation".to_string(),
                    });
                }
                let r = self.stack.pop().ok_or_else(|| InterpError::Runtime {
                    line,
                    message: "Stack underflow".to_string(),
                })?;
                let l = self.stack.pop().ok_or_else(|| InterpError::Runtime {
                    line,
                    message: "Stack underflow".to_string(),
                })?;
                Ok((l, r))
            }
            _ => Err(InterpError::Syntax {
                line,
                message: "Invalid arguments for binary operation".to_string(),
            }),
        }
    }

    /// Evaluates a binary operation and returns a Value (for arithmetic, comparison, logic).
    fn evaluate_binary_op(
        op: BinOp,
        left: &Value,
        right: &Value,
        line: usize,
    ) -> InterpResult<Value> {
        match op {
            BinOp::Add => Self::apply_arithmetic(left, right, |x, y| x + y, |x, y| x + y, line),
            BinOp::Sub => Self::apply_arithmetic(left, right, |x, y| x - y, |x, y| x - y, line),
            BinOp::Mul => Self::apply_arithmetic(left, right, |x, y| x * y, |x, y| x * y, line),
            BinOp::Div => {
                if crate::utils::is_zero(right) {
                    return Err(InterpError::Runtime {
                        line,
                        message: "Division by zero".to_string(),
                    });
                }
                Self::apply_arithmetic(left, right, |x, y| x / y, |x, y| x / y, line)
            }
            BinOp::Mod => {
                if crate::utils::is_zero(right) {
                    return Err(InterpError::Runtime {
                        line,
                        message: "Modulo by zero".to_string(),
                    });
                }
                match (left, right) {
                    (Value::Int(ai), Value::Int(bi)) => Ok(Value::Int(ai % bi)),
                    _ => {
                        let af = match left {
                            Value::Int(i) => *i as f64,
                            Value::Float(f) => *f,
                            _ => {
                                return Err(InterpError::Runtime {
                                    line,
                                    message: "Operands must be numbers".to_string(),
                                });
                            }
                        };
                        let bf = match right {
                            Value::Int(i) => *i as f64,
                            Value::Float(f) => *f,
                            _ => {
                                return Err(InterpError::Runtime {
                                    line,
                                    message: "Operands must be numbers".to_string(),
                                });
                            }
                        };
                        Ok(Value::Float(af % bf))
                    }
                }
            }
            BinOp::Pow => match (left, right) {
                (Value::Int(ai), Value::Int(bi)) => {
                    if *bi < 0 {
                        let af = *ai as f64;
                        let bf = *bi as f64;
                        Ok(Value::Float(af.powf(bf)))
                    } else {
                        match ai.checked_pow(*bi as u32) {
                            Some(result) => Ok(Value::Int(result)),
                            None => {
                                let af = *ai as f64;
                                let bf = *bi as f64;
                                Ok(Value::Float(af.powf(bf)))
                            }
                        }
                    }
                }
                _ => {
                    let af = match left {
                        Value::Int(i) => *i as f64,
                        Value::Float(f) => *f,
                        _ => {
                            return Err(InterpError::Runtime {
                                line,
                                message: "Operands must be numbers".to_string(),
                            });
                        }
                    };
                    let bf = match right {
                        Value::Int(i) => *i as f64,
                        Value::Float(f) => *f,
                        _ => {
                            return Err(InterpError::Runtime {
                                line,
                                message: "Operands must be numbers".to_string(),
                            });
                        }
                    };
                    Ok(Value::Float(af.powf(bf)))
                }
            },
            _ => {
                let bool_result = Self::evaluate_binary_op_bool(op, left, right, line)?;
                Ok(Value::Bool(bool_result))
            }
        }
    }

    /// Evaluates a binary operation that yields a boolean (comparisons and logical AND/OR).
    fn evaluate_binary_op_bool(
        op: BinOp,
        left: &Value,
        right: &Value,
        line: usize,
    ) -> InterpResult<bool> {
        match op {
            BinOp::Eq | BinOp::Ne | BinOp::Gt | BinOp::Ge | BinOp::Lt | BinOp::Le => {
                Self::compare_values(left, right, op, line)
            }
            BinOp::And => Ok(left.as_bool() && right.as_bool()),
            BinOp::Or => Ok(left.as_bool() || right.as_bool()),
            _ => Err(InterpError::Internal(format!(
                "Non-boolean operation: {:?}",
                op
            ))),
        }
    }

    /// Compares two values according to the comparison operator.
    fn compare_values(left: &Value, right: &Value, op: BinOp, line: usize) -> InterpResult<bool> {
        use std::cmp::Ordering;
        let cmp_result = match (left, right) {
            (Value::Int(li), Value::Int(ri)) => Some(li.cmp(ri)),
            (Value::Int(li), Value::Float(rf)) => {
                Some((*li as f64).partial_cmp(rf).unwrap_or(Ordering::Equal))
            }
            (Value::Float(lf), Value::Int(ri)) => {
                Some(lf.partial_cmp(&(*ri as f64)).unwrap_or(Ordering::Equal))
            }
            (Value::Float(lf), Value::Float(rf)) => {
                Some(lf.partial_cmp(rf).unwrap_or(Ordering::Equal))
            }
            (Value::String(ls), Value::String(rs)) => Some(ls.cmp(rs)),
            (Value::Bool(lb), Value::Bool(rb)) => Some(lb.cmp(rb)),
            _ => None,
        };

        match (op, cmp_result) {
            (BinOp::Eq, Some(ord)) => Ok(ord == Ordering::Equal),
            (BinOp::Ne, Some(ord)) => Ok(ord != Ordering::Equal),
            (BinOp::Gt, Some(ord)) => Ok(ord == Ordering::Greater),
            (BinOp::Ge, Some(ord)) => Ok(ord == Ordering::Greater || ord == Ordering::Equal),
            (BinOp::Lt, Some(ord)) => Ok(ord == Ordering::Less),
            (BinOp::Le, Some(ord)) => Ok(ord == Ordering::Less || ord == Ordering::Equal),
            _ => Err(InterpError::Runtime {
                line,
                message: format!("Cannot compare values of types {:?} and {:?}", left, right),
            }),
        }
    }

    /// Helper to apply arithmetic operations on two Values.
    fn apply_arithmetic<FInt, FFloat>(
        a: &Value,
        b: &Value,
        op_int: FInt,
        op_float: FFloat,
        line: usize,
    ) -> InterpResult<Value>
    where
        FInt: Fn(i64, i64) -> i64,
        FFloat: Fn(f64, f64) -> f64,
    {
        match (a, b) {
            (Value::Int(ai), Value::Int(bi)) => {
                let result = op_int(*ai, *bi);
                Ok(Value::Int(result))
            }
            _ => {
                let af = match a {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: "Operands must be numbers".to_string(),
                        });
                    }
                };
                let bf = match b {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: "Operands must be numbers".to_string(),
                        });
                    }
                };
                let result = op_float(af, bf);
                Ok(Value::Float(result))
            }
        }
    }

    /// Executes a single command.
    fn execute_command(&mut self, cmd: &Command) -> InterpResult<()> {
        let line = self.line_num + 1;
        match cmd {
            Command::Hello => {
                println!("Hello, World!");
                self.line_num += 1;
            }

            Command::Push(val) => {
                self.stack.push(val.clone());
                self.line_num += 1;
            }

            Command::Pop(var_opt) => {
                if self.stack.is_empty() {
                    return Err(InterpError::Runtime {
                        line,
                        message: "Cannot POP from empty stack".to_string(),
                    });
                }

                let value = self.stack.pop().ok_or_else(|| InterpError::Runtime {
                    line,
                    message: "Stack underflow".to_string(),
                })?;

                if let Some(var) = var_opt {
                    if self.is_global_scope() {
                        self.globals.insert(var.clone(), value);
                    } else if let Some(locals) = self.current_locals_mut() {
                        locals.insert(var.clone(), value);
                    } else {
                        self.globals.insert(var.clone(), value);
                    }
                }
                self.line_num += 1;
            }

            Command::Let(name, value) => {
                if self.is_global_scope() {
                    self.globals.insert(name.clone(), value.clone());
                } else if let Some(locals) = self.current_locals_mut() {
                    locals.insert(name.clone(), value.clone());
                } else {
                    self.globals.insert(name.clone(), value.clone());
                }
                self.line_num += 1;
            }

            Command::Print(args) => {
                let mut output = String::new();
                for v in args {
                    output.push_str(&v.to_string());
                }
                println!("{}", output);
                self.line_num += 1;
            }

            Command::Input(prompt_opt, var) => {
                if let Some(prompt) = prompt_opt {
                    print!("{}", prompt);
                    std::io::stdout().flush().map_err(InterpError::Io)?;
                }

                let mut input = String::new();
                let bytes_read = std::io::stdin()
                    .read_line(&mut input)
                    .map_err(InterpError::Io)?;

                if bytes_read == 0 {
                    return Err(InterpError::Runtime {
                        line,
                        message: "EOF reached while reading input".to_string(),
                    });
                }

                let input = input.trim_end_matches(&['\n', '\r'][..]);
                let value = crate::utils::parse(input);

                if self.is_global_scope() {
                    self.globals.insert(var.clone(), value);
                } else if let Some(locals) = self.current_locals_mut() {
                    locals.insert(var.clone(), value);
                } else {
                    self.globals.insert(var.clone(), value);
                }
                self.line_num += 1;
            }

            // ---------- Binary ----------
            Command::Binary(op, a_opt, b_opt) => {
                let (left, right) = self.resolve_binary_args(a_opt, b_opt, line)?;
                let result = Self::evaluate_binary_op(op.clone(), &left, &right, line)?;
                self.stack.push(result);
                self.line_num += 1;
            }

            Command::Not(a) => {
                let value = match a {
                    Some(v) => v.clone(),
                    None => {
                        if self.stack.len() < 1 {
                            return Err(InterpError::Runtime {
                                line,
                                message: "Not enough values on stack for logic operation"
                                    .to_string(),
                            });
                        }
                        self.stack.pop().ok_or_else(|| InterpError::Runtime {
                            line,
                            message: "Stack underflow".to_string(),
                        })?
                    }
                };

                let result = !value.as_bool();

                self.stack.push(Value::Bool(result));
                self.line_num += 1;
            }

            Command::If(cond) => {
                if !cond.as_bool() {
                    let target = self.if_jump_map.get(&self.line_num).ok_or_else(|| {
                        InterpError::Internal("No matching jump target for IF".to_string())
                    })?;
                    self.line_num = *target + 1;
                } else {
                    self.line_num += 1;
                }
            }

            Command::Else => {
                let target = self.else_jump_map.get(&self.line_num).ok_or_else(|| {
                    InterpError::Internal("No matching ENDIF for ELSE".to_string())
                })?;
                self.line_num = *target;
            }

            Command::Endif => {
                self.line_num += 1;
            }

            Command::While(cond) => {
                if !cond.as_bool() {
                    let target = self.loop_start_map.get(&self.line_num).ok_or_else(|| {
                        InterpError::Internal("No matching DO for WHILE".to_string())
                    })?;
                    self.line_num = target + 1;
                } else {
                    if let Some(do_line) = self.loop_start_map.get(&self.line_num) {
                        let frame = self.call_stack.last_mut().ok_or_else(|| {
                            InterpError::Internal("No call frame available".to_string())
                        })?;
                        frame.active_loops.push(*do_line);
                    }
                    self.line_num += 1;
                }
            }

            Command::Do => {
                let target = self.loop_back_map.get(&self.line_num).ok_or_else(|| {
                    InterpError::Internal("DO without matching WHILE".to_string())
                })?;

                let frame = self
                    .call_stack
                    .last_mut()
                    .ok_or_else(|| InterpError::Internal("No call frame available".to_string()))?;

                if let Some(&last) = frame.active_loops.last() {
                    if last == self.line_num {
                        frame.active_loops.pop();
                    }
                }

                self.line_num = *target;
            }

            Command::Break => {
                let frame = self
                    .call_stack
                    .last_mut()
                    .ok_or_else(|| InterpError::Internal("No call frame available".to_string()))?;

                let do_line = frame
                    .active_loops
                    .pop()
                    .ok_or_else(|| InterpError::Runtime {
                        line,
                        message: "BREAK used outside of any loop".to_string(),
                    })?;
                self.line_num = do_line + 1;
            }

            Command::Func(name) => {
                let end = self.func_end_map.get(name).ok_or_else(|| {
                    InterpError::Internal(format!("Function '{}' has no ENDF", name))
                })?;
                self.line_num = end + 1;
            }

            Command::Ret => {
                if let Some(frame) = self.call_stack.pop() {
                    self.line_num = frame.return_line;
                } else {
                    return Err(InterpError::Runtime {
                        line,
                        message: "RET outside of any function".to_string(),
                    });
                }
            }

            Command::Endf => {
                if let Some(frame) = self.call_stack.pop() {
                    self.line_num = frame.return_line;
                } else {
                    return Err(InterpError::Runtime {
                        line,
                        message: "ENDF outside of any function".to_string(),
                    });
                }
            }

            Command::Call(name) => {
                let start =
                    self.func_start_map
                        .get(name)
                        .ok_or_else(|| InterpError::FuncNotFound {
                            name: name.clone(),
                            line,
                        })?;
                let return_line = self.line_num + 1;
                let new_frame = CallFrame::new(return_line);
                self.call_stack.push(new_frame);
                self.line_num = start + 1;
            }

            Command::Len(arg_opt) => {
                let val = match arg_opt {
                    Some(v) => v.clone(),
                    None => self.stack.pop().ok_or_else(|| InterpError::Runtime {
                        line,
                        message: "Stack underflow in LEN".to_string(),
                    })?,
                };
                let result = match &val {
                    Value::String(s) => Value::Int(s.len() as i64),
                    Value::List(vec) => Value::Int(vec.len() as i64),
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("LEN requires a string, got {:?}", val),
                        });
                    }
                };
                self.stack.push(result);
                self.line_num += 1;
            }

            Command::Concat(a_opt, b_opt) => {
                let (left, right) = match (a_opt, b_opt) {
                    (Some(l), Some(r)) => (l.clone(), r.clone()),
                    (None, None) => {
                        let r = self.stack.pop().ok_or_else(|| InterpError::Runtime {
                            line,
                            message: "Stack underflow in CONCAT".to_string(),
                        })?;
                        let l = self.stack.pop().ok_or_else(|| InterpError::Runtime {
                            line,
                            message: "Stack underflow in CONCAT".to_string(),
                        })?;
                        (l, r)
                    }
                    _ => {
                        return Err(InterpError::Syntax {
                            line,
                            message: "Invalid arguments for CONCAT".to_string(),
                        });
                    }
                };
                let result = match (&left, &right) {
                    (Value::String(ls), Value::String(rs)) => {
                        Value::String(format!("{}{}", ls, rs))
                    }
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!(
                                "CONCAT requires two strings, got {:?} and {:?}",
                                left, right
                            ),
                        });
                    }
                };
                self.stack.push(result);
                self.line_num += 1;
            }

            Command::Substr(s_opt, start_opt, len_opt) => {
                let (string, start, len) = match (s_opt, start_opt, len_opt) {
                    (Some(s), Some(st), Some(l)) => (s.clone(), st.clone(), l.clone()),
                    (None, None, None) => {
                        let l = self.stack.pop().ok_or_else(|| InterpError::Runtime {
                            line,
                            message: "Stack underflow in SUBSTR".to_string(),
                        })?;
                        let st = self.stack.pop().ok_or_else(|| InterpError::Runtime {
                            line,
                            message: "Stack underflow in SUBSTR".to_string(),
                        })?;
                        let s = self.stack.pop().ok_or_else(|| InterpError::Runtime {
                            line,
                            message: "Stack underflow in SUBSTR".to_string(),
                        })?;
                        (s, st, l)
                    }
                    _ => {
                        return Err(InterpError::Syntax {
                            line,
                            message: "SUBSTR requires 3 arguments or none (to use stack)"
                                .to_string(),
                        });
                    }
                };
                let string_val = match string {
                    Value::String(s) => s,
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("SUBSTR requires a string, got {:?}", string),
                        });
                    }
                };
                let start_val = match start {
                    Value::Int(i) => i,
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("SUBSTR start requires integer, got {:?}", start),
                        });
                    }
                };
                let len_val = match len {
                    Value::Int(i) => i,
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("SUBSTR length requires integer, got {:?}", len),
                        });
                    }
                };

                if start_val < 0 || len_val < 0 {
                    return Err(InterpError::Runtime {
                        line,
                        message: "SUBSTR start and length must be non-negative".to_string(),
                    });
                }

                let start_idx = start_val as usize;
                let end_idx = start_idx + len_val as usize;
                if start_idx > string_val.len() {
                    self.stack.push(Value::String(String::new()));
                } else {
                    let substr = if end_idx > string_val.len() {
                        &string_val[start_idx..]
                    } else {
                        &string_val[start_idx..end_idx]
                    };
                    self.stack.push(Value::String(substr.to_string()));
                }
                self.line_num += 1;
            }

            Command::Upper(arg_opt) => {
                let val = match arg_opt {
                    Some(v) => v.clone(),
                    None => self.stack.pop().ok_or_else(|| InterpError::Runtime {
                        line,
                        message: "Stack underflow in UPPER".to_string(),
                    })?,
                };
                let result = match val {
                    Value::String(s) => Value::String(s.to_uppercase()),
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("UPPER requires a string, got {:?}", val),
                        });
                    }
                };
                self.stack.push(result);
                self.line_num += 1;
            }

            Command::Lower(arg_opt) => {
                let val = match arg_opt {
                    Some(v) => v.clone(),
                    None => self.stack.pop().ok_or_else(|| InterpError::Runtime {
                        line,
                        message: "Stack underflow in LOWER".to_string(),
                    })?,
                };
                let result = match val {
                    Value::String(s) => Value::String(s.to_lowercase()),
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("LOWER requires a string, got {:?}", val),
                        });
                    }
                };
                self.stack.push(result);
                self.line_num += 1;
            }

            Command::Trim(arg_opt) => {
                let val = match arg_opt {
                    Some(v) => v.clone(),
                    None => self.stack.pop().ok_or_else(|| InterpError::Runtime {
                        line,
                        message: "Stack underflow in TRIM".to_string(),
                    })?,
                };
                let result = match val {
                    Value::String(s) => Value::String(s.trim().to_string()),
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("TRIM requires a string, got {:?}", val),
                        });
                    }
                };
                self.stack.push(result);
                self.line_num += 1;
            }

            Command::List(elements) => {
                self.stack.push(Value::List(elements.clone()));
                self.line_num += 1;
            }

            Command::Index(list_val, idx_val) => {
                let list = match list_val {
                    Value::List(vec) => vec,
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("INDEX requires a list, got {:?}", list_val),
                        });
                    }
                };
                let idx = match idx_val {
                    Value::Int(i) => *i,
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("INDEX index must be integer, got {:?}", idx_val),
                        });
                    }
                };
                if idx < 0 || idx >= list.len() as i64 {
                    return Err(InterpError::Runtime {
                        line,
                        message: format!("Index {} out of bounds (len={})", idx, list.len()),
                    });
                }
                let element = list[idx as usize].clone();
                self.stack.push(element);
                self.line_num += 1;
            }

            Command::Append(list_val, el_val) => {
                let list = match list_val {
                    Value::List(vec) => vec,
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!("APPEND requires a list, got {:?}", list_val),
                        });
                    }
                };
                let mut new_list = list.clone();
                new_list.push(el_val.clone());
                self.stack.push(Value::List(new_list));
                self.line_num += 1;
            }
        }
        Ok(())
    }
}
