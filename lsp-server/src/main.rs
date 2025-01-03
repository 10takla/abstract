use lexer::{
    lexer2::{
        code::Source, diag::Diag, AnyBlock, AssignExpr, Block, ErrorType, FnArgs, Ident,
        IdentError, Idents, Item, Items, Keyword, Literal, NamedBlock, NamedDistrBlock, Slicable,
    },
    parse,
};
use std::{
    fmt::Debug,
    iter::{empty, once},
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

const BLOCK: SemanticTokenType = SemanticTokenType::new("namedBlock");
const TOKENS: LazyLock<Vec<SemanticTokenType>> = LazyLock::new(|| {
    vec![
        SemanticTokenType::VARIABLE,
        BLOCK,
        SemanticTokenType::STRING,
        SemanticTokenType::NUMBER,
        SemanticTokenType::KEYWORD,
        SemanticTokenType::FUNCTION,
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
                diags
                    .iter()
                    .cloned()
                    .map(|diag| {
                        let [[start, start_line], [end, end_line]] = tmp(&diag, &code);
                        Diagnostic {
                            range: Range {
                                start: Position::new(start_line as u32, start as u32),
                                end: Position::new(end_line as u32, end as u32),
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: None,
                            source: Some("abstract".to_string()),
                            message: format!("Ошибка парсера: {diag}"),
                            ..Default::default()
                        }
                    })
                    .collect(),
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

fn tmp(diag: &Diag, code: &str) -> [[usize; 2]; 2] {
    let lines = &mut code.split_inclusive('\n').enumerate().peekable();

    let [item_start, item_end] = [*diag.slice.start(), *diag.slice.end()];
    let mut acc = 0;
    let mut iter = lines.map(|(i, line)| {
        let start = acc;
        let len = line.chars().count();
        if len > 0 {
            acc += len;
        }
        let end = acc;
        (i, [start, end])
    });

    while let Some((i, [line_start, line_end])) = iter.next() {
        // если начало находится на линии
        if item_start <= line_end {
            let [start_o, start_line_o] = [item_start - line_start, i];

            // если конец находится на линии
            if item_end <= line_end {
                return [[start_o, start_line_o], [item_end - line_start, i]];
            } else {
                // иначе продолжить проход по линиям пока не будет найден конец
                while let Some((i, [line_start, line_end])) = iter.next() {
                    // если конец находится на линии
                    if item_end <= line_end {
                        // то перейти к следующему элементу
                        return [[start_o, start_line_o], [item_end - line_start, i]];
                    }
                }
            }
        }
    }
    unreachable!()
}

#[test]
fn diag() {
    let check = |source, b| {
        assert_eq!(tmp(&parse(source).1[0], source), b);
    };

    check("22dd", [[0, 0], [3, 0]]);
    check(
        "
22dd",
        [[4, 1], [7, 1]],
    );
}

fn tokenize(items: &Items, code: &str) -> Vec<SemanticToken> {
    let items = &mut distruct_items(items);
    let lines = &mut code.split_inclusive('\n').enumerate().peekable();

    let glen = |line: &str| line.chars().count();

    let (mut tokens, mut last_start, mut last_line, mut len) = (vec![], None, 0, 0);
    'l: while let Some(([start, end], type_)) = items.next() {
        let mut push_token = |delta_line, delta_start, length| {
            tokens.push(SemanticToken {
                delta_line: delta_line as u32,
                delta_start: delta_start as u32,
                length: length as u32,
                token_type: token_type(type_.clone()),
                token_modifiers_bitset: 0,
            });
        };

        macro_rules! go_to_next_item {
            ({$($arg:expr),+} $start:expr, $i:expr) => {
                push_token($($arg),+);
                last_start = Some($start);
                last_line = $i;
                continue 'l;
            };
        }

        while let Some(&(i, line)) = lines.peek() {
            // len with increment line len
            let end_len = len + glen(line);

            // если начало элемента находится на линии
            if start < end_len {
                let delta_line = i - last_line;
                let delta_start = start - last_start.unwrap_or(len);

                // если конец элемента находится на линии
                if end < end_len {
                    go_to_next_item!(
                        {delta_line, delta_start, end - start + 1}
                        start, i
                    );
                } else {
                    push_token(delta_line, delta_start, end_len - start);
                    lines.next().unwrap();
                    last_start = Some(start);
                    last_line = i;
                    len = end_len;

                    // иначе продолжить проход по линиям пока не будет найден конец элемента
                    while let Some(&(i, line)) = lines.peek() {
                        let line_len = glen(line);
                        let tmp_len = len + line_len;
                        let delta_line = i - last_line;

                        // если конец элемента находится на линии
                        if end < tmp_len {
                            // то перейти к следующему элементу
                            go_to_next_item!(
                                {delta_line, 0, end - len + 1}
                                len, i
                            );
                        } else {
                            // иначе добавить диапазон до конца линии токен, как часть одного общего токена (необходимо, т.к. lsp поддерживает токены построчно)
                            push_token(delta_line, 0, line_len);
                            last_line = i;
                            lines.next().unwrap();
                            len = tmp_len;
                        }
                    }
                }
            } else {
                lines.next().unwrap();
                len = end_len;
                last_start = None;
            }
        }
    }

    tokens
}

fn token_type(t: SemanticTokenType) -> u32 {
    TOKENS
        .iter()
        .enumerate()
        .find(|&(_, token)| *token == t)
        .unwrap()
        .0 as u32
}

type DistrItem = ([usize; 2], SemanticTokenType);

// итератор для ленивого прохода
fn distruct_items<'a>(items: &'a Items) -> impl Iterator<Item = DistrItem> + 'a {
    items.iter().flat_map(distruct_item)
}

// итератор для ленивого на всех вложенных уровнях
type T<'a> = Box<dyn Iterator<Item = DistrItem> + 'a>;
fn distruct_item<'a>(item: &'a Item) -> T<'a> {
    fn fast_once<'a>(
        v: &impl StartEnd,
        t: SemanticTokenType,
    ) -> impl Iterator<Item = DistrItem> + 'a {
        once((v.start_end(), t))
    }
    fn fast_box<'a>(v: &impl StartEnd, t: SemanticTokenType) -> T<'a> {
        Box::new(fast_once(v, t))
    }

    let block = |v: &'a Block| {
        fast_once(&v.0, BLOCK)
            .chain(distruct_items(&v.1))
            .chain(fast_once(&v.2, BLOCK))
    };
    let named_block = |v: &'a NamedBlock| Box::new(fast_once(&v.0, BLOCK).chain(block(&v.1)));

    let ident = |v| fast_box(v, SemanticTokenType::VARIABLE);

    let literal = |v: &'a self::Literal| {
        use self::Literal::*;
        match v {
            String(..) => fast_box(v, SemanticTokenType::STRING),
            Number(..) => fast_box(v, SemanticTokenType::NUMBER),
        }
    };
    use Item::*;
    match item {
        FnHead(v) => Box::new(
            fast_box(&v.0, SemanticTokenType::KEYWORD)
                .chain(fast_box(&v.1, SemanticTokenType::FUNCTION))
                .chain(v.2.color())
                .chain(block(&v.3)),
        ),
        AnyBlock(v) => {
            use self::AnyBlock::*;
            match v {
                Block(v) => Box::new(block(v)),
                NamedBlock(v) => named_block(v),
                NamedDistrBlock(v) => Box::new(named_block(&v.0).chain(fast_once(&v.1, BLOCK))),
                DistrBlock(v) => fast_box(v, BLOCK),
            }
        }
        Ignore(..) => Box::new(empty()),
        AssignExpr(v) => {
            use self::AssignExpr::*;
            match v {
                Assign(v) => Box::new(ident(&v.0).chain(literal(&v.2))),
                AssignAnd(v) => Box::new(ident(&v.0).chain(literal(&v.2))),
            }
        }
        Literal(v) => literal(v),
        Idents(v) => {
            use self::Idents::*;
            match v {
                Ident(v) => ident(v),
                Keyword(v) => fast_box(v, SemanticTokenType::KEYWORD),
            }
        }
    }
}

trait Tr {
    fn color(&self) -> impl Iterator<Item = DistrItem> {
        self.cl().into_iter().map(|(a, b)| (a.start_end(), b))
    }
    fn cl(&self) -> Vec<(&dyn StartEnd, SemanticTokenType)>;
}

impl Tr for FnArgs {
    fn cl(&self) -> Vec<(&dyn StartEnd, SemanticTokenType)> {
        use FnArgs::*;
        match self {
            StructArgsC(v) => {
                vec![
                    (&v.0, SemanticTokenType::FUNCTION),
                    (&v.2, SemanticTokenType::FUNCTION),
                ]
            }
            TupleArgsC(v) => {
                vec![
                    (&v.0, SemanticTokenType::FUNCTION),
                    (&v.2, SemanticTokenType::FUNCTION),
                ]
            }
        }
    }
}

trait StartEnd: Slicable {
    fn start_end(&self) -> [usize; 2] {
        let v = self.slice();
        [*v.start(), *v.end()]
    }
}
impl<T: Slicable> StartEnd for T {}

#[test]
fn tokenize_() {
    let sem_token = |[delta_line, delta_start, length]: [u32; 3], token_type| SemanticToken {
        delta_line,
        delta_start,
        length,
        token_type: TOKENS.iter().position(|t| *t == token_type).unwrap() as u32,
        token_modifiers_bitset: 0,
    };

    let any_types = |code, vec| {
        assert_eq!(tokenize(&parse(code).0, code), vec);
    };

    let all_types = |token_type: SemanticTokenType| {
        move |code: &'static str, b: Vec<[u32; 3]>| {
            any_types(
                code,
                b.into_iter()
                    .map(|v| sem_token(v, token_type.clone()))
                    .collect(),
            );
        }
    };

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
