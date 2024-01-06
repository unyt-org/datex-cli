use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
         Ok(InitializeResult {
             capabilities: ServerCapabilities {
                 hover_provider: Some(HoverProviderCapability::Simple(true)),
                 completion_provider: Some(CompletionOptions::default()),
                 ..Default::default()
             },
             ..Default::default()
         })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

	async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
		self.client
			.log_message(MessageType::INFO, "server initialized!")
			.await;

		Ok(Some(Hover {
			contents: HoverContents::Markup(MarkupContent {
				kind: MarkupKind::Markdown,
				value: "# Example\n123".to_string()
			}),
			range: None
		}))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}