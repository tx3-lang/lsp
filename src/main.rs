use tower::ServiceBuilder;
use tower_lsp::{LspService, Server};
use tx3_lsp::Context;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Context::new_for_client);

    // Create a logging middleware
    let service = ServiceBuilder::new()
        .map_request(|request| request)
        .map_response(|response| response)
        .service(service);

    Server::new(stdin, stdout, socket).serve(service).await;
}
