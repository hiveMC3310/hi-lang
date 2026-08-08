//! Language Server Protocol implementation for Hi.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use hi_analyzer::analysis::{AnalysisResult, Analyzer, SymbolInfo};
use hi_analyzer::symbol;
use hi_interpreter::ast::{Program, Span};
use hi_interpreter::builtins::{self, KEYWORDS};
use hi_interpreter::parser::Parser;
use hi_interpreter::parser::lexer::Lexer;

struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Uri, String>>>,
    analysis_cache: Arc<RwLock<HashMap<Uri, AnalysisResult>>>,
    analyzer: Arc<Analyzer>,
}

/// Конвертирует наш Span в LSP Range.
fn span_to_range(span: &Span) -> Range {
    Range {
        start: Position {
            line: span.start_line.saturating_sub(1) as u32,
            character: span.start_col.saturating_sub(1) as u32,
        },
        end: Position {
            line: span.end_line.saturating_sub(1) as u32,
            character: span.end_col.saturating_sub(1) as u32,
        },
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Hi language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.documents
            .write()
            .expect("RwLock poisoned")
            .insert(uri.clone(), text.clone());

        let (diagnostics, analysis, ok) = self.analyze_document(&text);
        if ok {
            self.analysis_cache
                .write()
                .expect("RwLock poisoned")
                .insert(uri.clone(), analysis);
        } else {
            // Если анализ не удался, возможно, уже есть кэш – не трогаем
            // Или можно удалить кэш, чтобы не было ложных данных
            // Но лучше оставить старый кэш, если он есть.
        }
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();
            self.documents
                .write()
                .expect("RwLock poisoned")
                .insert(uri.clone(), text.clone());

            let (diagnostics, analysis, ok) = self.analyze_document(&text);
            if ok {
                self.analysis_cache
                    .write()
                    .expect("RwLock poisoned")
                    .insert(uri.clone(), analysis);
            }
            // Если не ok, кэш не обновляем – остаётся старый валидный результат
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents
            .write()
            .expect("RwLock poisoned")
            .remove(&uri);
        self.analysis_cache
            .write()
            .expect("RwLock poisoned")
            .remove(&uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let line = (position.line + 1) as usize;
        let col = (position.character + 1) as usize;

        let analysis = {
            let cache = self.analysis_cache.read().expect("RwLock poisoned");
            cache.get(&uri).cloned()
        };

        if let Some(result) = analysis {
            // 1. Проверка вызова функции модуля (module:func)
            if let Some((span, module_str, func_str)) =
                result.module_calls.iter().find(|(sp, _, _)| {
                    sp.start_line <= line
                        && line <= sp.end_line
                        && sp.start_col <= col
                        && col <= sp.end_col
                })
            {
                let module_sym = hi_common::intern(module_str);
                let func_sym = hi_common::intern(func_str);
                let module_funcs = builtins::get_module_functions_map();
                if let Some(funcs) = module_funcs.get(&module_sym) {
                    if let Some(mf) = funcs.iter().find(|f| f.name == func_sym) {
                        let params_str = if mf.params.is_empty() {
                            "".to_string()
                        } else if mf.params.len() == 1 && hi_common::resolve(mf.params[0]) == "..."
                        {
                            "...".to_string()
                        } else {
                            mf.params
                                .iter()
                                .map(|s| s.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        let mut content = format!(
                            "```hi\n{}:{}({})\n```\n\n*function*",
                            module_str, func_str, params_str
                        );
                        if !mf.doc.is_empty() {
                            content.push_str("\n\n---\n");
                            content.push_str(mf.doc);
                        }
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: content,
                            }),
                            range: Some(span_to_range(span)),
                        }));
                    }
                }
            }

            // 1.5 Проверка доступа к переменной модуля (module:var)
            if let Some((span, module_str, var_str)) =
                result.module_accesses.iter().find(|(sp, _, _)| {
                    sp.start_line <= line
                        && line <= sp.end_line
                        && sp.start_col <= col
                        && col <= sp.end_col
                })
            {
                let module_sym = hi_common::intern(module_str);
                let var_sym = hi_common::intern(var_str);
                let module_vars = builtins::get_module_variables_map();
                if let Some(vars) = module_vars.get(&module_sym) {
                    if vars.contains(&var_sym) {
                        let content =
                            format!("```hi\n{}:{}\n```\n\n*variable*", module_str, var_str);
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: content,
                            }),
                            range: Some(span_to_range(span)),
                        }));
                    }
                }
            }

            // 2. Поиск символа по определению (попадание в сам идентификатор)
            if let Some(sym) = result.symbol_at(line, col) {
                let (kind_str, params_opt) = match &sym.kind {
                    symbol::SymbolKind::Variable => ("variable", None),
                    symbol::SymbolKind::Function(params) => ("function", Some(params)),
                    symbol::SymbolKind::BuiltinFunction(params) => {
                        ("builtin function", Some(params))
                    }
                    symbol::SymbolKind::Module => ("module", None),
                    symbol::SymbolKind::Builtin => ("builtin", None),
                };

                let header = if let Some(params) = params_opt {
                    let params_str = if params.len() == 1 && hi_common::resolve(params[0]) == "..."
                    {
                        "...".to_string()
                    } else {
                        params
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    format!("```hi\n{}({})\n```", sym.name, params_str)
                } else {
                    format!("```hi\n{}\n```", sym.name)
                };
                let mut content = format!("{}\n\n*{}*", header, kind_str);
                if let Some(doc) = &sym.doc {
                    content.push_str("\n\n---\n");
                    content.push_str(doc);
                }
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: content,
                    }),
                    range: Some(span_to_range(&sym.span)),
                }));
            }

            // 3. Поиск использования (имя уже разрешено, ищем символ по имени)
            if let Some((use_span, name)) = result.use_at(line, col) {
                let sym = result
                    .symbols
                    .iter()
                    .find(|s| s.name == name)
                    .or_else(|| self.analyzer.global_symbols.iter().find(|s| s.name == name));
                if let Some(sym) = sym {
                    let (kind_str, params_opt) = match &sym.kind {
                        symbol::SymbolKind::Variable => ("variable", None),
                        symbol::SymbolKind::Function(params) => ("function", Some(params)),
                        symbol::SymbolKind::BuiltinFunction(params) => {
                            ("builtin function", Some(params))
                        }
                        symbol::SymbolKind::Module => ("module", None),
                        symbol::SymbolKind::Builtin => ("builtin", None),
                    };

                    let header = if let Some(params) = params_opt {
                        let params_str =
                            if params.len() == 1 && hi_common::resolve(params[0]) == "..." {
                                "...".to_string()
                            } else {
                                params
                                    .iter()
                                    .map(|s| s.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            };
                        format!("```hi\n{}({})\n```", sym.name, params_str)
                    } else {
                        format!("```hi\n{}\n```", sym.name)
                    };
                    let mut content = format!("{}\n\n*{}*", header, kind_str);
                    if let Some(doc) = &sym.doc {
                        content.push_str("\n\n---\n");
                        content.push_str(doc);
                    }
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: Some(span_to_range(&sym.span)),
                    }));
                }
            }
        }
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let line = position.line as usize;
        let col = position.character as usize;

        let trigger_character = params.context.and_then(|ctx| ctx.trigger_character);

        // --- БЕРЁМ АНАЛИЗ ТОЛЬКО ИЗ КЭША (он всегда валидный) ---
        let analysis = {
            let cache = self.analysis_cache.read().expect("RwLock poisoned");
            if let Some(cached) = cache.get(&uri).cloned() {
                cached
            } else {
                return Ok(Some(CompletionResponse::Array(vec![])));
            }
        };

        eprintln!(
            "[completion] analysis.module_aliases = {:?}",
            analysis.module_aliases
        );

        // Контекстное автодополнение после ':'
        if let Some(ref ch) = trigger_character {
            if ch == ":" {
                let documents = self.documents.read().expect("RwLock poisoned");
                if let Some(text) = documents.get(&uri) {
                    let line_text = text.lines().nth(line).unwrap_or("");
                    let before_cursor = &line_text[..col.min(line_text.len())];
                    if let Some(pos) = before_cursor.rfind(':') {
                        let ident_candidate = before_cursor[..pos].trim_end();
                        if !ident_candidate.is_empty() {
                            let module_name =
                                ident_candidate.split_whitespace().last().unwrap_or("");

                            eprintln!(
                                "[completion] detected module_name before ':' = '{}'",
                                module_name
                            );

                            if !module_name.is_empty() {
                                let module_sym = hi_common::intern(module_name);

                                let real_module_sym = analysis
                                    .module_aliases
                                    .get(&module_sym)
                                    .copied()
                                    .unwrap_or(module_sym);

                                eprintln!(
                                    "[completion] module_sym='{}', real_module_sym='{}'",
                                    hi_common::resolve(module_sym),
                                    hi_common::resolve(real_module_sym)
                                );

                                let is_imported =
                                    analysis.imported_modules.contains(&real_module_sym);
                                eprintln!("[completion] is_imported = {}", is_imported);

                                if !is_imported {
                                    return Ok(Some(CompletionResponse::Array(vec![])));
                                }

                                let module_funcs = builtins::get_module_functions_map();
                                if let Some(funcs) = module_funcs.get(&real_module_sym) {
                                    let mut items = Vec::new();
                                    for mf in funcs {
                                        let detail =
                                            symbol::SymbolKind::BuiltinFunction(mf.params.clone())
                                                .signature();
                                        items.push(CompletionItem {
                                            label: mf.name.to_string(),
                                            kind: Some(CompletionItemKind::FUNCTION),
                                            detail: Some(detail),
                                            ..Default::default()
                                        });
                                    }
                                    let module_vars = builtins::get_module_variables_map();
                                    if let Some(vars) = module_vars.get(&real_module_sym) {
                                        for var in vars {
                                            items.push(CompletionItem {
                                                label: var.to_string(),
                                                kind: Some(CompletionItemKind::VARIABLE),
                                                detail: Some("variable".to_string()),
                                                ..Default::default()
                                            });
                                        }
                                    }
                                    return Ok(Some(CompletionResponse::Array(items)));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Обычное завершение (ключевые слова + символы из анализа)
        let mut items = Vec::new();
        for kw in KEYWORDS {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some(kw.to_string()),
                ..Default::default()
            });
        }

        let line = position.line as usize + 1;
        let col = position.character as usize + 1;
        let scope_idx = analysis
            .scope_at(line, col)
            .map(|scope| {
                analysis
                    .scopes
                    .iter()
                    .position(|s| std::ptr::eq(s, scope))
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        let visible_indices = analysis.visible_symbol_indices(scope_idx);
        let mut global_syms: Vec<&SymbolInfo> = visible_indices
            .iter()
            .map(|&idx| &analysis.symbols[idx])
            .collect();

        let mut seen = std::collections::HashSet::new();
        for sym in &global_syms {
            seen.insert(sym.name);
        }
        for sym in &self.analyzer.global_symbols {
            if seen.insert(sym.name) {
                global_syms.push(sym);
            }
        }

        for sym in global_syms {
            let kind = match sym.kind {
                symbol::SymbolKind::Variable => CompletionItemKind::VARIABLE,
                symbol::SymbolKind::Function(_) => CompletionItemKind::FUNCTION,
                symbol::SymbolKind::BuiltinFunction(_) => CompletionItemKind::FUNCTION,
                symbol::SymbolKind::Module => CompletionItemKind::MODULE,
                symbol::SymbolKind::Builtin => CompletionItemKind::FUNCTION,
            };
            let detail = if sym.kind == symbol::SymbolKind::Variable {
                "variable".to_string()
            } else {
                sym.kind.signature()
            };
            items.push(CompletionItem {
                label: sym.name.to_string(),
                kind: Some(kind),
                detail: Some(detail),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let line = (position.line + 1) as usize;
        let col = (position.character + 1) as usize;

        let analysis = {
            let cache = self.analysis_cache.read().expect("RwLock poisoned");
            cache.get(&uri).cloned()
        };

        if let Some(result) = analysis {
            if let Some(def_span) = result.definition_at(line, col) {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: span_to_range(&def_span),
                })));
            }
        }
        Ok(None)
    }
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
            analyzer: Arc::new(Analyzer::new()),
        }
    }

    /// Анализирует документ и возвращает (диагностика, результат анализа).
    fn analyze_document(&self, text: &str) -> (Vec<Diagnostic>, AnalysisResult, bool) {
        let mut diagnostics = Vec::new();

        // Лексер
        let tokens = match Lexer::tokenize(text) {
            Ok(t) => t,
            Err(e) => {
                let span = e.span().unwrap_or_else(|| Span {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                });
                diagnostics.push(Diagnostic {
                    range: span_to_range(&span),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: e.message,
                    ..Default::default()
                });
                return (diagnostics, AnalysisResult::default(), false);
            }
        };

        // Парсер
        let mut parser = Parser::new(&tokens);
        let program = match parser.parse() {
            Ok(p) => p,
            Err(e) => {
                let span = e.span().unwrap_or_else(|| Span {
                    start_line: 0,
                    start_col: 0,
                    end_line: 0,
                    end_col: 0,
                });
                diagnostics.push(Diagnostic {
                    range: span_to_range(&span),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: e.message,
                    ..Default::default()
                });
                // Возвращаем пустой результат, но с флагом false
                return (diagnostics, AnalysisResult::default(), false);
            }
        };

        // Семантический анализ (всегда выполняется, даже если есть ошибки внутри)
        let result = self.analyzer.analyze(&program);

        eprintln!(
            "[lsp] analyze_document: imported_modules = {:?}",
            result
                .imported_modules
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
        eprintln!(
            "[lsp] analyze_document: module_aliases = {:?}",
            result
                .module_aliases
                .iter()
                .map(|(k, v)| format!("{} -> {}", k, v))
                .collect::<Vec<_>>()
        );

        for error in &result.errors {
            diagnostics.push(Diagnostic {
                range: span_to_range(&error.span),
                severity: Some(DiagnosticSeverity::ERROR),
                message: error.message.clone(),
                ..Default::default()
            });
        }

        // Успешный анализ (даже если есть семантические ошибки)
        (diagnostics, result, true)
    }
}

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::build(|client| Backend::new(client)).finish();
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
