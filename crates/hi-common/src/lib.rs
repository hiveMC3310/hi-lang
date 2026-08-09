//! Symbol interning for efficient string representation.
//!
//! This module provides a global interner that maps strings to unique `Symbol` values.
//! Symbols are cheap to copy and compare, and can be resolved back to the original string.
//! The interner is thread-safe via a `Mutex` and is lazily initialized.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

pub struct Interner {
    strings: Vec<Box<str>>,
    map: HashMap<Box<str>, Symbol>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            map: HashMap::new(),
        }
    }

    /// Returns a symbol for the string, interning it if necessary.
    pub fn intern(&mut self, s: &str) -> Symbol {
        // Look up existing allocation
        if let Some(&sym) = self.map.get(s) {
            return sym;
        }
        let idx = self.strings.len() as u32;
        let boxed: Box<str> = s.into();
        self.strings.push(boxed.clone());
        self.map.insert(boxed, Symbol(idx));
        Symbol(idx)
    }

    /// Retrieves the string corresponding to a symbol.
    pub fn resolve(&self, sym: Symbol) -> &str {
        &self.strings[sym.0 as usize]
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", INTERNER.lock()        .expect("Interner mutex is poisoned").resolve(*self))
    }
}

lazy_static::lazy_static! {
    static ref INTERNER: Mutex<Interner> = Mutex::new(Interner::new());
}

/// Global function to intern a string.
pub fn intern(s: &str) -> Symbol {
    INTERNER.lock()        .expect("Interner mutex is poisoned").intern(s)
}

/// Global function to resolve a symbol (returns a `String` for convenience,
/// but prefer using `Display` or direct `&str` access via the `Interner`).
pub fn resolve(sym: Symbol) -> String {
    INTERNER.lock()        .expect("Interner mutex is poisoned").resolve(sym).to_owned()
}
