use std::{collections::HashMap, error::Error};

use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument},
    request::{Completion, GotoDefinition},
    CompletionItem, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionParams,
    GotoDefinitionResponse, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree, TreeCursor};

const BUILTIN_FUNCTIONS: [&str; 39] = [
    "abs",
    "acos",
    "asin",
    "assert",
    "atan",
    "atan2",
    "ceil",
    "chr",
    "concat",
    "cos",
    "cross",
    "dxf_cross",
    "dxf_dim",
    "exp",
    "floor",
    "is_bool",
    "is_list",
    "is_num",
    "is_string",
    "is_undef",
    "len",
    "ln",
    "log",
    "lookup",
    "max",
    "min",
    "norm",
    "ord",
    "pow",
    "rands",
    "round",
    "search",
    "sign",
    "sin",
    "sqrt",
    "str",
    "tan",
    "version",
    "version_num",
];

const BUILTIN_MODULES: [&str; 36] = [
    "children",
    "circle",
    "color",
    "cube",
    "cylinder",
    "difference",
    "echo",
    "else",
    "for",
    "group",
    "hull",
    "if",
    "import",
    "intersection",
    "intersection_for",
    "let",
    "linear_extrude",
    "minkowski",
    "mirror",
    "multmatrix",
    "offset",
    "parent_module",
    "polygon",
    "polyhedron",
    "projection",
    "render",
    "resize",
    "rotate",
    "rotate_extrude",
    "scale",
    "sphere",
    "square",
    "surface",
    "text",
    "translate",
    "union",
];

const KEYWORDS: [&str; 7] = [
    "false", "function", "include", "module", "return", "true", "use",
];

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

fn error_nodes(mut cursor: TreeCursor) -> Vec<Node> {
    let mut ret = vec![];
    find_error_nodes(&mut ret, &mut cursor);
    ret
}

fn find_error_nodes<'a>(ret: &mut Vec<Node<'a>>, cursor: &mut TreeCursor<'a>) {
    let node = cursor.node();
    if node.is_error() || node.is_missing() {
        ret.push(node);
    }

    if !cursor.goto_first_child() {
        return;
    }
    loop {
        find_error_nodes(ret, cursor);
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    cursor.goto_parent();
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
                start_position: Point {
                    row: range.start.line as usize,
                    column: range.start.character as usize,
                },
                old_end_position: Point {
                    row: range.end.line as usize,
                    column: range.end.character as usize,
                },
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
    fn handle_goto_definition(&mut self, id: RequestId, _params: GotoDefinitionParams) {
        let result = Some(GotoDefinitionResponse::Array(Vec::new()));
        let result = serde_json::to_value(&result).unwrap();
        self.respond(Response {
            id,
            result: Some(result),
            error: None,
        });
    }

    fn handle_completion(&mut self, id: RequestId, params: CompletionParams) {
        let mut local_names = vec![];
        {
            let uri = params.text_document_position.text_document.uri;
            let pos = params.text_document_position.position;
            eprintln!("completion at {:?} {:?}", uri, pos);

            let file = match self.code.get(&uri) {
                Some(x) => x,
                None => {
                    eprintln!("unknown file {:?}", uri);
                    return;
                }
            };

            let point = Point {
                row: pos.line as usize,
                column: pos.character as usize,
            };
            let mut cursor = file.tree.root_node().walk();
            while cursor.goto_first_child_for_point(point).is_some() {}
            eprintln!("parents:");
            loop {
                eprintln!("- {}", node_debug(&file.code, &cursor));
                if cursor.goto_first_child() {
                    loop {
                        eprintln!("  - {}", node_debug(&file.code, &cursor));

                        let node = cursor.node();
                        match node.kind() {
                            "module_declaration" | "function_declaration" => {
                                if let Some(name) = node.child_by_field_name("name") {
                                    local_names.push(name.utf8_text(file.code.as_bytes()).unwrap());
                                }
                            }
                            "assignment" => {
                                if let Some(left) = node.child_by_field_name("left") {
                                    local_names.push(left.utf8_text(file.code.as_bytes()).unwrap());
                                }
                            }
                            _ => {}
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
            BUILTIN_FUNCTIONS
                .iter()
                .chain(BUILTIN_MODULES.iter())
                .chain(KEYWORDS.iter())
                .chain(local_names.iter())
                .map(|&s| CompletionItem {
                    label: s.to_owned(),
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

        eprintln!("text: {:?}", pc.code);

        show_node(&pc.code, &mut pc.tree.walk(), 0);

        let diags: Vec<_> = error_nodes(pc.tree.walk())
            .into_iter()
            .map(|node| Diagnostic {
                range: Range {
                    start: Position {
                        line: node.start_position().row as u32,
                        character: node.start_position().column as u32,
                    },
                    end: Position {
                        line: node.end_position().row as u32,
                        character: node.end_position().column as u32,
                    },
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
                    let req = match cast_request::<GotoDefinition>(req) {
                        Ok((id, params)) => {
                            self.handle_goto_definition(id, params);
                            continue;
                        }
                        Err(req) => req,
                    };
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
