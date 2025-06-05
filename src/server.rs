use serde_json::Value;
use tower_lsp::{jsonrpc::Result, lsp_types::*, LanguageServer};

use crate::{cmds, position_to_offset, span_contains, span_to_lsp_range, Context};

#[tower_lsp::async_trait]
impl LanguageServer for Context {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(Default::default()),
                definition_provider: Some(OneOf::Left(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["generate-tir".to_string(), "generate-ast".to_string()],
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "tx3-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Return empty completion list for now
        Ok(Some(CompletionResponse::Array(vec![])))
    }

    async fn goto_definition(
        &self,
        _: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // Return None for now, indicating no definition found
        Ok(None)
    }

    async fn references(&self, _: ReferenceParams) -> Result<Option<Vec<Location>>> {
        // Return empty references list for now
        Ok(Some(vec![]))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let document = self.documents.get(uri);
        if let Some(document) = document {
            let text = document.value().to_string();

            let ast = match tx3_lang::parsing::parse_string(text.as_str()) {
                Ok(ast) => ast,
                Err(_) => return Ok(None),
            };

            let offset = position_to_offset(&text, position);

            for party in &ast.parties {
                if span_contains(&party.span, offset) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!(
                                "**Party**: `{}`\n\nA party in the transaction. It can be an address for a script or a wallet.",
                                party.name
                            ),
                        }),
                        range: Some(span_to_lsp_range(document.value(), &party.span)),
                    }));
                }
            }

            for policy in &ast.policies {
                if span_contains(&policy.span, offset) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("**Policy**: `{}`\n\nA policy definition.", policy.name),
                        }),
                        range: Some(span_to_lsp_range(document.value(), &policy.span)),
                    }));
                }
            }

            for tx in &ast.txs {
                for input in &tx.inputs {
                    if span_contains(&input.span, offset) {
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!("**Input**: `{}`\n\nTransaction input.", input.name),
                            }),
                            range: Some(span_to_lsp_range(document.value(), &input.span)),
                        }));
                    }
                }

                for output in &tx.outputs {
                    if span_contains(&output.span, offset) {
                        let default_output = "output".to_string();
                        let name = output.name.as_ref().unwrap_or(&default_output);
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!("**Output**: `{}`\n\nTransaction output.", name),
                            }),
                            range: Some(span_to_lsp_range(document.value(), &output.span)),
                        }));
                    }
                }

                if span_contains(&tx.parameters.span, offset) {
                    for param in &tx.parameters.parameters {
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!(
                                    "**Parameter**: `{}`\n\n**Type**: `{:?}`",
                                    param.name, param.r#type
                                ),
                            }),
                            range: Some(span_to_lsp_range(document.value(), &tx.parameters.span)),
                        }));
                    }
                }

                if span_contains(&tx.span, offset) {
                    let mut hover_text = format!("**Transaction**: `{}`\n\n", tx.name);

                    if !tx.parameters.parameters.is_empty() {
                        hover_text.push_str("**Parameters**:\n");
                        for param in &tx.parameters.parameters {
                            hover_text
                                .push_str(&format!("- `{}`: `{:?}`\n", param.name, param.r#type));
                        }
                        hover_text.push_str("\n");
                    }

                    if !tx.inputs.is_empty() {
                        hover_text.push_str("**Inputs**:\n");
                        for input in &tx.inputs {
                            hover_text.push_str(&format!("- `{}`\n", input.name));
                        }
                        hover_text.push_str("\n");
                    }

                    if !tx.outputs.is_empty() {
                        hover_text.push_str("**Outputs**:\n");
                        for output in &tx.outputs {
                            let default_output = "output".to_string();
                            let name = output.name.as_ref().unwrap_or(&default_output);
                            hover_text.push_str(&format!("- `{}`\n", name));
                        }
                    }

                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: Some(span_to_lsp_range(document.value(), &tx.span)),
                    }));
                }
            }
        }

        Ok(None)
    }

    // TODO: Add error handling and improve
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        fn make_symbol(
            name: String,
            detail: String,
            kind: SymbolKind,
            range: Range,
            children: Option<Vec<DocumentSymbol>>,
        ) -> DocumentSymbol {
            #[allow(deprecated)]
            DocumentSymbol {
                name,
                detail: Some(detail),
                kind,
                range: range,
                selection_range: range,
                children: children,
                tags: Default::default(),
                deprecated: Default::default(),
            }
        }

        let mut symbols: Vec<DocumentSymbol> = Vec::new();
        let uri = &params.text_document.uri;
        let document = self.documents.get(uri);
        if let Some(document) = document {
            let text = document.value().to_string();
            let ast = tx3_lang::parsing::parse_string(text.as_str());
            if ast.is_ok() {
                let ast = ast.unwrap();
                for party in ast.parties {
                    symbols.push(make_symbol(
                        party.name.clone(),
                        "Party".to_string(),
                        SymbolKind::OBJECT,
                        span_to_lsp_range(document.value(), &party.span),
                        None,
                    ));
                }

                for policy in ast.policies {
                    symbols.push(make_symbol(
                        policy.name.clone(),
                        "Policy".to_string(),
                        SymbolKind::KEY,
                        span_to_lsp_range(document.value(), &policy.span),
                        None,
                    ));
                }

                for tx in ast.txs {
                    let mut children: Vec<DocumentSymbol> = Vec::new();
                    for parameter in tx.parameters.parameters {
                        children.push(make_symbol(
                            parameter.name.clone(),
                            format!("Parameter<{:?}>", parameter.r#type),
                            SymbolKind::FIELD,
                            span_to_lsp_range(document.value(), &tx.parameters.span),
                            None,
                        ));
                    }

                    for input in tx.inputs {
                        children.push(make_symbol(
                            input.name.clone(),
                            "Input".to_string(),
                            SymbolKind::OBJECT,
                            span_to_lsp_range(document.value(), &input.span),
                            None,
                        ));
                    }

                    for output in tx.outputs {
                        children.push(make_symbol(
                            output.name.unwrap_or_else(|| { "output" }.to_string()),
                            "Output".to_string(),
                            SymbolKind::OBJECT,
                            span_to_lsp_range(document.value(), &output.span),
                            None,
                        ));
                    }

                    symbols.push(make_symbol(
                        tx.name.clone(),
                        "Tx".to_string(),
                        SymbolKind::METHOD,
                        span_to_lsp_range(document.value(), &tx.span),
                        Some(children),
                    ));
                }
            }
        }
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn symbol(&self, _: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        // Return empty workspace symbols list for now
        Ok(Some(vec![]))
    }

    async fn symbol_resolve(&self, params: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        dbg!(&params);
        Ok(params)
    }

    // TODO: not sure if using execute_command is a good idea, but it's the simplest way to return a value to the client without going outside of the lsp protocol
    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        match cmds::handle_command(self, params).await {
            Ok(x) => Ok(x),
            Err(e) => {
                dbg!(&e);
                Err(e.into())
            }
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        let text = params.text_document.text.as_str();

        let diagnostics = self.process_document(uri.clone(), text).await;

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        let text = params
            .content_changes
            .first()
            .map(|x| x.text.as_str())
            .unwrap_or("");

        let diagnostics = self.process_document(uri.clone(), text).await;

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }
}
