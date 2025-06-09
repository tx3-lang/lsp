use clap::Parser;
use tower::ServiceBuilder;
use tower_lsp::{LspService, Server};
use tx3_lsp::Context;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    stdio: bool,
}

#[tokio::main]
async fn main() {
    let _args = Args::parse();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Context::new_for_client);

    // Create a logging middleware
    let service = ServiceBuilder::new()
        .map_request(|request| request)
        .map_response(|response| response)
        .service(service);

    let server = Server::new(stdin, stdout, socket);

    server.serve(service).await;
}
