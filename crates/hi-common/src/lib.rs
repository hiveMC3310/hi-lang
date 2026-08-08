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

    /// Вернуть символ для строки, добавив её при необходимости.
    pub fn intern(&mut self, s: &str) -> Symbol {
        // Поиск по существующей аллокации
        if let Some(&sym) = self.map.get(s) {
            return sym;
        }
        let idx = self.strings.len() as u32;
        let boxed: Box<str> = s.into();
        self.strings.push(boxed.clone());
        self.map.insert(boxed, Symbol(idx));
        Symbol(idx)
    }

    /// Восстановить строку по символу.
    pub fn resolve(&self, sym: Symbol) -> &str {
        &self.strings[sym.0 as usize]
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", INTERNER.lock().unwrap().resolve(*self))
    }
}

lazy_static::lazy_static! {
    static ref INTERNER: Mutex<Interner> = Mutex::new(Interner::new());
}

/// Глобальный метод для интернирования.
pub fn intern(s: &str) -> Symbol {
    INTERNER.lock().unwrap().intern(s)
}

/// Глобальный метод для разрешения символа (возвращает String для удобства,
/// но стараемся использовать Display или доступ к &str через Interner напрямую).
pub fn resolve(sym: Symbol) -> String {
    INTERNER.lock().unwrap().resolve(sym).to_owned()
}
