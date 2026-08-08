//! Semantic analysis: builds symbol table and resolves references.

use crate::scope::Scope;
use crate::symbol::{Symbol, SymbolKind};
use hi_common::Symbol as HiSymbol;
use hi_interpreter::ast::{Expr, Program, Span, Stmt};
use hi_interpreter::builtins;
use std::collections::{HashMap, HashSet};

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
    pub parent: Option<usize>, // индекс в AnalysisResult.scopes
    pub span: Span,            // диапазон всего блока, в котором действует скоуп
    pub symbols: Vec<usize>,   // индексы в AnalysisResult.symbols
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub symbols: Vec<SymbolInfo>,
    pub errors: Vec<AnalysisError>,
    pub uses: Vec<(Span, HiSymbol)>,
    pub module_calls: Vec<(Span, String, String)>,
    pub imported_modules: HashSet<HiSymbol>,
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

    /// Находит самый глубокий скоуп, содержащий заданную позицию.
    pub fn scope_at(&self, line: usize, col: usize) -> Option<&ScopeInfo> {
        self.scopes.iter().rev().find(|scope| {
            scope.span.start_line <= line
                && line <= scope.span.end_line
                && scope.span.start_col <= col
                && col <= scope.span.end_col
        })
    }

    /// Собирает индексы видимых символов из указанного скоупа и всех его родителей.
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
// Analyzer (глобальный, иммутабельный)
// -----------------------------------------------------------------------------

pub struct Analyzer {
    pub global_symbols: Vec<SymbolInfo>,
    global_scope: Scope,
    builtin_module_names: Vec<HiSymbol>,
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
        }
    }

    pub fn analyze(&self, program: &Program) -> AnalysisResult {
        let mut result = AnalysisResult::default();
        // Корневой скоуп с неограниченным span
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
        };

        for stmt in &program.stmts {
            file_analyzer.analyze_stmt(stmt);
        }

        result.module_aliases = aliases;

        eprintln!(
            "[analyzer] Final module_aliases: {:?}",
            result
                .module_aliases
                .iter()
                .map(|(k, v)| format!("{} -> {}", k, v))
                .collect::<Vec<_>>()
        );

        result
    }
}

// -----------------------------------------------------------------------------
// Временный анализатор одного файла
// -----------------------------------------------------------------------------

struct FileAnalyzer<'a> {
    scope: &'a mut Scope,
    result: &'a mut AnalysisResult,
    module_aliases: &'a mut HashMap<HiSymbol, HiSymbol>,
    builtin_module_names: &'a [HiSymbol],
    depth: usize,
    current_scope: usize,
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
                // Определяем саму функцию в текущем скоупе
                self.define_symbol(
                    *name,
                    SymbolKind::Function(params.clone()),
                    *span,
                    doc.clone(),
                );
                // Создаём дочерний скоуп для тела функции
                let new_scope_idx = self.result.scopes.len();
                self.result.scopes.push(ScopeInfo {
                    parent: Some(self.current_scope),
                    span: *span,
                    symbols: Vec::new(),
                });
                let old_scope_idx = self.current_scope;
                self.current_scope = new_scope_idx;

                // Переключаем символьный скоуп
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
                // Переменная цикла видна во внешнем скоупе (как в интерпретаторе)
                self.define_symbol(*var, SymbolKind::Variable, *span, None);
                // Тело цикла не создаёт отдельного скоупа, поэтому depth и current_scope не меняем,
                // но вложенные стейтменты всё равно попадут в текущий скоуп (который может быть глобальным или функциональным)
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
                eprintln!("[analyzer] Import: path={:?}, alias={:?}", path, alias);
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
        let module_name = path.trim_end_matches(".hi").to_string();
        let module_sym = hi_common::intern(&module_name);

        eprintln!(
            "[analyzer] analyze_import: path='{}', module_sym='{}', alias={:?}",
            path,
            hi_common::resolve(module_sym),
            alias.map(|s| s.to_string())
        );
        eprintln!(
            "[analyzer] builtin_module_names: {:?}",
            self.builtin_module_names
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );

        // Проверяем, существует ли модуль
        if !self.builtin_module_names.contains(&module_sym) {
            eprintln!("[analyzer] Module '{}' not found", module_name);
            self.error(&span, &format!("Module '{}' not found", module_name));
            return;
        }

        // Всегда добавляем оригинальное имя модуля в область видимости (если ещё не добавлено)
        if self.scope.lookup(module_sym).is_none() {
            self.define_symbol(module_sym, SymbolKind::Module, span, None);
        }

        if let Some(alias_sym) = alias {
            // Добавляем алиас как отдельный символ, указывающий на модуль
            self.module_aliases.insert(alias_sym, module_sym);
            self.define_symbol(alias_sym, SymbolKind::Module, span, None);
            self.result.module_aliases.insert(alias_sym, module_sym);
            self.result.imported_modules.insert(module_sym);
            eprintln!(
                "[analyzer] Added alias: {} -> {}",
                hi_common::resolve(alias_sym),
                hi_common::resolve(module_sym)
            );
        } else {
            // Без алиаса – инлайним функции и переменные модуля (они уже доступны по имени модуля)
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
            self.result.imported_modules.insert(module_sym);
            eprintln!(
                "[analyzer] Imported module (no alias): {}",
                hi_common::resolve(module_sym)
            );
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
        // Добавляем в текущий скоуп
        self.result.scopes[self.current_scope].symbols.push(idx);
    }

    fn error(&mut self, span: &Span, msg: &str) {
        self.result.errors.push(AnalysisError {
            message: msg.to_string(),
            span: *span,
        });
    }
}
