use std::{collections::HashMap, error::Error};

use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument},
    request::Completion,
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams, InsertTextFormat,
    InsertTextMode, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree, TreeCursor};

#[derive(Clone, Debug)]
enum ItemKind {
    Variable,
    Function,
    Keyword,
    LeafModule(String),
    GroupModule(String),
}

impl ItemKind {
    fn completion_kind(&self) -> CompletionItemKind {
        match self {
            ItemKind::Variable => CompletionItemKind::Variable,
            ItemKind::Function => CompletionItemKind::Function,
            ItemKind::Keyword => CompletionItemKind::Keyword,
            ItemKind::LeafModule(_) => CompletionItemKind::Module,
            ItemKind::GroupModule(_) => CompletionItemKind::Module,
        }
    }
}

struct Item<'a> {
    name: &'a str,
    kind: ItemKind,
}

impl<'a> Item<'a> {
    fn new(name: &'a str, kind: ItemKind) -> Self {
        Self { name, kind }
    }

    fn make_snippet(&self) -> String {
        match self.kind {
            ItemKind::Variable => self.name.to_owned(),
            ItemKind::Function => format!("{}($0)", self.name),
            ItemKind::Keyword => self.name.to_owned(),
            ItemKind::LeafModule(ref args) => format!("{}({});$0", self.name, args),
            ItemKind::GroupModule(ref args) => format!("{}({}) {{\n  $0\n}}", self.name, args),
        }
    }
}

lazy_static::lazy_static! {
    static ref BUILTINS: Vec<Item<'static>> = vec![
        // Leaf modules.
        Item::new("children", ItemKind::LeafModule("$1".to_owned())),
        Item::new("circle", ItemKind::LeafModule("$1".to_owned())),
        Item::new("cube", ItemKind::LeafModule("[${1:DX}, ${2:DY}, ${3:DZ}]".to_owned())),
        Item::new("cylinder", ItemKind::LeafModule("$1".to_owned())),
        Item::new("polygon", ItemKind::LeafModule("[[$1]]".to_owned())),
        Item::new("polyhedron", ItemKind::LeafModule("$1".to_owned())),
        Item::new("sphere", ItemKind::LeafModule("${1:RADIUS}".to_owned())),
        Item::new("square", ItemKind::LeafModule("[${1:DX}, ${2:DY}]".to_owned())),
        Item::new("surface", ItemKind::LeafModule("$1".to_owned())),
        Item::new("text", ItemKind::LeafModule("$1".to_owned())),
        // Group modules.
        Item::new("color", ItemKind::GroupModule("".to_owned())),
        Item::new("difference", ItemKind::GroupModule("".to_owned())),
        Item::new("echo", ItemKind::GroupModule("".to_owned())),
        Item::new("else", ItemKind::GroupModule("".to_owned())),
        Item::new("for", ItemKind::GroupModule("".to_owned())),
        Item::new("group", ItemKind::GroupModule("".to_owned())),
        Item::new("hull", ItemKind::GroupModule("".to_owned())),
        Item::new("if", ItemKind::GroupModule("${1:COND}".to_owned())),
        Item::new("import", ItemKind::GroupModule("".to_owned())),
        Item::new("intersection", ItemKind::GroupModule("".to_owned())),
        Item::new("intersection_for", ItemKind::GroupModule("${1:ARGS}".to_owned())),
        Item::new("let", ItemKind::GroupModule("${1:ARGS}".to_owned())),
        Item::new("linear_extrude", ItemKind::GroupModule("${1:ARGS}".to_owned())),
        Item::new("minkowski", ItemKind::GroupModule("".to_owned())),
        Item::new("mirror", ItemKind::GroupModule("${1:NORMAL}".to_owned())),
        Item::new("multmatrix", ItemKind::GroupModule("${1:MATRIX}".to_owned())),
        Item::new("offset", ItemKind::GroupModule("".to_owned())),
        Item::new("parent_module", ItemKind::GroupModule("".to_owned())),
        Item::new("projection", ItemKind::GroupModule("".to_owned())),
        Item::new("render", ItemKind::GroupModule("".to_owned())),
        Item::new("resize", ItemKind::GroupModule("".to_owned())),
        Item::new("rotate", ItemKind::GroupModule("${1:ARGS}".to_owned())),
        Item::new("rotate_extrude", ItemKind::GroupModule("${1:ARGS}".to_owned())),
        Item::new("scale", ItemKind::GroupModule("${1:ARGS}".to_owned())),
        Item::new("translate", ItemKind::GroupModule("[${1:DX}, ${2:DY}, ${3:DZ}]".to_owned())),
        Item::new("union", ItemKind::GroupModule("".to_owned())),
        // Keywords.
        Item::new("false", ItemKind::Keyword),
        Item::new("function", ItemKind::Keyword),
        Item::new("include", ItemKind::Keyword),
        Item::new("module", ItemKind::Keyword),
        Item::new("return", ItemKind::Keyword),
        Item::new("true", ItemKind::Keyword),
        Item::new("use", ItemKind::Keyword),
        // Functions.
        Item::new("abs", ItemKind::Function),
        Item::new("acos", ItemKind::Function),
        Item::new("asin", ItemKind::Function),
        Item::new("assert", ItemKind::Function),
        Item::new("atan", ItemKind::Function),
        Item::new("atan2", ItemKind::Function),
        Item::new("ceil", ItemKind::Function),
        Item::new("chr", ItemKind::Function),
        Item::new("concat", ItemKind::Function),
        Item::new("cos", ItemKind::Function),
        Item::new("cross", ItemKind::Function),
        Item::new("dxf_cross", ItemKind::Function),
        Item::new("dxf_dim", ItemKind::Function),
        Item::new("exp", ItemKind::Function),
        Item::new("floor", ItemKind::Function),
        Item::new("is_bool", ItemKind::Function),
        Item::new("is_list", ItemKind::Function),
        Item::new("is_num", ItemKind::Function),
        Item::new("is_string", ItemKind::Function),
        Item::new("is_undef", ItemKind::Function),
        Item::new("len", ItemKind::Function),
        Item::new("ln", ItemKind::Function),
        Item::new("log", ItemKind::Function),
        Item::new("lookup", ItemKind::Function),
        Item::new("max", ItemKind::Function),
        Item::new("min", ItemKind::Function),
        Item::new("norm", ItemKind::Function),
        Item::new("ord", ItemKind::Function),
        Item::new("pow", ItemKind::Function),
        Item::new("rands", ItemKind::Function),
        Item::new("round", ItemKind::Function),
        Item::new("search", ItemKind::Function),
        Item::new("sign", ItemKind::Function),
        Item::new("sin", ItemKind::Function),
        Item::new("sqrt", ItemKind::Function),
        Item::new("str", ItemKind::Function),
        Item::new("tan", ItemKind::Function),
        Item::new("version", ItemKind::Function),
        Item::new("version_num", ItemKind::Function),
    ];
}

fn node_debug(code: &str, cursor: &TreeCursor) -> String {
    let node = cursor.node();
    format!(
        "{} {} {} {:?}",
        cursor.field_name().unwrap_or(if node.is_missing() {
            "MISSING"
        } else if node.is_error() {
            "ERROR"
        } else {
            "<none>"
        }),
        cursor.field_id().unwrap_or(u16::MAX),
        node.kind(),
        &code[node.start_byte()..node.end_byte().min(node.start_byte() + 32)],
    )
}

fn show_node(code: &str, cursor: &mut TreeCursor, depth: usize) {
    let node = cursor.node();
    if !node.is_named() {
        return;
    }

    eprintln!("{}{}", "    ".repeat(depth), node_debug(code, cursor));

    if !cursor.goto_first_child() {
        return;
    }
    loop {
        show_node(code, cursor, depth + 1);
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    cursor.goto_parent();
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

struct Server {
    connection: Connection,
    code: HashMap<Url, ParsedCode>,
}

// Message handlers.
impl Server {
    fn handle_completion(&mut self, id: RequestId, params: CompletionParams) {
        let mut local_items = vec![];

        {
            let uri = params.text_document_position.text_document.uri;
            let pos = params.text_document_position.position;
            let file = match self.code.get(&uri) {
                Some(x) => x,
                None => {
                    eprintln!("unknown file {:?}", uri);
                    return;
                }
            };

            let point = to_point(pos);
            let mut cursor = file.tree.root_node().walk();
            while cursor.goto_first_child_for_point(point).is_some() {}
            loop {
                if cursor.goto_first_child() {
                    loop {
                        let node = cursor.node();
                        let extract_info = match node.kind() {
                            "module_declaration" => {
                                Some(("name", ItemKind::LeafModule("$0".to_owned())))
                            }
                            "function_declaration" => Some(("name", ItemKind::Function)),
                            "assignment" => Some(("left", ItemKind::Variable)),
                            _ => None,
                        };
                        if let Some((child, kind)) = extract_info {
                            if let Some(child) = node.child_by_field_name(child) {
                                local_items.push(Item::new(
                                    &file.code[child.start_byte()..child.end_byte()],
                                    kind,
                                ));
                            }
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
        }
        let result = CompletionResponse::Array(
            BUILTINS
                .iter()
                .chain(local_items.iter())
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

        show_node(&pc.code, &mut pc.tree.walk(), 0);

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

impl Server {
    fn new(connection: Connection) -> Self {
        Self {
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

    fn main_loop(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        let caps = serde_json::to_value(&ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::Incremental,
            )),
            completion_provider: Some(Default::default()),
            ..Default::default()
        })?;
        self.connection.initialize(caps)?;

        while let Ok(msg) = self.connection.receiver.recv() {
            eprintln!("got msg: {:?}", msg);
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    let req = match cast_request::<Completion>(req) {
                        Ok((id, params)) => {
                            self.handle_completion(id, params);
                            continue;
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
                            continue;
                        }
                        Err(notif) => notif,
                    };
                    let notif = match cast_notification::<DidChangeTextDocument>(notif) {
                        Ok(params) => {
                            self.handle_did_change_text_document(params);
                            continue;
                        }
                        Err(notif) => notif,
                    };
                    let notif = match cast_notification::<DidSaveTextDocument>(notif) {
                        Ok(_) => continue,
                        Err(notif) => notif,
                    };

                    eprintln!("unknown notification: {:?}", notif);
                }
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
