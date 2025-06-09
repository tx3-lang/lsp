use std::str::FromStr as _;

use dashmap::DashMap;
use ropey::Rope;
use thiserror::Error;
use tower_lsp::jsonrpc::ErrorCode;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tx3_lang::Protocol;

mod cmds;
mod server;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid command args: {0}")]
    InvalidCommandArgs(String),

    #[error("Url Parse error: {0}")]
    ParseError(#[from] url::ParseError),

    #[error("Document not found: {0}")]
    DocumentNotFound(Url),

    #[error("Protocol loading error: {0}")]
    ProtocolLoadingError(#[from] tx3_lang::loading::Error),

    #[error("Tx3 Lowering error: {0}")]
    TxLoweringError(#[from] tx3_lang::lowering::Error),
}

impl From<&Error> for ErrorCode {
    fn from(err: &Error) -> Self {
        match err {
            Error::InvalidCommand(_) => ErrorCode::InvalidRequest,
            Error::ParseError(_) => ErrorCode::InvalidParams,
            Error::DocumentNotFound(_) => ErrorCode::InvalidParams,
            Error::InvalidCommandArgs(_) => ErrorCode::InvalidParams,
            Error::ProtocolLoadingError(_) => ErrorCode::InvalidRequest,
            Error::TxLoweringError(_) => ErrorCode::InvalidRequest,
        }
    }
}

impl From<Error> for tower_lsp::jsonrpc::Error {
    fn from(err: Error) -> Self {
        tower_lsp::jsonrpc::Error {
            code: From::from(&err),
            message: err.to_string().into(),
            data: None,
        }
    }
}

pub fn char_index_to_line_col(rope: &Rope, idx: usize) -> (usize, usize) {
    let line = rope.char_to_line(idx);
    let line_start = rope.line_to_char(line);
    let col = idx - line_start;
    (line, col)
}

pub fn position_to_offset(text: &str, position: Position) -> usize {
    let mut offset = 0;
    for (line_num, line) in text.lines().enumerate() {
        if line_num == position.line as usize {
            offset += position.character.min(line.len() as u32) as usize;
            break;
        }
        offset += line.len() + 1;
    }
    offset
}

// TODO: Find the smallest span at the offset in the AST
pub fn get_identifier_at_position(text: &str, offset: usize) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();

    if offset >= chars.len() {
        return None;
    }

    let mut start = offset;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    let mut end = offset;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

pub fn span_contains(span: &tx3_lang::ast::Span, offset: usize) -> bool {
    offset >= span.start && offset < span.end
}

pub fn span_to_lsp_range(rope: &Rope, loc: &tx3_lang::ast::Span) -> Range {
    let (start_line, start_col) = char_index_to_line_col(rope, loc.start);
    let (end_line, end_col) = char_index_to_line_col(rope, loc.end);
    let start = Position::new(start_line as u32, start_col as u32);
    let end = Position::new(end_line as u32, end_col as u32);
    Range::new(start, end)
}

fn parse_error_to_diagnostic(rope: &Rope, err: &tx3_lang::parsing::Error) -> Diagnostic {
    let range = span_to_lsp_range(rope, &err.span);
    let message = err.message.clone();
    let source = err.src.clone();

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(source),
        message,
        ..Default::default()
    }
}

fn analyze_error_to_diagnostic(rope: &Rope, err: &tx3_lang::analyzing::Error) -> Diagnostic {
    let range = span_to_lsp_range(rope, err.span());
    let message = err.to_string();
    let source = err.src().unwrap_or("tx3").to_string();

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(source),
        message,
        ..Default::default()
    }
}

fn analyze_report_to_diagnostic(
    rope: &Rope,
    report: &tx3_lang::analyzing::AnalyzeReport,
) -> Vec<Diagnostic> {
    report
        .errors
        .iter()
        .map(|err| analyze_error_to_diagnostic(rope, err))
        .collect()
}

#[derive(Debug)]
pub struct Context {
    pub client: Client,
    pub documents: DashMap<Url, Rope>,
    //asts: DashMap<Url, tx3_lang::ast::Program>,
}

impl Context {
    fn collect_semantic_tokens(
        &self,
        ast: &tx3_lang::ast::Program,
        rope: &Rope,
    ) -> Vec<SemanticToken> {
        // Token type indices based on the legend order
        const TOKEN_KEYWORD: u32 = 0;
        const TOKEN_TYPE: u32 = 1;
        const TOKEN_PARAMETER: u32 = 2;
        const TOKEN_VARIABLE: u32 = 3;
        const TOKEN_FUNCTION: u32 = 4;
        const TOKEN_CLASS: u32 = 5;
        const TOKEN_PROPERTY: u32 = 6;
        const TOKEN_PARTY: u32 = 7;
        const TOKEN_POLICY: u32 = 8;
        const TOKEN_TRANSACTION: u32 = 9;
        const TOKEN_INPUT: u32 = 10;
        const TOKEN_OUTPUT: u32 = 11;
        const TOKEN_REFERENCE: u32 = 12;

        // Token modifiers
        const MOD_DECLARATION: u32 = 1 << 0;
        const MOD_DEFINITION: u32 = 1 << 1;

        #[derive(Debug, Clone)]
        struct TokenInfo {
            range: Range,
            token_type: u32,
            token_modifiers: u32,
        }

        let mut token_infos: Vec<TokenInfo> = Vec::new();

        // TODO: Use the span function to get the identifier ranges
        let extract_identifier_after_keyword =
            |span: &tx3_lang::ast::Span, keyword: &str, identifier: &str| -> Option<Range> {
                let start_char = span.start;
                let end_char = span.end;

                if start_char >= end_char {
                    return None;
                }

                let text_slice = rope.slice(start_char..end_char);
                let text = text_slice.to_string();

                // Find keyword first
                if let Some(keyword_pos) = text.find(keyword) {
                    // Look for the identifier after the keyword
                    let after_keyword = &text[keyword_pos + keyword.len()..];
                    if let Some(id_pos) = after_keyword.find(identifier) {
                        let absolute_id_start = start_char + keyword_pos + keyword.len() + id_pos;
                        let absolute_id_end = absolute_id_start + identifier.len();

                        return Some(span_to_lsp_range(
                            rope,
                            &tx3_lang::ast::Span::new(absolute_id_start, absolute_id_end),
                        ));
                    }
                }

                // If we can't find keyword, just try to find the identifier
                if let Some(id_pos) = text.find(identifier) {
                    let absolute_id_start = start_char + id_pos;
                    let absolute_id_end = absolute_id_start + identifier.len();

                    return Some(span_to_lsp_range(
                        rope,
                        &tx3_lang::ast::Span::new(absolute_id_start, absolute_id_end),
                    ));
                }

                None
            };

        // Parties
        for party in &ast.parties {
            if let Some(range) = extract_identifier_after_keyword(&party.span, "party", &party.name)
            {
                token_infos.push(TokenInfo {
                    range,
                    token_type: TOKEN_PARTY,
                    token_modifiers: MOD_DECLARATION,
                });
            }
        }

        // Policies
        for policy in &ast.policies {
            if let Some(range) =
                extract_identifier_after_keyword(&policy.span, "policy", &policy.name)
            {
                token_infos.push(TokenInfo {
                    range,
                    token_type: TOKEN_POLICY,
                    token_modifiers: MOD_DECLARATION,
                });
            }
        }

        // Types
        for type_def in &ast.types {
            if let Some(range) =
                extract_identifier_after_keyword(&type_def.span, "type", &type_def.name)
            {
                token_infos.push(TokenInfo {
                    range,
                    token_type: TOKEN_TYPE,
                    token_modifiers: MOD_DECLARATION,
                });
            }
        }

        // Assets
        for asset in &ast.assets {
            if let Some(range) = extract_identifier_after_keyword(&asset.span, "asset", &asset.name)
            {
                token_infos.push(TokenInfo {
                    range,
                    token_type: TOKEN_CLASS,
                    token_modifiers: MOD_DECLARATION,
                });
            }
        }

        // Transactions
        for tx in &ast.txs {
            if let Some(range) = extract_identifier_after_keyword(&tx.span, "tx", &tx.name) {
                token_infos.push(TokenInfo {
                    range,
                    token_type: TOKEN_TRANSACTION,
                    token_modifiers: MOD_DECLARATION,
                });
            }
        }

        // Sort tokens by position
        token_infos.sort_by(|a, b| match a.range.start.line.cmp(&b.range.start.line) {
            std::cmp::Ordering::Equal => a.range.start.character.cmp(&b.range.start.character),
            other => other,
        });

        // Remove duplicates
        token_infos.dedup_by(|a, b| a.range.start == b.range.start && a.range.end == b.range.end);

        // Convert to semantic tokens with deltas
        let mut semantic_tokens = Vec::new();
        let mut prev_line = 0;
        let mut prev_start = 0;

        for token_info in token_infos {
            let line = token_info.range.start.line;
            let start = token_info.range.start.character;
            let length = token_info.range.end.character.saturating_sub(start);

            if length == 0 {
                continue;
            }

            let delta_line = line.saturating_sub(prev_line);
            let delta_start = if delta_line == 0 {
                start.saturating_sub(prev_start)
            } else {
                start
            };

            semantic_tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type: token_info.token_type,
                token_modifiers_bitset: token_info.token_modifiers,
            });

            prev_line = line;
            prev_start = start;
        }

        semantic_tokens
    }

    pub fn new_for_client(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    fn get_document(&self, url_arg: &str) -> Result<Rope, Error> {
        let uri = Url::from_str(url_arg)?;

        let document = self
            .documents
            .get(&uri)
            .ok_or(Error::DocumentNotFound(uri))?;

        Ok(document.value().clone())
    }

    fn get_document_protocol(&self, url_arg: &str) -> Result<Protocol, Error> {
        let document = self.get_document(url_arg)?;

        let protocol = Protocol::from_string(document.to_string()).load()?;

        Ok(protocol)
    }

    async fn process_document(&self, uri: Url, text: &str) -> Vec<Diagnostic> {
        let rope = Rope::from_str(text);
        self.documents.insert(uri.clone(), rope.clone());

        let ast = tx3_lang::parsing::parse_string(text);

        match ast {
            Ok(mut ast) => {
                let analysis = tx3_lang::analyzing::analyze(&mut ast);
                analyze_report_to_diagnostic(&rope, &analysis)
            }
            Err(e) => vec![parse_error_to_diagnostic(&rope, &e)],
        }
    }
}
