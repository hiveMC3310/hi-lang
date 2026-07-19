//! Core interpreter: runs Hi programs.

use crate::commands::Command;
use crate::error::{InterpError, InterpResult};
use crate::tokenizer::Tokenizer;
use crate::value::Value;
use std::collections::HashMap;
use std::io::Write;

/// A call frame: holds return address, local variables, and active loops for BREAK.
struct CallFrame {
    return_line: usize,
    locals: HashMap<String, Value>,
    active_loops: Vec<usize>,
}

impl CallFrame {
    fn new(return_line: usize) -> Self {
        Self {
            return_line,
            locals: HashMap::new(),
            active_loops: Vec::new(),
        }
    }
}

/// The interpreter state: lines, stack, variables, jump maps, and call stack.
pub struct Interpreter {
    lines: Vec<String>,
    line_num: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    call_stack: Vec<CallFrame>,
    if_jump_map: HashMap<usize, usize>,
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
            loop_start_map: HashMap::new(),
            loop_back_map: HashMap::new(),
            func_start_map: HashMap::new(),
            func_end_map: HashMap::new(),
        };
        s.call_stack.push(CallFrame::new(0));
        s
    }

    /// Runs the program. Returns an error if any occurs.
    pub fn run(&mut self) -> InterpResult<()> {
        self.build_maps()?;

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

    /// Builds jump maps for IF/WHILE/FUNC structures.
    fn build_maps(&mut self) -> InterpResult<()> {
        let mut if_stack = Vec::new();
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
                    if self.func_start_map.contains_key(&name) {
                        return Err(InterpError::Syntax {
                            line: i + 1,
                            message: format!("Function '{}' already defined", name),
                        });
                    }
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
                "IF" => if_stack.push(i),
                "ENDIF" => {
                    if let Some(start) = if_stack.pop() {
                        self.if_jump_map.insert(start, i);
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
            "ADD" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Add(a, b))
            }
            "SUB" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Sub(a, b))
            }
            "MUL" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Mul(a, b))
            }
            "DIV" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Div(a, b))
            }
            "EQ" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Eq(a, b))
            }
            "NE" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Ne(a, b))
            }
            "GT" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Gt(a, b))
            }
            "GE" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Ge(a, b))
            }
            "LT" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Lt(a, b))
            }
            "LE" => {
                let (a, b) = self.parse_binary_args(tokens)?;
                Ok(Command::Le(a, b))
            }
            "IF" => {
                if tokens.len() < 2 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "IF requires a condition".to_string(),
                    });
                }
                let cond = Tokenizer::resolve_value(
                    &tokens[1],
                    &self.stack,
                    &self.globals,
                    self.current_locals(),
                )?;
                Ok(Command::If(cond))
            }
            "ENDIF" => Ok(Command::Endif),
            "WHILE" => {
                if tokens.len() < 2 {
                    return Err(InterpError::Syntax {
                        line: self.line_num + 1,
                        message: "WHILE requires a condition".to_string(),
                    });
                }
                let cond = Tokenizer::resolve_value(
                    &tokens[1],
                    &self.stack,
                    &self.globals,
                    self.current_locals(),
                )?;
                Ok(Command::While(cond))
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
                    if let Some(locals) = self.current_locals_mut() {
                        locals.insert(var.clone(), value);
                    } else {
                        self.globals.insert(var.clone(), value);
                    }
                }
                self.line_num += 1;
            }

            Command::Let(name, value) => {
                if let Some(locals) = self.current_locals_mut() {
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
                    // Безопасный вывод приглашения
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

                if let Some(locals) = self.current_locals_mut() {
                    locals.insert(var.clone(), value);
                } else {
                    self.globals.insert(var.clone(), value);
                }
                self.line_num += 1;
            }

            // ---------- Арифметические операции ----------
            Command::Add(a, b) | Command::Sub(a, b) | Command::Mul(a, b) | Command::Div(a, b) => {
                let (left, right) = match (a, b) {
                    (Some(l), Some(r)) => (l.clone(), r.clone()),
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
                        (l, r)
                    }
                    _ => {
                        return Err(InterpError::Syntax {
                            line,
                            message: "Invalid arguments for binary operation".to_string(),
                        });
                    }
                };

                let result = match cmd {
                    Command::Add(_, _) => {
                        Self::apply_arithmetic(&left, &right, |x, y| x + y, |x, y| x + y, line)?
                    }
                    Command::Sub(_, _) => {
                        Self::apply_arithmetic(&left, &right, |x, y| x - y, |x, y| x - y, line)?
                    }
                    Command::Mul(_, _) => {
                        Self::apply_arithmetic(&left, &right, |x, y| x * y, |x, y| x * y, line)?
                    }
                    Command::Div(_, _) => {
                        if crate::utils::is_zero(&right) {
                            return Err(InterpError::Runtime {
                                line,
                                message: "Division by zero".to_string(),
                            });
                        }
                        Self::apply_arithmetic(&left, &right, |x, y| x / y, |x, y| x / y, line)?
                    }
                    _ => unreachable!(),
                };

                self.stack.push(result);
                self.line_num += 1;
            }

            // ---------- Операции сравнения ----------
            Command::Eq(a, b)
            | Command::Ne(a, b)
            | Command::Gt(a, b)
            | Command::Ge(a, b)
            | Command::Lt(a, b)
            | Command::Le(a, b) => {
                let (left, right) = match (a, b) {
                    (Some(l), Some(r)) => (l.clone(), r.clone()),
                    (None, None) => {
                        if self.stack.len() < 2 {
                            return Err(InterpError::Runtime {
                                line,
                                message: "Not enough values on stack for comparison".to_string(),
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
                        (l, r)
                    }
                    _ => {
                        return Err(InterpError::Syntax {
                            line,
                            message: "Invalid arguments for comparison".to_string(),
                        });
                    }
                };

                use std::cmp::Ordering;
                let cmp_result = match (&left, &right) {
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

                let result_bool = match (cmd, cmp_result) {
                    (Command::Eq(_, _), Some(ord)) => ord == Ordering::Equal,
                    (Command::Ne(_, _), Some(ord)) => ord != Ordering::Equal,
                    (Command::Gt(_, _), Some(ord)) => ord == Ordering::Greater,
                    (Command::Ge(_, _), Some(ord)) => {
                        ord == Ordering::Greater || ord == Ordering::Equal
                    }
                    (Command::Lt(_, _), Some(ord)) => ord == Ordering::Less,
                    (Command::Le(_, _), Some(ord)) => {
                        ord == Ordering::Less || ord == Ordering::Equal
                    }
                    _ => {
                        return Err(InterpError::Runtime {
                            line,
                            message: format!(
                                "Cannot compare values of types {:?} and {:?}",
                                left, right
                            ),
                        });
                    }
                };

                self.stack.push(Value::Bool(result_bool));
                self.line_num += 1;
            }

            Command::If(cond) => {
                let condition = cond.as_bool();
                if !condition {
                    let target = self.if_jump_map.get(&self.line_num).ok_or_else(|| {
                        InterpError::Internal("No matching ENDIF for IF".to_string())
                    })?;
                    self.line_num = target + 1;
                } else {
                    self.line_num += 1;
                }
            }

            Command::Endif => {
                self.line_num += 1;
            }

            Command::While(cond) => {
                let condition = cond.as_bool();
                if !condition {
                    let target = self.loop_start_map.get(&self.line_num).ok_or_else(|| {
                        InterpError::Internal("No matching DO for WHILE".to_string())
                    })?;
                    self.line_num = target + 1;
                } else {
                    if let Some(do_line) = self.loop_start_map.get(&self.line_num) {
                        let frame = self.call_stack.last_mut().unwrap();
                        frame.active_loops.push(*do_line);
                    }
                    self.line_num += 1;
                }
            }

            Command::Do => {
                let target = self.loop_back_map.get(&self.line_num).ok_or_else(|| {
                    InterpError::Internal("DO without matching WHILE".to_string())
                })?;

                if let Some(&last) = self.call_stack.last_mut().unwrap().active_loops.last()
                    && last == self.line_num
                {
                    self.call_stack.last_mut().unwrap().active_loops.pop();
                }
                self.line_num = *target;
            }

            Command::Break => {
                let do_line = self
                    .call_stack
                    .last_mut()
                    .unwrap()
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
        }
        Ok(())
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
}
