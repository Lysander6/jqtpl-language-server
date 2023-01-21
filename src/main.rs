mod parser;

use tree_sitter::{Parser, Query, QueryCursor};
use tree_sitter_jqtpl::language;

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

        let lang = language();

        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree = parser.parse(&params.text, None).unwrap();

        let matches = {
            let query = Query::new(lang, "(ERROR) @error").unwrap();
            let mut cursor = QueryCursor::new();

            // matches/captures are fucked, goddammit
            // https://github.com/tree-sitter/tree-sitter/issues/1656
            // https://github.com/tree-sitter/tree-sitter/issues/608
            let mut bla = vec![];

            for m in cursor.matches(&query, tree.root_node(), params.text.as_bytes()) {
                bla.extend_from_slice(
                    &m.captures
                        .iter()
                        .map(|c| {
                            let r = c.node.byte_range();
                            let start_line = rope.byte_to_line(r.start);
                            let start_col = r.start - rope.line_to_byte(start_line);
                            let end_line = rope.byte_to_line(r.end);
                            let end_col = r.end - rope.line_to_byte(end_line);
                            ((start_line, start_col), (end_line, end_col))
                        })
                        .collect::<Vec<_>>()[..],
                );
            }

            bla
        };

        self.client
            .log_message(MessageType::INFO, format!("Error indices: {:?}", matches))
            .await;

        if matches.len() > 1 {
            // Temporarily just publish some (any) error
            self.client
                .publish_diagnostics(
                    params.uri,
                    vec![Diagnostic::new_simple(
                        Range {
                            start: Position {
                                line: matches[1].0.0 as u32,
                                character: matches[1].0.1 as u32,
                            },
                            end: Position {
                                line: matches[1].1.0 as u32,
                                character: matches[1].1.1 as u32,
                            },
                        },
                        "whoopsie!".to_string(),
                    )],
                    Some(params.version),
                )
                .await;
        } else {
            self.client
                .publish_diagnostics(params.uri, vec![], Some(params.version))
                .await;
        }

        let sexp = tree.root_node().to_sexp();
        self.client.log_message(MessageType::INFO, sexp).await;
    }
}

const SNIPPETS: [(&str, &str); 6] = [
    ("{{= ...}}", "{{= $1}}$0"),
    ("{{if ...}}", "{{if $1}}$2{{/if}}$0"),
    ("{{var ...}}", "{{var ${1:locals.}$2 = $3}}$0"),
    ("{{html ...}}", "{{html $1}}$0"),
    (
        "{{each ...}}",
        "{{each({${2:i}, ${3:item}}) $1}}$4{{/each}}$0",
    ),
    (
        "{{tmpl ...}}",
        "{{tmpl({$3}) partials.getTemplate($1, $2)}}$0",
    ),
];

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
        let document = &*self
            .document_map
            .get(&params.text_document_position.text_document.uri.to_string())
            .unwrap();
        let Position { line, character } = params.text_document_position.position;
        let cursor_idx: usize = character.try_into().unwrap();
        let line = document.line(line.try_into().unwrap());

        let mut edit_range = Range {
            start: params.text_document_position.position,
            end: params.text_document_position.position,
        };
        if cursor_idx > 1 && line.char(cursor_idx - 2) == '{' {
            edit_range.start.character -= 1;
        }
        if cursor_idx > 0 && line.char(cursor_idx - 1) == '{' {
            edit_range.start.character -= 1;
        }
        if cursor_idx < line.len_chars() && line.char(cursor_idx) == '}' {
            edit_range.end.character += 1;
        }
        if cursor_idx < line.len_chars().saturating_sub(1) && line.char(cursor_idx + 1) == '}' {
            edit_range.end.character += 1;
        }

        let completions = SNIPPETS
            .iter()
            .map(|(label, new_text)| CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    new_text: new_text.to_string(),
                    range: edit_range,
                })),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        Ok(Some(CompletionResponse::Array(completions)))
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
