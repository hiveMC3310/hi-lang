//! Built-in functions and modules registration.
//!
//! This module defines the core built-in functions and modules available in the Hi language.
//! It uses the `inventory` crate for compile-time registration of global functions,
//! module functions, and module variables. The module also provides helper functions
//! to retrieve these items as `Symbol`-based structures for use in the interpreter.
#![allow(dead_code)]

use crate::ast::Span;
use crate::error::InterpResult;
use crate::interpreter::Interpreter;
use crate::value::Value;
use hi_common::Symbol;
use std::collections::HashMap;

/// List of all reserved keywords in the Hi language.
pub const KEYWORDS: &[&str] = &[
    "LET", "INPUT", "IF", "THEN", "ELSE", "END", "WHILE", "DO", "FOR", "TO", "NEXT", "FUNC", "RET",
    "BREAK", "PRINT", "IMPORT", "AS", "TRUE", "FALSE", "AND", "OR", "NOT",
];

/// Type alias for a built-in function implementation.
///
/// Built-in functions take the interpreter, a slice of argument values, and a span
/// for error reporting, and return a `Value` or an interpreter error.
pub type BuiltinFnImpl = fn(&mut Interpreter, &[Value], &Span) -> InterpResult<Value>;

// ---------- Structures for inventory (use string literals) ----------
#[derive(Clone)]
pub struct GlobalFunction {
    pub name: &'static str,
    pub params: &'static [&'static str],
    pub doc: &'static str,
    pub func: BuiltinFnImpl,
}

#[derive(Clone)]
pub struct ModuleFunction {
    pub module: &'static str,
    pub name: &'static str,
    pub params: &'static [&'static str],
    pub doc: &'static str,
    pub func: BuiltinFnImpl,
}

#[derive(Clone)]
pub struct ModuleVariable {
    pub module: &'static str,
    pub name: &'static str,
}

inventory::collect!(GlobalFunction);
inventory::collect!(ModuleFunction);
inventory::collect!(ModuleVariable);

// ---------- Structures for the external API (using Symbol) ----------
#[derive(Clone)]
pub struct GlobalFunctionSym {
    pub name: Symbol,
    pub params: Vec<Symbol>,
    pub doc: &'static str,
    pub func: BuiltinFnImpl,
}

#[derive(Clone)]
pub struct ModuleFunctionSym {
    pub module: Symbol,
    pub name: Symbol,
    pub params: Vec<Symbol>,
    pub doc: &'static str,
    pub func: BuiltinFnImpl,
}

/// Returns a list of global functions as `Symbol`-based structures.
pub fn get_global_functions() -> Vec<GlobalFunctionSym> {
    inventory::iter::<GlobalFunction>
        .into_iter()
        .map(|gf| GlobalFunctionSym {
            name: hi_common::intern(gf.name),
            params: gf.params.iter().map(|s| hi_common::intern(s)).collect(),
            doc: gf.doc,
            func: gf.func,
        })
        .collect()
}

/// Returns a list of module functions as `Symbol`-based structures.
pub fn get_module_functions() -> Vec<ModuleFunctionSym> {
    inventory::iter::<ModuleFunction>
        .into_iter()
        .map(|mf| ModuleFunctionSym {
            module: hi_common::intern(mf.module),
            name: hi_common::intern(mf.name),
            params: mf.params.iter().map(|s| hi_common::intern(s)).collect(),
            doc: mf.doc,
            func: mf.func,
        })
        .collect()
}

/// Builds a map from module `Symbol` to a list of its functions (`ModuleFunctionSym`).
pub fn get_module_functions_map() -> HashMap<Symbol, Vec<ModuleFunctionSym>> {
    let mut map = HashMap::new();
    for func in get_module_functions() {
        map.entry(func.module).or_insert_with(Vec::new).push(func);
    }
    map
}

/// Returns a list of module variables as pairs of `(module_symbol, variable_symbol)`.
pub fn get_module_variables() -> Vec<(Symbol, Symbol)> {
    inventory::iter::<ModuleVariable>
        .into_iter()
        .map(|mv| (hi_common::intern(mv.module), hi_common::intern(mv.name)))
        .collect()
}

/// Builds a map from module `Symbol` to a list of its variable symbols.
pub fn get_module_variables_map() -> HashMap<Symbol, Vec<Symbol>> {
    let mut map = HashMap::new();
    for (module, var) in get_module_variables() {
        map.entry(module).or_insert_with(Vec::new).push(var);
    }
    map
}
