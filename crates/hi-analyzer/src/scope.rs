//! Scoping and resolution.

use crate::symbol::Symbol as Sym;
use hi_common::Symbol;
use std::collections::HashMap;
use std::sync::Arc;

/// Простая цепочка областей видимости.
#[derive(Debug, Clone)]
pub struct Scope {
    symbols: HashMap<Symbol, Sym>,
    parent: Option<Arc<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    /// Создаёт дочернюю область видимости.
    pub fn child(&self) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: Some(Arc::new(self.clone())),
        }
    }

    /// Ищет символ по имени, поднимаясь по цепочке.
    pub fn lookup(&self, name: Symbol) -> Option<&Sym> {
        self.symbols
            .get(&name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    /// Определяет символ в текущей области.
    pub fn define(&mut self, sym: Sym) {
        self.symbols.insert(sym.name, sym);
    }
}
