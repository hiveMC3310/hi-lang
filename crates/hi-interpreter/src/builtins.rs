use crate::ast::Span;
use crate::error::InterpResult;
use crate::interpreter::Interpreter;
use crate::value::Value;
use hi_common::Symbol;
use std::collections::HashMap;

pub const KEYWORDS: &[&str] = &[
    "LET", "INPUT", "IF", "THEN", "ELSE", "END", "WHILE", "DO", "FOR", "TO", "NEXT", "FUNC", "RET",
    "BREAK", "PRINT", "IMPORT", "AS", "TRUE", "FALSE", "AND", "OR", "NOT",
];

pub type BuiltinFnImpl = fn(&mut Interpreter, &[Value], &Span) -> InterpResult<Value>;

// ---------- Структуры для inventory (остаются со строками) ----------
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

// ---------- Структуры для внешнего API (с Symbol) ----------
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

/// Возвращает список глобальных функций в виде Symbol.
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

/// Возвращает список функций модулей с Symbol.
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

/// Строит мапу модуль (Symbol) -> список функций с Symbol.
pub fn get_module_functions_map() -> HashMap<Symbol, Vec<ModuleFunctionSym>> {
    let mut map = HashMap::new();
    for func in get_module_functions() {
        map.entry(func.module).or_insert_with(Vec::new).push(func);
    }
    map
}

/// Возвращает список переменных модулей (имя как Symbol).
pub fn get_module_variables() -> Vec<(Symbol, Symbol)> {
    inventory::iter::<ModuleVariable>
        .into_iter()
        .map(|mv| (hi_common::intern(mv.module), hi_common::intern(mv.name)))
        .collect()
}

/// Строит мапу модуль (Symbol) -> список имён переменных (Symbol).
pub fn get_module_variables_map() -> HashMap<Symbol, Vec<Symbol>> {
    let mut map = HashMap::new();
    for (module, var) in get_module_variables() {
        map.entry(module).or_insert_with(Vec::new).push(var);
    }
    map
}
