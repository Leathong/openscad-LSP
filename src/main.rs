use std::{collections::HashMap, error::Error};

use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument},
    request::{Completion, HoverRequest},
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverContents, HoverParams, HoverProviderCapability, InsertTextFormat, InsertTextMode,
    MarkedString, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentContentChangeEvent, TextDocumentPositionParams, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree, TreeCursor};

const BUILTINS_SCAD: &str = include_str!("builtins.scad");

const KEYWORDS: &[(&str, &str)] = &[
    ("else", "else {  $0\n}"),
    ("false", "false"),
    ("for", "for (${1:LOOP}) {\n  $0\n}"),
    ("function", "function ${1:NAME}(${2:ARGS}) = $0;"),
    ("if", "if (${1:COND}) {\n  $0\n}"),
    ("include", "include <${1:PATH}>;$0"),
    ("intersection_for", "intersection_for(${1:LOOP}) {\n  $0\n}"),
    ("let", "let (${1:VARS}) $0"),
    ("module", "module ${1:NAME}(${2:ARGS}) {\n  $0\n}"),
    ("true", "true"),
    ("use", "use <${1:PATH}>;$0"),
];

#[derive(Clone, Debug)]
struct Param {
    name: String,
    default: Option<String>,
}

impl Param {
    fn parse_declaration(code: &str, node: &Node) -> Vec<Param> {
        node.children(&mut node.walk())
            .filter_map(|child| match child.kind() {
                "identifier" => Some(Param {
                    name: code[child.start_byte()..child.end_byte()].to_owned(),
                    default: None,
                }),
                "assignment" => child.child_by_field_name("left").and_then(|left| {
                    child.child_by_field_name("right").map(|right| Param {
                        name: code[left.start_byte()..left.end_byte()].to_owned(),
                        default: Some(code[right.start_byte()..right.end_byte()].to_owned()),
                    })
                }),
                "special_variable" => None,
                _ => None,
            })
            .collect()
    }

    fn make_snippet(params: &[Param]) -> String {
        params
            .iter()
            .filter_map(|p| p.default.is_none().then(|| &p.name))
            .enumerate()
            .map(|(i, name)| format!("${{{}:{}}}", i + 1, name))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Clone, Debug)]
enum ItemKind {
    Variable,
    Function(Vec<Param>),
    Keyword(String),
    Module { group: bool, params: Vec<Param> },
}

impl ItemKind {
    fn completion_kind(&self) -> CompletionItemKind {
        match self {
            ItemKind::Variable => CompletionItemKind::Variable,
            ItemKind::Function(_) => CompletionItemKind::Function,
            ItemKind::Keyword(_) => CompletionItemKind::Keyword,
            ItemKind::Module { .. } => CompletionItemKind::Module,
        }
    }
}

struct Item {
    name: String,
    kind: ItemKind,
}

impl Item {
    fn make_snippet(&self) -> String {
        match &self.kind {
            ItemKind::Variable => self.name.clone(),
            ItemKind::Function(ref params) => {
                format!("{}({})$0", self.name, Param::make_snippet(params))
            }
            ItemKind::Keyword(comp) => comp.clone(),
            ItemKind::Module { params, group } => {
                let params = Param::make_snippet(params);
                if *group {
                    format!("{}({}) {{\n  $0\n}}", self.name, params)
                } else {
                    format!("{}({});$0", self.name, params)
                }
            }
        }
    }

    fn make_hover(&self) -> String {
        let format_params = |params: &[Param]| {
            params
                .iter()
                .map(|p| match &p.default {
                    Some(d) => format!("{}={}", p.name, d),
                    None => p.name.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        match &self.kind {
            ItemKind::Variable => "".to_owned(),
            ItemKind::Function(params) => format!("{}({})", self.name, format_params(params)),
            ItemKind::Keyword(_) => self.name.clone(),
            ItemKind::Module { params, .. } => {
                format!("{}({})", self.name, format_params(params))
            }
        }
    }
}

impl Item {
    fn parse(code: &str, node: &Node) -> Option<Self> {
        let extract_name = |name| {
            node.child_by_field_name(name)
                .map(|child| code[child.start_byte()..child.end_byte()].to_owned())
        };

        match node.kind() {
            "module_declaration" => Some(Self {
                name: extract_name("name")?,
                kind: ItemKind::Module {
                    group: false,
                    params: node
                        .child_by_field_name("parameters")
                        .map_or(vec![], |params| Param::parse_declaration(code, &params)),
                },
            }),
            "function_declaration" => Some(Self {
                name: extract_name("name")?,
                kind: ItemKind::Function(
                    node.child_by_field_name("parameters")
                        .map_or(vec![], |params| Param::parse_declaration(code, &params)),
                ),
            }),
            "assignment" => Some(Self {
                name: extract_name("left")?,
                kind: ItemKind::Variable,
            }),
            _ => None,
        }
    }
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let (connection, io_threads) = Connection::stdio();
    let mut server = Server::new(connection);
    server.main_loop()?;
    io_threads.join()?;
    Ok(())
}

fn find_offset(text: &str, pos: Position) -> Option<usize> {
    let mut line_start = 0;
    for _ in 0..pos.line {
        line_start = text[line_start..].find('\n')? + line_start + 1;
    }
    Some(line_start + pos.character as usize)
}

fn to_point(p: Position) -> Point {
    Point {
        row: p.line as usize,
        column: p.character as usize,
    }
}

fn to_position(p: Point) -> Position {
    Position {
        line: p.row as u32,
        character: p.column as u32,
    }
}

fn error_nodes(mut cursor: TreeCursor) -> Vec<Node> {
    fn helper<'a>(ret: &mut Vec<Node<'a>>, cursor: &mut TreeCursor<'a>) {
        let node = cursor.node();
        if node.is_error() || node.is_missing() {
            ret.push(node);
        }

        if !cursor.goto_first_child() {
            return;
        }
        loop {
            helper(ret, cursor);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }

    let mut ret = vec![];
    helper(&mut ret, &mut cursor);
    ret
}

struct ParsedCode {
    parser: Parser,
    code: String,
    tree: Tree,
}

impl ParsedCode {
    fn new(lang: Language, code: String) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(lang)
            .expect("Error loading openscad grammar");
        let tree = parser.parse(&code, None).unwrap();
        Self { parser, code, tree }
    }

    fn edit(&mut self, events: &[TextDocumentContentChangeEvent]) {
        for event in events {
            let range = event.range.unwrap();
            let start_ofs = find_offset(&self.code, range.start).unwrap();
            let end_ofs = find_offset(&self.code, range.end).unwrap();
            self.code.replace_range(start_ofs..end_ofs, &event.text);

            let new_end_position = match event.text.rfind('\n') {
                Some(ind) => {
                    let num_newlines = event.text.bytes().filter(|&c| c == b'\n').count();
                    Point {
                        row: range.start.line as usize + num_newlines,
                        column: event.text.len() - ind,
                    }
                }
                None => Point {
                    row: range.end.line as usize,
                    column: range.end.character as usize + event.text.len(),
                },
            };

            self.tree.edit(&InputEdit {
                start_byte: start_ofs,
                old_end_byte: end_ofs,
                new_end_byte: start_ofs + event.text.len(),
                start_position: to_point(range.start),
                old_end_position: to_point(range.end),
                new_end_position,
            });
        }
        self.tree = self.parser.parse(&self.code, Some(&self.tree)).unwrap();
    }
}

enum LoopAction {
    Exit,
    Continue,
}

struct Server {
    builtins: Vec<Item>,
    connection: Connection,
    code: HashMap<Url, ParsedCode>,
}

// Message handlers.
impl Server {
    fn handle_hover(&mut self, id: RequestId, params: HoverParams) {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let file = match self.code.get(uri) {
            Some(x) => x,
            None => return,
        };

        let point = to_point(pos);
        let mut cursor = file.tree.root_node().walk();
        while cursor.goto_first_child_for_point(point).is_some() {}

        let node = cursor.node();
        let result = match node.kind() {
            "identifier" => {
                let items = match self.find_visible_items(&params.text_document_position_params) {
                    Ok(x) => x,
                    Err(_) => return,
                };

                let name = &file.code[node.start_byte()..node.end_byte()];
                self.builtins
                    .iter()
                    .chain(items.iter())
                    .find(|item| item.name == name)
                    .map(|item| Hover {
                        contents: HoverContents::Scalar(MarkedString::String(item.make_hover())),
                        range: None,
                    })
            }
            _ => None,
        };

        let result = result.map(|r| serde_json::to_value(&r).unwrap());
        self.respond(Response {
            id,
            result,
            error: None,
        });
    }

    fn handle_completion(&mut self, id: RequestId, params: CompletionParams) {
        let items = match self.find_visible_items(&params.text_document_position) {
            Ok(x) => x,
            Err(_) => return,
        };
        let result = CompletionResponse::Array(
            self.builtins
                .iter()
                .chain(items.iter())
                .map(|item| CompletionItem {
                    label: item.name.to_owned(),
                    kind: Some(item.kind.completion_kind()),
                    insert_text: Some(item.make_snippet()),
                    insert_text_format: Some(InsertTextFormat::Snippet),
                    insert_text_mode: Some(InsertTextMode::AdjustIndentation),
                    ..Default::default()
                })
                .collect(),
        );
        let result = serde_json::to_value(&result).unwrap();
        self.respond(Response {
            id,
            result: Some(result),
            error: None,
        });
    }

    fn handle_did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        let DidOpenTextDocumentParams { text_document: doc } = params;
        self.code.insert(
            doc.uri,
            ParsedCode::new(tree_sitter_openscad::language(), doc.text),
        );
    }

    fn handle_did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document,
            content_changes,
        } = params;

        let pc = match self.code.get_mut(&text_document.uri) {
            Some(x) => x,
            None => {
                eprintln!("unknown document {}", text_document.uri);
                return;
            }
        };
        pc.edit(&content_changes);

        let diags: Vec<_> = error_nodes(pc.tree.walk())
            .into_iter()
            .map(|node| Diagnostic {
                range: Range {
                    start: to_position(node.start_position()),
                    end: to_position(node.end_position()),
                },
                severity: Some(DiagnosticSeverity::Error),
                message: if node.is_missing() {
                    format!("missing {}", node.kind())
                } else {
                    "syntax error".to_owned()
                },
                ..Default::default()
            })
            .collect();

        self.notify(lsp_server::Notification::new(
            "textDocument/publishDiagnostics".into(),
            PublishDiagnosticsParams {
                uri: text_document.uri,
                diagnostics: diags,
                version: Some(text_document.version),
            },
        ));
    }
}

// Code-related helpers.
impl Server {
    fn find_visible_items(&self, params: &TextDocumentPositionParams) -> Result<Vec<Item>, String> {
        let uri = &params.text_document.uri;
        let pos = params.position;
        let file = match self.code.get(uri) {
            Some(x) => x,
            None => {
                return Err(format!("unknown file {:?}", uri));
            }
        };

        let point = to_point(pos);
        let mut cursor = file.tree.root_node().walk();
        while cursor.goto_first_child_for_point(point).is_some() {}

        let mut items = vec![];
        loop {
            if let Some(item) = Item::parse(&file.code, &cursor.node()) {
                match item.kind {
                    ItemKind::Module { params, .. } => {
                        for p in params {
                            items.push(Item {
                                name: p.name,
                                kind: ItemKind::Variable,
                            })
                        }
                    }
                    ItemKind::Function(params) => {
                        for p in params {
                            items.push(Item {
                                name: p.name,
                                kind: ItemKind::Variable,
                            })
                        }
                    }
                    _ => {}
                }
            }

            if cursor.goto_first_child() {
                loop {
                    let node = cursor.node();
                    if let Some(item) = Item::parse(&file.code, &node) {
                        items.push(item);
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }

            if !cursor.goto_parent() {
                break;
            }
        }
        Ok(items)
    }
}

// Miscellaneous high-level logic.
impl Server {
    fn read_builtins() -> Vec<Item> {
        let code = BUILTINS_SCAD.to_owned();
        let pc = ParsedCode::new(tree_sitter_openscad::language(), code.clone());
        let mut cursor: TreeCursor = pc.tree.walk();
        let mut ret = vec![];
        if cursor.goto_first_child() {
            loop {
                if let Some(item) = Item::parse(&code, &cursor.node()) {
                    ret.push(item);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        ret.extend(KEYWORDS.iter().map(|&(name, comp)| Item {
            name: name.to_owned(),
            kind: ItemKind::Keyword(comp.to_owned()),
        }));
        ret
    }

    fn new(connection: Connection) -> Self {
        Self {
            builtins: Self::read_builtins(),
            connection,
            code: Default::default(),
        }
    }

    fn notify(&self, notif: lsp_server::Notification) {
        self.connection
            .sender
            .send(Message::Notification(notif))
            .unwrap()
    }

    fn respond(&self, resp: Response) {
        self.connection
            .sender
            .send(Message::Response(resp))
            .unwrap()
    }

    fn handle_message(&mut self, msg: Message) -> Result<LoopAction, Box<dyn Error + Sync + Send>> {
        match msg {
            Message::Request(req) => {
                if self.connection.handle_shutdown(&req)? {
                    return Ok(LoopAction::Exit);
                }
                let req = match cast_request::<HoverRequest>(req) {
                    Ok((id, params)) => {
                        self.handle_hover(id, params);
                        return Ok(LoopAction::Continue);
                    }
                    Err(req) => req,
                };
                let req = match cast_request::<Completion>(req) {
                    Ok((id, params)) => {
                        self.handle_completion(id, params);
                        return Ok(LoopAction::Continue);
                    }
                    Err(req) => req,
                };
                eprintln!("unknown request: {:?}", req);
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(notif) => {
                let notif = match cast_notification::<DidOpenTextDocument>(notif) {
                    Ok(params) => {
                        self.handle_did_open_text_document(params);
                        return Ok(LoopAction::Continue);
                    }
                    Err(notif) => notif,
                };
                let notif = match cast_notification::<DidChangeTextDocument>(notif) {
                    Ok(params) => {
                        self.handle_did_change_text_document(params);
                        return Ok(LoopAction::Continue);
                    }
                    Err(notif) => notif,
                };
                let notif = match cast_notification::<DidSaveTextDocument>(notif) {
                    Ok(_) => return Ok(LoopAction::Continue),
                    Err(notif) => notif,
                };

                eprintln!("unknown notification: {:?}", notif);
            }
        }
        Ok(LoopAction::Continue)
    }

    fn main_loop(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        let caps = serde_json::to_value(&ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::Incremental,
            )),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(Default::default()),
            ..Default::default()
        })?;
        self.connection.initialize(caps)?;
        while let Ok(msg) = self.connection.receiver.recv() {
            match self.handle_message(msg)? {
                LoopAction::Continue => {}
                LoopAction::Exit => break,
            }
        }
        Ok(())
    }
}

fn cast_request<R>(req: Request) -> Result<(RequestId, R::Params), Request>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_notification<N>(
    notif: lsp_server::Notification,
) -> Result<N::Params, lsp_server::Notification>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    notif.extract(N::METHOD)
}
