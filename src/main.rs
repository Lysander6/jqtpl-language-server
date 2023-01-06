// use tree_sitter::Parser;
// use tree_sitter_jqtpl::language;

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct TextDocumentItem {
    uri: Url,
    text: String,
    version: i32,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["{".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "jqtpl-language-server reports for duty!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![CompletionItem {
            label: "{{if ...}}".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: "{{if $1}}$2{{/if}}$0".to_string(),
                range: Range {
                    start: params.text_document_position.position,
                    end: params.text_document_position.position,
                },
            })),
            ..Default::default()
        }])))
    }

    async fn did_open(&self, mut params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "jqtpl-language-server did open {}",
                    params.text_document.uri
                ),
            )
            .await;

        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.text_document.text),
            version: params.text_document.version,
        })
        .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "jqtpl-language-server did change {}",
                    params.text_document.uri
                ),
            )
            .await;

        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
        })
        .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "jqtpl-language-server says bye, bye!")
            .await;

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    // let mut parser = Parser::new();
    // parser.set_language(language()).unwrap();
    //
    // let tree = parser.parse("<p>{{= hello.world}}</p>", None).unwrap();
    // println!("{}", tree.root_node().to_sexp());
}
