use lexer::{
    items::{
        item::{
            assign_expr::{AssignExpr, AssignExprType},
            block::{
                distruct::BlockDistruct,
                init::{self, unnamed::UnnamedBlock, InitBlock},
                Block,
            },
            Item,
        },
        Items,
    },
    Code, DiagParse, Diagn, Parse, Slicable,
};
use std::{fmt::Debug, iter::Enumerate, str::Lines, sync::RwLock};
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        text: Default::default(),
        legend: Default::default(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[derive(Debug)]
struct Backend {
    client: Client,
    text: RwLock<Option<String>>,
    legend: RwLock<Option<SemanticTokensLegend>>,
}

const NAMED_BLOCK: SemanticTokenType = SemanticTokenType::new("namedBlock");

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: {
                                let legend = SemanticTokensLegend {
                                    token_types: vec![
                                        SemanticTokenType::VARIABLE,
                                        SemanticTokenType::FUNCTION,
                                        SemanticTokenType::new("namedBlock"),
                                    ],
                                    token_modifiers: vec![],
                                };
                                *self.legend.write().unwrap() = Some(legend.clone());
                                legend
                            },
                            work_done_progress_options: WorkDoneProgressOptions {
                                work_done_progress: Some(true),
                            },
                            range: None,
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.save_text(&params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(text) = params.content_changes.first().map(|change| &change.text) else {
            return;
        };
        self.save_text(text).await;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: Some("some_id".to_string()),
            data: self.analyze_syntax(uri).await,
        })))
    }

    async fn semantic_tokens_full_delta(
        &self,
        params: SemanticTokensDeltaParams,
    ) -> Result<Option<SemanticTokensFullDeltaResult>> {
        let uri = params.text_document.uri;
        Ok(Some(SemanticTokensFullDeltaResult::Tokens(
            SemanticTokens {
                result_id: Some("some_id".to_string()),
                data: self.analyze_syntax(uri).await,
            },
        )))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let uri = params.text_document.uri;
        Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
            result_id: Some("some_id".to_string()),
            data: self.analyze_syntax(uri).await,
        })))
    }
}

impl Backend {
    async fn analyze_syntax(&self, uri: Url) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();

        let Some(text) = self.text.read().unwrap().clone() else {
            return tokens;
        };
        let (items, diags) = Items::analyz(&text);

        let token_type = {
            let Some(legend) = self.legend.read().unwrap().clone() else {
                return tokens;
            };
            move |t| {
                legend
                    .token_types
                    .iter()
                    .enumerate()
                    .find(|&(_, token)| *token == t)
                    .unwrap()
                    .0 as u32
            }
        };

        let mut lines = text.split_inclusive('\n').enumerate();

        tokenize(&mut tokens, &mut items.iter(), &mut lines, &token_type);

        let diagnostics = diags
            .iter()
            .cloned()
            .map(|diag| Diagnostic {
                range: Range {
                    start: Position::new(0, diag.pos.unwrap() as u32),
                    end: Position::new(0, diag.pos.unwrap() as u32 + 1),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                source: Some("abstract".to_string()),
                message: format!("Ошибка парсера: {}", diag.expect(&Code::new(&text), diags.pos.unwrap())),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;

        tokens
    }

    async fn save_text(&self, text: &str) {
        *self.text.write().unwrap() = Some(text.to_string());
    }
}

fn tokenize<'a, T: Iterator<Item = &'a Item<'a>> + Debug>(
    tokens: &mut Vec<SemanticToken>,
    items: &mut T,
    lines: &mut impl Iterator<Item = (usize, &'a str)>,
    token_type: &impl Fn(SemanticTokenType) -> u32,
) {
    let glen = |line: &str| line.chars().count();
    let get_token = |delta_line, delta_start, length, item: &Item| {
        let token_type = match item {
            Item::Ident(_) => token_type(SemanticTokenType::VARIABLE),
            Item::Block(block) => match block {
                Block::Init(init) => match init {
                    InitBlock::Named(name_block) => {
                        // tokenize(
                        //     tokens,
                        //     &mut name_block.block.items.iter(),
                        //     lines,
                        //     token_type,
                        // );
                        token_type(SemanticTokenType::FUNCTION)
                    }
                    InitBlock::Unnamed(unnamed_block) => {
                        // tokenize(tokens, &unnamed_block.items, lines, token_type);
                        token_type(SemanticTokenType::FUNCTION)
                    }
                },
                Block::Distruct(distruct) => match distruct {
                    BlockDistruct::Init(init) => {
                        // tokenize(tokens, &init.named_block.block.items, lines, token_type);
                        token_type(NAMED_BLOCK)
                    }
                    BlockDistruct::Call(_) => token_type(SemanticTokenType::FUNCTION),
                },
            },
            Item::AssignExpr(assign_expr) => match assign_expr.type_ {
                AssignExprType::Assign => token_type(SemanticTokenType::FUNCTION),
                AssignExprType::AssignAnd(_) => token_type(SemanticTokenType::FUNCTION),
            },
            _ => 0,
        };

        SemanticToken {
            delta_line: delta_line as u32,
            delta_start: delta_start as u32,
            length: length as u32,
            token_type,
            token_modifiers_bitset: 0,
        }
    };

    let mut lines = lines.peekable();
    let (mut last_start, mut last_line, mut len) = (None, 0, 0);
    'l: while let Some(item) = items.next() {
        let [start, end] = (|| {
            let v = item.get_slice();
            [*v.start(), *v.end()]
        })();
        while let Some(&(i, line)) = lines.peek() {
            let tmp_len = len + glen(line);
            if start < tmp_len {
                let delta_line = i - last_line;
                let delta_start = start - last_start.unwrap_or(len);

                if end < tmp_len {
                    let length = end - start + 1;

                    tokens.push(get_token(delta_line, delta_start, length, item));
                    last_start = Some(start);
                    last_line = i;
                    continue 'l;
                } else {
                    tokens.push(get_token(delta_line, delta_start, tmp_len - start, item));
                    lines.next().unwrap();
                    last_start = Some(start);
                    last_line = i;
                    len = tmp_len;

                    while let Some(&(i, line)) = lines.peek() {
                        let line_len = glen(line);
                        let tmp_len = len + line_len;
                        let delta_line = i - last_line;
                        if end < tmp_len {
                            tokens.push(get_token(delta_line, 0, end - len + 1, item));
                            last_start = Some(len);
                            last_line = i;
                            continue 'l;
                        } else {
                            tokens.push(get_token(delta_line, 0, line_len, item));
                            last_line = i;
                            lines.next().unwrap();
                            len = tmp_len;
                        }
                    }
                }
            } else {
                lines.next().unwrap();
                len = tmp_len;
                last_start = None;
            }
        }
    }
}

#[test]
fn tokenize_test() {
    let fast = |[delta_line, delta_start, length]: [u32; 3], token_type| SemanticToken {
        delta_line,
        delta_start,
        length,
        token_type,
        token_modifiers_bitset: 0,
    };

    let any_types = |code, vec| {
        let get_tokens = |code: &'static str| {
            let token_type = |t| {
                vec![
                    SemanticTokenType::VARIABLE,
                    SemanticTokenType::FUNCTION,
                    SemanticTokenType::new("namedBlock"),
                ]
                .iter()
                .enumerate()
                .find(|&(_, token)| *token == t)
                .unwrap()
                .0 as u32
            };

            let mut tokens = vec![];
            let mut lines = code.split_inclusive('\n').enumerate();

            tokenize(
                &mut tokens,
                &mut Items::analyz(code).0.iter(),
                &mut lines,
                &token_type,
            );
            tokens
        };

        assert_eq!(get_tokens(code), vec);
    };

    let all_types = |token_type| {
        move |code: &'static str, b: Vec<[u32; 3]>| {
            any_types(
                code,
                b.into_iter()
                    .map(|v| fast(v, token_type))
                    .collect::<Vec<_>>(),
            );
        }
    };

    all_types(0)("abc abc", vec![[0, 0, 3], [0, 4, 3]]);
    all_types(0)("abc\ndef", vec![[0, 0, 3], [1, 0, 3]]);
    all_types(0)("a\nb", vec![[0, 0, 1], [1, 0, 1]]);
    all_types(0)("\na", vec![[1, 0, 1]]);
    all_types(0)("\n a", vec![[1, 1, 1]]);
    all_types(0)("a\n a", vec![[0, 0, 1], [1, 1, 1]]);

    all_types(1)("main {}", vec![[0, 0, 7]]);
    all_types(1)("main {\n}", vec![[0, 0, 7], [1, 0, 1]]);
    all_types(1)("main {\n\n}", vec![[0, 0, 7], [1, 0, 1], [1, 0, 1]]);

    any_types(
        "main {\n} a",
        vec![fast([0, 0, 7], 1), fast([1, 0, 1], 1), fast([0, 2, 1], 0)],
    );
    any_types(
        "main {\n\n} a",
        vec![
            fast([0, 0, 7], 1),
            fast([1, 0, 1], 1),
            fast([1, 0, 1], 1),
            fast([0, 2, 1], 0),
        ],
    );
}
