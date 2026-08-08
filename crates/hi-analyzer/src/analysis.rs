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
            Func(name, params, body, doc, span) => {
                self.define_symbol(
                    *name,
                    SymbolKind::Function(params.clone()),
                    *span,
                    doc.clone(),
                );
                let new_scope_idx = self.result.scopes.len();
                self.result.scopes.push(ScopeInfo {
                    parent: Some(self.current_scope),
                    span: *span,
                    symbols: Vec::new(),
                });
                let old_scope_idx = self.current_scope;
                self.current_scope = new_scope_idx;

                let child = self.scope.child();
                let old_scope_env = std::mem::replace(self.scope, child);
                self.depth += 1;

                for p in params {
                    self.define_symbol(*p, SymbolKind::Variable, *span, None);
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
            For(var, start, end, step, body, span) => {
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
            Call(name, args, span) => {
                self.resolve_identifier(*name, *span);
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
            ModuleAccess(module, var, span) => {
                let real_module = *self.module_aliases.get(module).unwrap_or(module);
                if self.scope.lookup(*module).is_none() {
                    self.error(span, &format!("Undefined module '{}'", module));
                } else {
                    self.result.module_accesses.push((
                        *span,
                        hi_common::resolve(real_module),
                        hi_common::resolve(*var),
                    ));
                }
            }
            CallModule(module, func, args, span) => {
                let real_module = *self.module_aliases.get(module).unwrap_or(module);
                if self.scope.lookup(*module).is_none() {
                    self.error(span, &format!("Undefined module '{}'", module));
                } else {
                    self.result.module_calls.push((
                        *span,
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
        let module_name = path.trim_end_matches(".hi");
        let module_sym = hi_common::intern(module_name);

        if self.builtin_module_names.contains(&module_sym) {
            if self.scope.lookup(module_sym).is_none() {
                self.define_symbol(module_sym, SymbolKind::Module, span, None);
            }
            self.result.imported_modules.insert(module_sym);

            if let Some(alias_sym) = alias {
                self.module_aliases.insert(alias_sym, module_sym);
                self.define_symbol(alias_sym, SymbolKind::Module, span, None);
                self.result.module_aliases.insert(alias_sym, module_sym);
            } else {
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
            return;
        }
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
