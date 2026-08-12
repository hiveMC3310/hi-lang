//! Scoping and symbol resolution.

use crate::symbol::Symbol as Sym;
use hi_common::Symbol;
use std::collections::HashMap;
use std::sync::Arc;

/// Simple chain of scopes.
#[derive(Debug, Clone)]
pub struct Scope {
    symbols: HashMap<Symbol, Sym>,
    parent: Option<Arc<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    /// Creates a child scope.
    pub fn child(&self) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: Some(Arc::new(self.clone())),
        }
    }

    /// Looks up a symbol by name, walking up the chain.
    pub fn lookup(&self, name: Symbol) -> Option<&Sym> {
        self.symbols
            .get(&name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    /// Defines a symbol in the current scope.
    pub fn define(&mut self, sym: Sym) {
        self.symbols.insert(sym.name, sym);
    }
}
