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
