//! Semantic analysis: builds symbol table and resolves references.

use crate::scope::Scope;
use crate::symbol::{Symbol, SymbolKind};
use hi_common::Symbol as HiSymbol;
use hi_interpreter::ast::{Expr, Program, Span, Stmt};
use hi_interpreter::builtins;
use hi_interpreter::parser::Parser;
use hi_interpreter::parser::lexer::Lexer;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock as StdRwLock};

// -----------------------------------------------------------------------------
// AnalysisResult
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: HiSymbol,
    pub kind: SymbolKind,
    pub span: Span,
    pub defined_at: Option<Span>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AnalysisError {
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub parent: Option<usize>,
    pub span: Span,
    pub symbols: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub symbols: Vec<SymbolInfo>,
    pub errors: Vec<AnalysisError>,
    pub uses: Vec<(Span, HiSymbol)>,
    pub module_calls: Vec<(Span, String, String)>,
    pub imported_modules: HashSet<HiSymbol>,
    pub loaded_module_exports: HashMap<HiSymbol, Arc<Vec<SymbolInfo>>>,
    pub module_accesses: Vec<(Span, String, String)>,
    pub module_aliases: HashMap<HiSymbol, HiSymbol>,
    pub scopes: Vec<ScopeInfo>,
}

impl AnalysisResult {
    pub fn symbol_at(&self, line: usize, col: usize) -> Option<&SymbolInfo> {
        self.symbols.iter().find(|sym| {
            sym.span.start_line <= line
                && line <= sym.span.end_line
                && sym.span.start_col <= col
                && col <= sym.span.end_col
        })
    }

    pub fn use_at(&self, line: usize, col: usize) -> Option<(Span, HiSymbol)> {
        self.uses
            .iter()
            .find(|(use_span, _)| {
                use_span.start_line <= line
                    && line <= use_span.end_line
                    && use_span.start_col <= col
                    && col <= use_span.end_col
            })
            .map(|(s, n)| (*s, *n))
    }

    pub fn all_uses_of(&self, name: HiSymbol) -> Vec<Span> {
        let mut spans: Vec<Span> = self
            .uses
            .iter()
            .filter(|(_, n)| *n == name)
            .map(|(s, _)| *s)
            .collect();

        if let Some(sym) = self.symbols.iter().find(|s| s.name == name)
            && let Some(def_span) = sym.defined_at
        {
            spans.push(def_span);
        }
        spans
    }

    pub fn definition_at(&self, line: usize, col: usize) -> Option<Span> {
        if let Some((_, name)) = self.use_at(line, col) {
            self.symbols
                .iter()
                .find(|sym| sym.name == name)
                .and_then(|sym| sym.defined_at)
        } else {
            None
        }
    }

    pub fn all_symbols(&self) -> &[SymbolInfo] {
        &self.symbols
    }

    pub fn scope_at(&self, line: usize, col: usize) -> Option<&ScopeInfo> {
        self.scopes.iter().rev().find(|scope| {
            scope.span.start_line <= line
                && line <= scope.span.end_line
                && scope.span.start_col <= col
                && col <= scope.span.end_col
        })
    }

    pub fn visible_symbol_indices(&self, scope_idx: usize) -> Vec<usize> {
        let mut indices = Vec::new();
        let mut current = Some(scope_idx);
        while let Some(idx) = current {
            if let Some(scope) = self.scopes.get(idx) {
                indices.extend(&scope.symbols);
                current = scope.parent;
            } else {
                break;
            }
        }
        indices.sort();
        indices.dedup();
        indices
    }
}

// -----------------------------------------------------------------------------
// Analyzer
// -----------------------------------------------------------------------------

pub struct Analyzer {
    pub global_symbols: Vec<SymbolInfo>,
    pub builtin_module_names: Vec<HiSymbol>,
    global_scope: Scope,
    module_cache: StdRwLock<HashMap<PathBuf, Arc<Vec<SymbolInfo>>>>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn new() -> Self {
        let mut scope = Scope::new();
        let mut global_symbols = Vec::new();
        let mut builtin_module_names = Vec::new();

        for gf in builtins::get_global_functions() {
            let sym = Symbol {
                name: gf.name,
                kind: SymbolKind::BuiltinFunction(gf.params.clone()),
                span: Span::dummy(),
                defined_at: Some(Span::dummy()),
                doc: Some(gf.doc.to_string()),
            };
            scope.define(sym.clone());
            global_symbols.push(SymbolInfo {
                name: gf.name,
                kind: SymbolKind::BuiltinFunction(gf.params),
                span: Span::dummy(),
                defined_at: Some(Span::dummy()),
                doc: Some(gf.doc.to_string()),
            });
        }

        let args_sym = hi_common::intern("ARGS");
        let args_dict_sym = hi_common::intern("ARGS_DICT");

        let args_info = SymbolInfo {
            name: args_sym,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
            defined_at: Some(Span::dummy()),
            doc: Some("List of command-line arguments".to_string()),
        };
        global_symbols.push(args_info.clone());
        scope.define(Symbol {
            name: args_sym,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
            defined_at: Some(Span::dummy()),
            doc: Some("List of command-line arguments".to_string()),
        });

        let args_dict_info = SymbolInfo {
            name: args_dict_sym,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
            defined_at: Some(Span::dummy()),
            doc: Some("Dictionary of command-line flags".to_string()),
        };
        global_symbols.push(args_dict_info.clone());
        scope.define(Symbol {
            name: args_dict_sym,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
            defined_at: Some(Span::dummy()),
            doc: Some("Dictionary of command-line flags".to_string()),
        });

        for module_sym in builtins::get_module_functions_map().keys() {
            builtin_module_names.push(*module_sym);
        }

        Self {
            global_scope: scope,
            global_symbols,
            builtin_module_names,
            module_cache: StdRwLock::new(HashMap::new()),
        }
    }

    pub fn analyze(&self, program: &Program, current_file: Option<&Path>) -> AnalysisResult {
        let mut result = AnalysisResult::default();
        result.scopes.push(ScopeInfo {
            parent: None,
            span: Span {
                start_line: 1,
                start_col: 1,
                end_line: usize::MAX,
                end_col: usize::MAX,
            },
            symbols: Vec::new(),
        });
        let mut file_scope = self.global_scope.child();
        let mut aliases = HashMap::new();

        let mut file_analyzer = FileAnalyzer {
            scope: &mut file_scope,
            result: &mut result,
            module_aliases: &mut aliases,
            builtin_module_names: self.builtin_module_names.as_slice(),
            depth: 0,
            current_scope: 0,
            analyzer: self,
            current_file,
        };

        for stmt in &program.stmts {
            file_analyzer.analyze_stmt(stmt);
        }

        result.module_aliases = aliases;
        result
    }

    pub fn analyze_module(&self, program: &Program) -> Vec<SymbolInfo> {
        let mut result = AnalysisResult::default();
        result.scopes.push(ScopeInfo {
            parent: None,
            span: Span {
                start_line: 1,
                start_col: 1,
                end_line: usize::MAX,
                end_col: usize::MAX,
            },
            symbols: vec![],
        });

        let mut file_scope = self.global_scope.child();
        let mut aliases = HashMap::new();
        let mut file_analyzer = FileAnalyzer {
            scope: &mut file_scope,
            result: &mut result,
            module_aliases: &mut aliases,
            builtin_module_names: self.builtin_module_names.as_slice(),
            depth: 0,
            current_scope: 0,
            analyzer: self,
            current_file: None,
        };
        for stmt in &program.stmts {
            file_analyzer.analyze_stmt(stmt);
        }
        result.scopes[0]
            .symbols
            .iter()
            .map(|&idx| result.symbols[idx].clone())
            .collect()
    }
}

// -----------------------------------------------------------------------------
// FileAnalyzer
// -----------------------------------------------------------------------------

struct FileAnalyzer<'a> {
    scope: &'a mut Scope,
    result: &'a mut AnalysisResult,
    module_aliases: &'a mut HashMap<HiSymbol, HiSymbol>,
    builtin_module_names: &'a [HiSymbol],
    depth: usize,
    current_scope: usize,
    analyzer: &'a Analyzer,
    current_file: Option<&'a Path>,
}

impl<'a> FileAnalyzer<'a> {
    fn analyze_stmt(&mut self, stmt: &Stmt) {
        use Stmt::*;
        match stmt {
            Let(name, expr, name_span, _) => {
                self.define_symbol(*name, SymbolKind::Variable, *name_span, None);
                self.analyze_expr(expr);
            }
            Func(name, params, body, doc, name_span, full_span) => {
                self.define_symbol(
                    *name,
                    SymbolKind::Function(params.iter().map(|(s, _)| *s).collect()),
                    *name_span,
                    doc.clone(),
                );
                let new_scope_idx = self.result.scopes.len();
                self.result.scopes.push(ScopeInfo {
                    parent: Some(self.current_scope),
                    span: *full_span,
                    symbols: Vec::new(),
                });
                let old_scope_idx = self.current_scope;
                self.current_scope = new_scope_idx;

                let child = self.scope.child();
                let old_scope_env = std::mem::replace(self.scope, child);
                self.depth += 1;

                for (p, p_span) in params {
                    self.define_symbol(*p, SymbolKind::Variable, *p_span, None);
                }
                for s in body {
                    self.analyze_stmt(s);
                }

                self.depth -= 1;
                *self.scope = old_scope_env;
                self.current_scope = old_scope_idx;
            }
            Assign(left, right, _) => {
                if let hi_interpreter::ast::Expr::Variable(name, var_span) = &**left {
                    self.resolve_identifier(*name, *var_span);
                }
                self.analyze_expr(right);
            }
            CompoundAssign(left, _, right, _) => {
                if let hi_interpreter::ast::Expr::Variable(name, var_span) = &**left {
                    self.resolve_identifier(*name, *var_span);
                }
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            If(cond, then_block, else_block, _) => {
                self.analyze_expr(cond);
                for s in then_block {
                    self.analyze_stmt(s);
                }
                if let Some(else_block) = else_block {
                    for s in else_block {
                        self.analyze_stmt(s);
                    }
                }
            }
            While(cond, body, _) => {
                self.analyze_expr(cond);
                for s in body {
                    self.analyze_stmt(s);
                }
            }
            For(var, start, end, step, body, span, _) => {
                self.analyze_expr(start);
                self.analyze_expr(end);
                if let Some(step) = step {
                    self.analyze_expr(step);
                }
                self.define_symbol(*var, SymbolKind::Variable, *span, None);
                for s in body {
                    self.analyze_stmt(s);
                }
            }
            Return(expr, _) => {
                if let Some(e) = expr {
                    self.analyze_expr(e);
                }
            }
            Print(args, _) => {
                for e in args {
                    self.analyze_expr(e);
                }
            }
            Input(_, var, span) => {
                self.define_symbol(*var, SymbolKind::Variable, *span, None);
            }
            Import(path, alias, span) => {
                self.analyze_import(path, *alias, *span);
            }
            Break(_) => {}
            Expr(expr, _) => self.analyze_expr(expr),
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        use Expr::*;
        match expr {
            Variable(name, span) => self.resolve_identifier(*name, *span),
            Call(name, args, name_span, _) => {
                self.resolve_identifier(*name, *name_span);
                for a in args {
                    self.analyze_expr(a);
                }
            }
            Binary(_, left, right, _) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Unary(_, inner, _) => self.analyze_expr(inner),
            Index(base, index, _) => {
                self.analyze_expr(base);
                self.analyze_expr(index);
            }
            List(elems, _) => {
                for e in elems {
                    self.analyze_expr(e);
                }
            }
            Dict(pairs, _) => {
                for (k, v) in pairs {
                    self.analyze_expr(k);
                    self.analyze_expr(v);
                }
            }
            ModuleAccess(module, var, name_span, _) => {
                let real_module = *self.module_aliases.get(module).unwrap_or(module);
                if self.scope.lookup(*module).is_none() {
                    self.error(name_span, &format!("Undefined module '{}'", module));
                } else {
                    self.result.module_accesses.push((
                        *name_span,
                        hi_common::resolve(real_module),
                        hi_common::resolve(*var),
                    ));
                }
            }
            CallModule(module, func, args, name_span, _) => {
                let real_module = *self.module_aliases.get(module).unwrap_or(module);
                if self.scope.lookup(*module).is_none() {
                    self.error(name_span, &format!("Undefined module '{}'", module));
                } else {
                    self.result.module_calls.push((
                        *name_span,
                        hi_common::resolve(real_module),
                        hi_common::resolve(*func),
                    ));
                }
                for a in args {
                    self.analyze_expr(a);
                }
            }
            _ => {}
        }
    }

    fn analyze_import(&mut self, path: &str, alias: Option<HiSymbol>, span: Span) {
        // Determine if this is a user module (ends with .hi)
        if path.ends_with(".hi") {
            // User module: strip .hi and check it's not a built-in name
            let module_name = path.trim_end_matches(".hi");
            let module_sym = hi_common::intern(module_name);

            // Built-in modules do NOT have .hi extension – reject if name matches
            if self.builtin_module_names.contains(&module_sym) {
                self.error(
                    &span,
                    &format!(
                        "Cannot import built-in module '{}' with .hi extension. Use 'IMPORT \"{}\"' without extension.",
                        module_name, module_name
                    ),
                );
                return;
            }

            // Load user module from disk (same as before)
            let base_dir = self
                .current_file
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Path::new("."));
            let import_path = base_dir.join(path);
            let abs_path = import_path.canonicalize().unwrap_or(import_path);

            let symbols = {
                let cache = self.analyzer.module_cache.read().unwrap();
                if let Some(cached) = cache.get(&abs_path) {
                    cached.clone()
                } else {
                    drop(cache);
                    let source = match std::fs::read_to_string(&abs_path) {
                        Ok(s) => s,
                        Err(_) => {
                            self.error(&span, &format!("Cannot read module '{}'", path));
                            return;
                        }
                    };
                    let tokens = match Lexer::tokenize(&source) {
                        Ok(t) => t,
                        Err(_) => {
                            self.error(&span, "Lexer error in module");
                            return;
                        }
                    };
                    let mut parser = Parser::new(&tokens);
                    let program = match parser.parse() {
                        Ok(p) => p,
                        Err(_) => {
                            self.error(&span, "Parser error in module");
                            return;
                        }
                    };
                    let syms = self.analyzer.analyze_module(&program);
                    let syms = Arc::new(syms);
                    self.analyzer
                        .module_cache
                        .write()
                        .unwrap()
                        .insert(abs_path, syms.clone());
                    syms
                }
            };

            // Register the module symbol and its exports
            if self.scope.lookup(module_sym).is_none() {
                self.define_symbol(module_sym, SymbolKind::Module, span, None);
            }
            self.result.imported_modules.insert(module_sym);
            self.result
                .loaded_module_exports
                .insert(module_sym, symbols.clone());

            if let Some(alias_sym) = alias {
                self.module_aliases.insert(alias_sym, module_sym);
                self.define_symbol(alias_sym, SymbolKind::Module, span, None);
                self.result.module_aliases.insert(alias_sym, module_sym);
            } else {
                for sym in symbols.iter() {
                    self.define_symbol(sym.name, sym.kind.clone(), span, sym.doc.clone());
                }
            }
        } else {
            // Built-in module – no .hi extension
            let module_sym = hi_common::intern(path);
            if self.builtin_module_names.contains(&module_sym) {
                // Import built-in module
                if self.scope.lookup(module_sym).is_none() {
                    self.define_symbol(module_sym, SymbolKind::Module, span, None);
                }
                self.result.imported_modules.insert(module_sym);

                if let Some(alias_sym) = alias {
                    self.module_aliases.insert(alias_sym, module_sym);
                    self.define_symbol(alias_sym, SymbolKind::Module, span, None);
                    self.result.module_aliases.insert(alias_sym, module_sym);
                } else {
                    // Inline built-in functions and variables
                    if let Some(funcs) = builtins::get_module_functions_map().get(&module_sym) {
                        for mf in funcs {
                            self.define_symbol(
                                mf.name,
                                SymbolKind::BuiltinFunction(mf.params.clone()),
                                span,
                                Some(mf.doc.to_string()),
                            );
                        }
                    }
                    if let Some(vars) = builtins::get_module_variables_map().get(&module_sym) {
                        for var_sym in vars {
                            self.define_symbol(*var_sym, SymbolKind::Variable, span, None);
                        }
                    }
                }
            } else {
                self.error(
                    &span,
                    &format!(
                        "Unknown module '{}'. If this is a user module, use '.hi' extension.",
                        path
                    ),
                );
            }
        }
    }

    fn resolve_identifier(&mut self, name: HiSymbol, use_span: Span) {
        if self.scope.lookup(name).is_some() {
            self.result.uses.push((use_span, name));
        } else {
            self.error(
                &use_span,
                &format!("Undefined variable or function '{}'", name),
            );
        }
    }

    fn define_symbol(&mut self, name: HiSymbol, kind: SymbolKind, span: Span, doc: Option<String>) {
        self.scope.define(Symbol {
            name,
            kind: kind.clone(),
            span,
            defined_at: Some(span),
            doc: doc.clone(),
        });
        let idx = self.result.symbols.len();
        self.result.symbols.push(SymbolInfo {
            name,
            kind,
            span,
            defined_at: Some(span),
            doc,
        });
        self.result.scopes[self.current_scope].symbols.push(idx);
    }

    fn error(&mut self, span: &Span, msg: &str) {
        self.result.errors.push(AnalysisError {
            message: msg.to_string(),
            span: *span,
        });
    }
}

// -----------------------------------------------------------------------------
// Unit tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use hi_common::intern;
    use hi_interpreter::parser::Parser;
    use hi_interpreter::parser::lexer::Lexer;

    fn analyze_program(code: &str) -> AnalysisResult {
        let tokens = Lexer::tokenize(code).expect("tokenization failed");
        let mut parser = Parser::new(&tokens);
        let program = parser.parse().expect("parsing failed");
        let analyzer = Analyzer::new();
        analyzer.analyze(&program, None)
    }

    fn find_symbol<'a>(result: &'a AnalysisResult, name: &str) -> Option<&'a SymbolInfo> {
        let sym = intern(name);
        result.symbols.iter().find(|s| s.name == sym)
    }

    #[test]
    fn test_variable_definition_and_use() {
        let code = "LET x = 10\nPRINT x";
        let result = analyze_program(code);
        let sym = find_symbol(&result, "x");
        assert!(sym.is_some());
        assert!(matches!(sym.unwrap().kind, SymbolKind::Variable));
        assert_eq!(result.uses.len(), 1);
        assert_eq!(result.uses[0].1, intern("x"));
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_function_definition() {
        let code = "FUNC add(a, b) RET a + b END";
        let result = analyze_program(code);
        let sym = find_symbol(&result, "add");
        assert!(sym.is_some());
        if let SymbolKind::Function(params) = &sym.unwrap().kind {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], intern("a"));
            assert_eq!(params[1], intern("b"));
        } else {
            panic!("Expected Function kind");
        }
        assert!(find_symbol(&result, "a").is_some());
        assert!(find_symbol(&result, "b").is_some());
        let uses: Vec<_> = result.uses.iter().map(|(_, s)| s).collect();
        assert!(uses.contains(&&intern("a")));
        assert!(uses.contains(&&intern("b")));
    }

    #[test]
    fn test_if_scope() {
        let code = "IF x > 0 THEN LET y = 1 END";
        let result = analyze_program(code);
        assert!(!result.errors.is_empty());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("Undefined variable or function 'x'"))
        );
        // y is defined in the current scope because the analyzer does not create a new scope for IF blocks
        assert!(find_symbol(&result, "y").is_some());
    }

    #[test]
    fn test_builtin_function() {
        let code = "hello()";
        let result = analyze_program(code);
        // Built-in functions are not added to result.symbols, only to the global scope.
        // Check that no error occurred and the use is recorded.
        assert!(result.errors.is_empty());
        assert!(result.uses.iter().any(|(_, s)| *s == intern("hello")));
    }

    #[test]
    fn test_module_import_and_call() {
        let code = "IMPORT \"math\"\nmath:sin(1.0)";
        let result = analyze_program(code);
        assert!(result.imported_modules.contains(&intern("math")));
        assert_eq!(result.module_calls.len(), 1);
        let (_, module, func) = &result.module_calls[0];
        assert_eq!(module, "math");
        assert_eq!(func, "sin");
        let sym = find_symbol(&result, "math");
        assert!(sym.is_some());
        assert!(matches!(sym.unwrap().kind, SymbolKind::Module));
        // sin is a built-in function from the math module, not added to result.symbols
        // but the call is recorded.
        assert!(
            result
                .module_calls
                .iter()
                .any(|(_, m, f)| m == "math" && f == "sin")
        );
    }

    #[test]
    fn test_module_alias() {
        let code = "IMPORT \"math\" AS m\nm:cos(0)";
        let result = analyze_program(code);
        assert!(result.imported_modules.contains(&intern("math")));
        assert_eq!(
            result.module_aliases.get(&intern("m")),
            Some(&intern("math"))
        );
        assert_eq!(result.module_calls.len(), 1);
        let (_, module, func) = &result.module_calls[0];
        assert_eq!(module, "math");
        assert_eq!(func, "cos");
        let sym = find_symbol(&result, "m");
        assert!(sym.is_some());
        assert!(matches!(sym.unwrap().kind, SymbolKind::Module));
    }

    #[test]
    fn test_module_variable_access() {
        let code = "IMPORT \"math\"\nmath:PI";
        let result = analyze_program(code);
        assert!(result.imported_modules.contains(&intern("math")));
        assert_eq!(result.module_accesses.len(), 1);
        let (_, module, var) = &result.module_accesses[0];
        assert_eq!(module, "math");
        assert_eq!(var, "PI");
        // PI is a built-in variable from the math module, not added to result.symbols
        // but the access is recorded.
        assert!(
            result
                .module_accesses
                .iter()
                .any(|(_, m, v)| m == "math" && v == "PI")
        );
    }

    #[test]
    fn test_error_undefined_variable() {
        let code = "PRINT z";
        let result = analyze_program(code);
        assert!(!result.errors.is_empty());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("Undefined variable or function 'z'"))
        );
        assert_eq!(result.uses.len(), 0);
    }

    #[test]
    fn test_nested_scopes() {
        let code = "FUNC outer()\n  LET x = 1\n  FUNC inner()\n    PRINT x\n  END\nEND";
        let result = analyze_program(code);
        let uses_x: Vec<_> = result
            .uses
            .iter()
            .filter(|(_, s)| *s == intern("x"))
            .collect();
        assert_eq!(uses_x.len(), 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_for_scope() {
        let code = "FOR i = 0 TO 10 DO PRINT i NEXT";
        let result = analyze_program(code);
        let sym = find_symbol(&result, "i");
        assert!(sym.is_some());
        assert!(matches!(sym.unwrap().kind, SymbolKind::Variable));
        // use of i in PRINT
        assert!(result.uses.iter().any(|(_, s)| *s == intern("i")));
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_import_error() {
        let code = "IMPORT \"nonexistent.hi\"";
        let result = analyze_program(code);
        // Should produce an error because the file cannot be read (we are not providing a file system)
        assert!(!result.errors.is_empty());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("Cannot read module"))
        );
    }

    #[test]
    fn test_import_builtin_with_extension_error() {
        let code = "IMPORT \"math.hi\"";
        let result = analyze_program(code);
        assert!(!result.errors.is_empty());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("Cannot import built-in module"))
        );
    }

    #[test]
    fn test_import_unknown_builtin() {
        let code = "IMPORT \"unknown\"";
        let result = analyze_program(code);
        assert!(!result.errors.is_empty());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("Unknown module"))
        );
    }

    #[test]
    fn test_parameter_use_in_module_call() {
        let code = r#"
        FUNC test(param)
            LET x = os:exists(param)
        END
        "#;
        let result = analyze_program(code);
        let uses_param: Vec<_> = result
            .uses
            .iter()
            .filter(|(_, s)| *s == intern("param"))
            .collect();
        assert_eq!(uses_param.len(), 1, "Expected one use of 'param'");
    }
}
