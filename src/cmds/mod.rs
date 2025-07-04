use serde_json::Value;

use tower_lsp::lsp_types::ExecuteCommandParams;

use crate::{Context, Error};

mod generate_ast;
mod generate_diagram;
mod generate_tir;

pub async fn handle_command(
    context: &Context,
    params: ExecuteCommandParams,
) -> Result<Option<Value>, Error> {
    match params.command.as_str() {
        "generate-tir" => generate_tir::run(context, params.arguments).await,
        "generate-ast" => generate_ast::run(context, params.arguments).await,
        "generate-diagram" => generate_diagram::run(context, params.arguments).await,
        _ => Err(Error::InvalidCommand(params.command)),
    }
}
