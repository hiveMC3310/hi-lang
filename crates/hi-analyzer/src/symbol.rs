//! Symbol table and symbol definitions.
use hi_common::Symbol as HiSymbol;
use hi_interpreter::ast::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: HiSymbol,
    pub kind: SymbolKind,
    pub span: Span,
    pub defined_at: Option<Span>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Function(Vec<HiSymbol>),
    BuiltinFunction(Vec<HiSymbol>),
    Module,
    Builtin,
}

impl SymbolKind {
    pub fn signature(&self) -> String {
        match self {
            SymbolKind::Variable | SymbolKind::Builtin | SymbolKind::Module => String::new(),
            SymbolKind::Function(params) | SymbolKind::BuiltinFunction(params) => {
                if params.is_empty() {
                    String::new()
                } else if params.len() == 1 && hi_common::resolve(params[0]) == "..." {
                    "(...)".to_string()
                } else {
                    let args = params
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("({})", args)
                }
            }
        }
    }
}
