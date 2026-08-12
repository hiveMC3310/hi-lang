//! Language Server Protocol implementation for Hi.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::sync::Mutex as TokioMutex;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use hi_analyzer::analysis::{AnalysisResult, Analyzer, SymbolInfo};
use hi_analyzer::symbol;
use hi_common::Symbol as HiSymbol;
use hi_interpreter::ast::Span;
use hi_interpreter::builtins::{self, KEYWORDS};
use hi_interpreter::parser::Parser;
use hi_interpreter::parser::lexer::Lexer;

// ---------------------------------------------------------------------------
// Backend
// ---------------------------------------------------------------------------

struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Uri, String>>>,
    analysis_cache: Arc<RwLock<HashMap<Uri, AnalysisResult>>>,
    analyzer: Arc<Analyzer>,
    /// Channel for canceling the previous analysis task.
    cancel_tx: Arc<TokioMutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// LanguageServer implementation
// ---------------------------------------------------------------------------

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
                    trigger_characters: Some(vec![":".to_string(), "\"".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                references_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        let _ = self
            .client
            .log_message(MessageType::INFO, "Hi language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    // ------------------------------------------------------------------
    // Document sync
    // ------------------------------------------------------------------

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        if let Ok(mut docs) = self.documents.write() {
            docs.insert(uri.clone(), text);
        } else {
            let _ = self
                .client
                .log_message(MessageType::ERROR, "Failed to write to documents")
                .await;
            return;
        }
        self.request_analysis(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();
            if let Ok(mut docs) = self.documents.write() {
                docs.insert(uri.clone(), text);
            } else {
                let _ = self
                    .client
                    .log_message(MessageType::ERROR, "Failed to write to documents")
                    .await;
                return;
            }
            self.request_analysis(uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Ok(mut docs) = self.documents.write() {
            docs.remove(&uri);
        }
        if let Ok(mut cache) = self.analysis_cache.write() {
            cache.remove(&uri);
        }
    }

    // ------------------------------------------------------------------
    // Hover
    // ------------------------------------------------------------------

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let line = (position.line + 1) as usize;
        let col = (position.character + 1) as usize;

        let analysis = {
            let cache = match self.analysis_cache.read() {
                Ok(c) => c,
                Err(_) => return Ok(None),
            };
            cache.get(&uri).cloned()
        };

        if let Some(result) = analysis {
            // 1. module:func
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

                // built-in
                let module_funcs = builtins::get_module_functions_map();
                if let Some(funcs) = module_funcs.get(&module_sym)
                    && let Some(mf) = funcs.iter().find(|f| f.name == func_sym) {
                        let params_str = Self::format_params(&mf.params);
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

                // user-defined
                if let Some(symbols) = result.loaded_module_exports.get(&module_sym)
                    && let Some(sym) = symbols.iter().find(|s| s.name == func_sym) {
                        let params_str = Self::format_info_params(&sym.kind);
                        let mut content = format!(
                            "```hi\n{}:{}({})\n```\n\n*function*",
                            module_str, func_str, params_str
                        );
                        if let Some(doc) = &sym.doc {
                            content.push_str("\n\n---\n");
                            content.push_str(doc);
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

            // 1.5 module:var
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

                // built-in
                let module_vars = builtins::get_module_variables_map();
                if let Some(vars) = module_vars.get(&module_sym)
                    && vars.contains(&var_sym) {
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

                // user-defined
                if let Some(symbols) = result.loaded_module_exports.get(&module_sym)
                    && symbols.iter().any(|s| s.name == var_sym) {
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

            // 2. symbol at
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

                let header = if params_opt.is_some() {
                    let params_str = Self::format_info_params(&sym.kind);
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

            // 3. use
            if let Some((_, name)) = result.use_at(line, col) {
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

                    let header = if params_opt.is_some() {
                        let params_str = Self::format_info_params(&sym.kind);
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

    // ------------------------------------------------------------------
    // Completion
    // ------------------------------------------------------------------

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let line = position.line as usize;
        let col = position.character as usize;

        let trigger_character = params.context.and_then(|ctx| ctx.trigger_character);

        let analysis = {
            let cache = match self.analysis_cache.read() {
                Ok(c) => c,
                Err(_) => return Ok(Some(CompletionResponse::Array(vec![]))),
            };
            match cache.get(&uri) {
                Some(cached) => cached.clone(),
                None => return Ok(Some(CompletionResponse::Array(vec![]))),
            }
        };

        // Contextual: after '"' (IMPORT)
        if let Some(ref ch) = trigger_character {
            if ch == "\"" {
                let documents = match self.documents.read() {
                    Ok(d) => d,
                    Err(_) => return Ok(Some(CompletionResponse::Array(vec![]))),
                };
                if let Some(text) = documents.get(&uri) {
                    let line_text = text.lines().nth(line).unwrap_or("");
                    let before_cursor = &line_text[..col.min(line_text.len())];
                    if let Some(pos) = before_cursor.rfind('"') {
                        let before_quote = &before_cursor[..pos];
                        if let Some(import_pos) = before_quote.rfind("IMPORT") {
                            let after_import = &before_quote[import_pos + 6..].trim();
                            let prefix = after_import.trim();
                            let mut items = Vec::new();
                            for module_sym in &self.analyzer.builtin_module_names {
                                let name = hi_common::resolve(*module_sym);
                                if name.starts_with(prefix) {
                                    items.push(CompletionItem {
                                        label: name.clone(),
                                        kind: Some(CompletionItemKind::MODULE),
                                        detail: Some("built-in module".to_string()),
                                        insert_text: Some(name),
                                        ..Default::default()
                                    });
                                }
                            }
                            return Ok(Some(CompletionResponse::Array(items)));
                        }
                    }
                }
            }

            // Contextual: after ':'
            if ch == ":" {
                let documents = match self.documents.read() {
                    Ok(d) => d,
                    Err(_) => return Ok(Some(CompletionResponse::Array(vec![]))),
                };
                if let Some(text) = documents.get(&uri) {
                    let line_text = text.lines().nth(line).unwrap_or("");
                    let before_cursor = &line_text[..col.min(line_text.len())];
                    if let Some(pos) = before_cursor.rfind(':') {
                        let ident_candidate = before_cursor[..pos].trim_end();
                        if !ident_candidate.is_empty() {
                            let module_name =
                                ident_candidate.split_whitespace().last().unwrap_or("");
                            if !module_name.is_empty() {
                                let module_sym = hi_common::intern(module_name);
                                let real_module_sym = analysis
                                    .module_aliases
                                    .get(&module_sym)
                                    .copied()
                                    .unwrap_or(module_sym);

                                if !analysis.imported_modules.contains(&real_module_sym) {
                                    return Ok(Some(CompletionResponse::Array(vec![])));
                                }

                                // built-in modules
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

                                // user-defined modules
                                if let Some(symbols) =
                                    analysis.loaded_module_exports.get(&real_module_sym)
                                {
                                    let mut items = Vec::new();
                                    for sym in symbols.iter() {
                                        let (kind, detail) = match &sym.kind {
                                            symbol::SymbolKind::Function(_)
                                            | symbol::SymbolKind::BuiltinFunction(_) => {
                                                (CompletionItemKind::FUNCTION, sym.kind.signature())
                                            }
                                            symbol::SymbolKind::Variable => (
                                                CompletionItemKind::VARIABLE,
                                                "variable".to_string(),
                                            ),
                                            _ => continue,
                                        };
                                        items.push(CompletionItem {
                                            label: sym.name.to_string(),
                                            kind: Some(kind),
                                            detail: Some(detail),
                                            ..Default::default()
                                        });
                                    }
                                    return Ok(Some(CompletionResponse::Array(items)));
                                }

                                return Ok(Some(CompletionResponse::Array(vec![])));
                            }
                        }
                    }
                }
            }
        }

        // Normal completion
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

    // ------------------------------------------------------------------
    // Goto definition
    // ------------------------------------------------------------------

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let line = (position.line + 1) as usize;
        let col = (position.character + 1) as usize;

        let analysis = {
            let cache = match self.analysis_cache.read() {
                Ok(c) => c,
                Err(_) => return Ok(None),
            };
            cache.get(&uri).cloned()
        };

        if let Some(result) = analysis
            && let Some(def_span) = result.definition_at(line, col) {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri,
                    range: span_to_range(&def_span),
                })));
            }
        Ok(None)
    }

    // ------------------------------------------------------------------
    // Rename
    // ------------------------------------------------------------------

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let position = params.position;
        let line = (position.line + 1) as usize;
        let col = (position.character + 1) as usize;

        let analysis = {
            let cache = match self.analysis_cache.read() {
                Ok(c) => c,
                Err(_) => return Ok(None),
            };
            cache.get(&uri).cloned()
        };

        if let Some(result) = analysis {
            if let Some(sym) = result.symbol_at(line, col) {
                if matches!(sym.kind, symbol::SymbolKind::BuiltinFunction(_)) {
                    return Ok(None);
                }
                return Ok(Some(PrepareRenameResponse::Range(span_to_range(&sym.span))));
            }
            if let Some((use_span, _)) = result.use_at(line, col) {
                return Ok(Some(PrepareRenameResponse::Range(span_to_range(&use_span))));
            }
        }
        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let line = (position.line + 1) as usize;
        let col = (position.character + 1) as usize;
        let new_name = params.new_name;

        let analysis = {
            let cache = match self.analysis_cache.read() {
                Ok(c) => c,
                Err(_) => return Ok(None),
            };
            cache.get(&uri).cloned()
        };

        if let Some(result) = analysis {
            let sym_name = if let Some(sym) = result.symbol_at(line, col) {
                if matches!(sym.kind, symbol::SymbolKind::BuiltinFunction(_)) {
                    return Ok(None);
                }
                sym.name
            } else if let Some((_, name)) = result.use_at(line, col) {
                name
            } else {
                return Ok(None);
            };

            let spans = result.all_uses_of(sym_name);
            if spans.is_empty() {
                return Ok(None);
            }

            let edits: Vec<TextEdit> = spans
                .iter()
                .map(|span| TextEdit {
                    range: span_to_range(span),
                    new_text: new_name.clone(),
                })
                .collect();

            let mut changes = HashMap::new();
            changes.insert(uri, edits);
            Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }))
        } else {
            Ok(None)
        }
    }

    // ------------------------------------------------------------------
    // References
    // ------------------------------------------------------------------

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let line = (position.line + 1) as usize;
        let col = (position.character + 1) as usize;

        let analysis = {
            let cache = match self.analysis_cache.read() {
                Ok(c) => c,
                Err(_) => return Ok(None),
            };
            cache.get(&uri).cloned()
        };

        if let Some(result) = analysis {
            let sym_name = if let Some(sym) = result.symbol_at(line, col) {
                sym.name
            } else if let Some((_, name)) = result.use_at(line, col) {
                name
            } else {
                return Ok(None);
            };

            let spans = result.all_uses_of(sym_name);
            let locations: Vec<Location> = spans
                .iter()
                .map(|span| Location {
                    uri: uri.clone(),
                    range: span_to_range(span),
                })
                .collect();

            Ok(Some(locations))
        } else {
            Ok(None)
        }
    }
}

// ---------------------------------------------------------------------------
// Backend helpers
// ---------------------------------------------------------------------------

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
            analyzer: Arc::new(Analyzer::new()),
            cancel_tx: Arc::new(TokioMutex::new(None)),
        }
    }

    /// Asynchronously runs file analysis with a 150 ms debounce.
    async fn request_analysis(&self, uri: Uri) {
        // Cancel the previous task
        let rx = {
            let mut cancel = self.cancel_tx.lock().await;
            if let Some(tx) = cancel.take() {
                let _ = tx.send(());
            }
            let (tx, rx) = tokio::sync::oneshot::channel();
            *cancel = Some(tx);
            rx
        };

        let documents = self.documents.clone();
        let analysis_cache = self.analysis_cache.clone();
        let analyzer = self.analyzer.clone();
        let client = self.client.clone();
        let uri_clone = uri.clone();

        tokio::spawn(async move {
            // Debounce: wait 150 ms or cancellation
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(150)) => {},
                _ = rx => { return; }
            }

            // Additional URI clone for passing to spawn_blocking
            let uri_for_analysis = uri_clone.clone();

            let result = tokio::task::spawn_blocking(move || {
                let documents = match documents.read() {
                    Ok(d) => d,
                    Err(_) => return (Vec::new(), AnalysisResult::default()),
                };
                let text = match documents.get(&uri_for_analysis) {
                    Some(t) => t.clone(),
                    None => return (Vec::new(), AnalysisResult::default()),
                };
                let mut diagnostics = Vec::new();

                let tokens = match Lexer::tokenize(&text) {
                    Ok(t) => t,
                    Err(e) => {
                        let span = e.span().unwrap_or(Span {
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
                        return (diagnostics, AnalysisResult::default());
                    }
                };

                let mut parser = Parser::new(&tokens);
                let program = match parser.parse() {
                    Ok(p) => p,
                    Err(e) => {
                        let span = e.span().unwrap_or(Span {
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
                        return (diagnostics, AnalysisResult::default());
                    }
                };

                let path = uri_for_analysis.to_file_path();
                let analysis = analyzer.analyze(&program, path.as_deref());

                for error in &analysis.errors {
                    diagnostics.push(Diagnostic {
                        range: span_to_range(&error.span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: error.message.clone(),
                        ..Default::default()
                    });
                }

                (diagnostics, analysis)
            })
            .await;

            let (diagnostics, analysis) = match result {
                Ok(res) => res,
                Err(e) => {
                    let _ = client
                        .log_message(MessageType::ERROR, format!("Analysis task panicked: {}", e))
                        .await;
                    return;
                }
            };

            // Update cache
            if let Ok(mut cache) = analysis_cache.write() {
                cache.insert(uri_clone.clone(), analysis);
            } else {
                let _ = client
                    .log_message(MessageType::ERROR, "Failed to write to analysis cache")
                    .await;
                return;
            }

            client
                .publish_diagnostics(uri_clone, diagnostics, None)
                .await;
        });
    }

    fn format_params(params: &[HiSymbol]) -> String {
        if params.is_empty() {
            String::new()
        } else if params.len() == 1 && hi_common::resolve(params[0]) == "..." {
            "...".to_string()
        } else {
            params
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    fn format_info_params(kind: &symbol::SymbolKind) -> String {
        match kind {
            symbol::SymbolKind::Function(params) | symbol::SymbolKind::BuiltinFunction(params) => {
                Self::format_params(params)
            }
            _ => String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::build(Backend::new).finish();
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
