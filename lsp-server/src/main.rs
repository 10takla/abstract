mod distruct;

use distruct::distruct_items;
use parser::{
    lexer2::{
        cache_and_diags::diag::Diag, code::Source, AnyBlock, Args, AssignExpr, Block, ErrorType,
        FnC, Ident, IdentError, Item, Items, Literal, NamedBlock, NamedDistrBlock, Slicable, Slice,
    },
    parse,
};
use std::{
    fmt::Debug,
    iter::{empty, once, Peekable},
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock, RwLock,
    },
};
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| Backend {
        client,
        text: Default::default(),
        legend: Default::default(),
        version: Default::default(),
    });
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}

#[derive(Debug)]
struct Backend {
    client: Client,
    text: RwLock<Option<String>>,
    legend: RwLock<Option<SemanticTokensLegend>>,
    version: AtomicUsize,
}

const TRAIT: SemanticTokenType = SemanticTokenType::new("trait");
const IMPL: SemanticTokenType = SemanticTokenType::new("impl");
const BLOCK: SemanticTokenType = SemanticTokenType::new("block");
const SYMBOL: SemanticTokenType = SemanticTokenType::new("symbol");
const TOKENS: LazyLock<Vec<SemanticTokenType>> = LazyLock::new(|| {
    vec![
        TRAIT,
        IMPL,
        BLOCK,
        SYMBOL,
        SemanticTokenType::VARIABLE,
        SemanticTokenType::STRING,
        SemanticTokenType::NUMBER,
        SemanticTokenType::KEYWORD,
        SemanticTokenType::FUNCTION,
        SemanticTokenType::STRUCT,
        SemanticTokenType::TYPE,
    ]
});

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
                                    token_types: TOKENS.clone(),
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
        dbg!("opened");
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(text) = params.content_changes.first().map(|change| &change.text) else {
            return;
        };
        self.save_text(text).await;
        dbg!("changed");
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: Some(self.version.load(Ordering::SeqCst).to_string()),
            data: dbg!(self.analyze_syntax(uri).await),
        })))
    }

    async fn semantic_tokens_full_delta(
        &self,
        params: SemanticTokensDeltaParams,
    ) -> Result<Option<SemanticTokensFullDeltaResult>> {
        let uri = params.text_document.uri;
        Ok(Some(SemanticTokensFullDeltaResult::Tokens(
            SemanticTokens {
                result_id: Some(self.version.load(Ordering::SeqCst).to_string()),
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
            result_id: Some(self.version.load(Ordering::SeqCst).to_string()),
            data: self.analyze_syntax(uri).await,
        })))
    }
}

impl Backend {
    async fn analyze_syntax(&self, uri: Url) -> Vec<SemanticToken> {
        let Some(code) = self.text.read().unwrap().clone() else {
            return Default::default();
        };
        let (items, diags) = parse(&code);
        let tokens = tokenize(&items, &code);

        self.client
            .publish_diagnostics(
                uri,
                {
                    let iter = &mut get_iter(&code);
                    diags
                        .iter()
                        .cloned()
                        .map(|diag| {
                            let [[start, start_line], [end, end_line]] =
                                diag_split(diag.slice.clone(), iter);
                            Diagnostic {
                                range: Range {
                                    start: Position::new(start_line as u32, start as u32),
                                    // +1, так как в vscode decrement
                                    end: Position::new(end_line as u32, end as u32 + 1),
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: None,
                                source: Some("abstract".to_string()),
                                message: format!("Ошибка парсера: {diag}"),
                                ..Default::default()
                            }
                        })
                        .collect()
                },
                None,
            )
            .await;
        tokens
    }

    async fn save_text(&self, text: &str) {
        *self.text.write().unwrap() = Some(text.to_string());
        self.version.fetch_add(1, Ordering::SeqCst);
    }
}

fn diag_split(
    diag_slice: Slice,
    iter: &mut Peekable<impl Iterator<Item = (usize, [usize; 2])>>,
) -> [[usize; 2]; 2] {
    let [item_start, item_end] = [*diag_slice.start(), *diag_slice.end()];
    let mut acc = [0, 0];
    while let Some(&(i, [line_start, line_end])) = dbg!(iter.peek()) {
        acc = [line_start, i];
        // если начало находится на линии
        if item_start <= line_end {
            let [start_char_index, start_line_index] = [item_start - line_start, i];
            // если конец находится на линии
            if item_end <= line_end {
                return [
                    [start_char_index, start_line_index],
                    [item_end - line_start, i],
                ];
            } else {
                iter.next().unwrap();

                let mut acc = [0, 0];
                // иначе продолжить проход по линиям пока не будет найден конец
                while let Some(&(i, [line_start, line_end])) = iter.peek() {
                    acc = [line_start, i];
                    // если конец находится на линии
                    if item_end <= line_end {
                        // то перейти к следующему элементу
                        return [
                            [start_char_index, start_line_index],
                            [item_end - line_start, i],
                        ];
                    } else {
                        iter.next().unwrap();
                    }
                }
                return [
                    [start_char_index, start_line_index],
                    [item_end - dbg!(acc)[0], acc[1]],
                ];
            }
        } else {
            iter.next().unwrap();
        }
    }
    [[item_start - acc[0], acc[1]], [item_end - acc[0], acc[1]]]
}

#[test]
fn diag_split_() {
    let check = |(source, slice), b: [[usize; 2]; 2]| {
        assert_eq!(diag_split(slice, &mut get_iter(source)), b)
    };

    // выход диганостики за гранцицу кода
    check(("{", 1..=1), [[1, 0], [1, 0]]);
    check(("{", 0..=1), [[0, 0], [1, 0]]);
    check(("  \n{", 0..=4), [[0, 0], [1, 1]]);
    check(("  \n\n\n{", 0..=6), [[0, 0], [1, 3]]);
}

#[test]
fn diag() {
    let check = |source, b: Vec<[[usize; 2]; 2]>| {
        assert_eq!(
            parse(source)
                .1
                .into_iter()
                .map(|v| diag_split(v.slice.clone(), &mut get_iter(source)))
                .collect::<Vec<_>>(),
            b
        );
    };
    check("22dd", vec![[[0, 0], [3, 0]]]);
    check(
        "
22dd
        ",
        vec![[[0, 1], [3, 1]]],
    );

    check(r#"22dd 22dd  "#, vec![[[0, 0], [3, 0]], [[5, 0], [8, 0]]]);

    check(
        r#"22dd 22dd  
7fs 22dsf  

   2hg"#,
        vec![
            [[0, 0], [3, 0]],
            [[5, 0], [8, 0]],
            [[0, 1], [2, 1]],
            [[4, 1], [8, 1]],
            [[3, 3], [5, 3]],
        ],
    );
    check(
        r#"22dd 22dd  
"sdfsfdsf 22dsf  

   2hg"#,
        vec![[[0, 0], [3, 0]], [[5, 0], [8, 0]], [[5, 3], [5, 3]]],
    );
    check(r#"{"#, vec![[[1, 0], [1, 0]]]);
}

fn tokenize(items: &Items, code: &str) -> Vec<SemanticToken> {
    let items = &mut distruct_items(items);

    let iter = &mut get_iter(code).peekable();

    let (mut tokens, mut last_start, mut last_line) = (vec![], None, 0);
    while let Some((v, type_)) = items.next() {
        item_split(
            v,
            iter,
            &mut last_start,
            &mut last_line,
            |delta_line, delta_start, length| {
                tokens.push(SemanticToken {
                    delta_line: delta_line as u32,
                    delta_start: delta_start as u32,
                    length: length as u32,
                    token_type: token_type(type_.clone()),
                    token_modifiers_bitset: 0,
                });
            },
        );
    }
    tokens
}

fn item_split(
    [item_start, item_end]: [usize; 2],
    iter: &mut Peekable<impl Iterator<Item = (usize, [usize; 2])>>,
    last_start: &mut Option<usize>,
    last_line: &mut usize,
    mut push_token: impl FnMut(usize, usize, usize),
) {
    while let Some(&(i, [line_start, line_end])) = iter.peek() {
        // если начало элемента находится на линии
        if item_start <= line_end {
            let delta_start = item_start - last_start.unwrap_or(line_start);

            // если конец элемента находится на линии
            if item_end <= line_end {
                push_token(i - *last_line, delta_start, item_end - item_start + 1);
                *last_start = Some(item_start);
                *last_line = i;
                return;
            } else {
                push_token(i - *last_line, delta_start, line_end - item_start + 1);
                *last_start = Some(item_start);
                *last_line = i;
                iter.next().unwrap();

                // иначе продолжить проход по линиям пока не будет найден конец элемента
                while let Some(&(i, [line_start, line_end])) = iter.peek() {
                    // если конец элемента находится на линии
                    if item_end <= line_end {
                        // то перейти к следующему элементу
                        push_token(i - *last_line, 0, item_end - line_start + 1);
                        *last_start = Some(item_start);
                        *last_line = i;
                        return;
                    } else {
                        // иначе добавить диапазон до конца линии токен, как часть одного общего токена (необходимо, т.к. lsp поддерживает токены построчно)
                        push_token(i - *last_line, 0, line_end - line_start + 1);
                        *last_line = i;
                        iter.next().unwrap();
                    }
                }
            }
        } else {
            *last_start = None;
            iter.next().unwrap();
        }
    }
    unreachable!();
}

fn get_iter<'a>(code: &'a str) -> Peekable<impl Iterator<Item = (usize, [usize; 2])> + 'a> {
    let mut acc = 0;
    code.split_inclusive('\n')
        .enumerate()
        .peekable()
        .map(move |(i, line)| {
            let start = acc;
            let len = line.chars().count();
            if len > 0 {
                acc += len;
            }
            let end = if acc != 0 { acc - 1 } else { 0 };
            (i, [start, end])
        })
        .peekable()
}

fn token_type(t: SemanticTokenType) -> u32 {
    TOKENS
        .iter()
        .enumerate()
        .find(|&(_, token)| *token == t)
        .unwrap()
        .0 as u32
}

#[cfg(test)]
mod tests {
    use crate::{tokenize, BLOCK, TOKENS};
    use lexer::parse;
    use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

    fn sem_token(
        [delta_line, delta_start, length]: [u32; 3],
        token_type: SemanticTokenType,
    ) -> SemanticToken {
        SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: TOKENS.iter().position(|t| *t == token_type).unwrap() as u32,
            token_modifiers_bitset: 0,
        }
    }

    fn any_types(code: &str, vec: Vec<SemanticToken>) {
        assert_eq!(tokenize(&parse(code).0, code), vec);
    }

    fn all_types(token_type: SemanticTokenType) -> impl Fn(&'static str, Vec<[u32; 3]>) {
        move |code: &'static str, b: Vec<[u32; 3]>| {
            any_types(
                code,
                b.into_iter()
                    .map(|v| sem_token(v, token_type.clone()))
                    .collect(),
            );
        }
    }
    #[test]
    fn tokenize_() {
        all_types(SemanticTokenType::VARIABLE)("abc abc", vec![[0, 0, 3], [0, 4, 3]]);
        all_types(SemanticTokenType::VARIABLE)("abc\ndef", vec![[0, 0, 3], [1, 0, 3]]);
        all_types(SemanticTokenType::VARIABLE)("a\nb", vec![[0, 0, 1], [1, 0, 1]]);
        all_types(SemanticTokenType::VARIABLE)("\na", vec![[1, 0, 1]]);
        all_types(SemanticTokenType::VARIABLE)("\n a", vec![[1, 1, 1]]);
        all_types(SemanticTokenType::VARIABLE)("a\n a", vec![[0, 0, 1], [1, 1, 1]]);
        all_types(SemanticTokenType::VARIABLE)(
            "main sdfsfd sdf sf sdf sf sfd sdf sf",
            vec![
                [0, 0, 4],
                [0, 5, 6],
                [0, 7, 3],
                [0, 4, 2],
                [0, 3, 3],
                [0, 4, 2],
                [0, 3, 3],
                [0, 4, 3],
                [0, 4, 2],
            ],
        );

        all_types(BLOCK)("main {}", vec![[0, 0, 4], [0, 5, 1], [0, 1, 1]]);
        all_types(BLOCK)("main {\n}", vec![[0, 0, 4], [0, 5, 1], [1, 0, 1]]);
        all_types(BLOCK)("main {\n\n}", vec![[0, 0, 4], [0, 5, 1], [2, 0, 1]]);

        any_types(
            "main {\n} a",
            vec![
                sem_token([0, 0, 4], BLOCK),
                sem_token([0, 5, 1], BLOCK),
                sem_token([1, 0, 1], BLOCK),
                sem_token([0, 2, 1], SemanticTokenType::VARIABLE),
            ],
        );
        any_types(
            "main {\n\n} a",
            vec![
                sem_token([0, 0, 4], BLOCK),
                sem_token([0, 5, 1], BLOCK),
                sem_token([2, 0, 1], BLOCK),
                sem_token([0, 2, 1], SemanticTokenType::VARIABLE),
            ],
        );
    }

    #[test]
    fn issues() {
        // issue - не все строки подсвечиваются как строка
        all_types(SemanticTokenType::STRING)(
            r#""dsf
sdfsdf
sdfs
sdfsdf
sdf""#,
            vec![[0, 0, 5], [1, 0, 7], [1, 0, 5], [1, 0, 7], [1, 0, 4]],
        );
    }
}
