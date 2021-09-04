use std::{collections::HashMap, error::Error};

use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument},
    request::{Completion, GotoDefinition},
    CompletionItem, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionResponse,
    InitializeParams, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
};
use tree_sitter::{Node, Parser, TreeCursor};

fn show_node(code: &str, cursor: &mut TreeCursor, depth: usize) {
    let node = cursor.node();
    if !node.is_named() {
        return;
    }

    eprintln!(
        "{} {} {} {} {:?}",
        "    ".repeat(depth),
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
    );

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
    let caps = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::Incremental,
        )),
        completion_provider: Some(Default::default()),
        ..Default::default()
    };
    let caps = serde_json::to_value(&caps).unwrap();
    let initialization_params = connection.initialize(caps)?;
    main_loop(&connection, initialization_params)?;
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

fn main_loop(
    connection: &Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let mut code = HashMap::new();

    for msg in &connection.receiver {
        eprintln!("got msg: {:?}", msg);
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let req = match cast_request::<GotoDefinition>(req) {
                    Ok((id, _params)) => {
                        let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(req) => req,
                };
                let req = match cast_request::<Completion>(req) {
                    Ok((id, _params)) => {
                        let result = CompletionResponse::Array(
                            "aaa abc ddd def ggg ghi 123 147"
                                .split(' ')
                                .map(|s| CompletionItem {
                                    label: s.to_owned(),
                                    ..Default::default()
                                })
                                .collect(),
                        );
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
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
                    Ok(DidOpenTextDocumentParams { text_document: doc }) => {
                        eprintln!("opened document:\n- {:?}\n- {:?}", doc.uri, doc.text);

                        let text = doc.text;
                        {
                            let mut parser = Parser::new();
                            parser
                                .set_language(tree_sitter_openscad::language())
                                .expect("Error loading openscad grammar");
                            let tree = parser.parse(&text, None).unwrap();

                            let mut cursor = tree.walk();

                            show_node(&text, &mut cursor, 0);
                            code.insert(doc.uri, text);
                        }
                        continue;
                    }
                    Err(notif) => notif,
                };
                let notif = match cast_notification::<DidChangeTextDocument>(notif) {
                    Ok(DidChangeTextDocumentParams {
                        text_document,
                        content_changes,
                    }) => {
                        let text = match code.get_mut(&text_document.uri) {
                            Some(r) => r,
                            None => {
                                eprintln!("unknown document {}", text_document.uri);
                                continue;
                            }
                        };
                        for event in content_changes {
                            let range = event.range.unwrap();
                            let start_ofs = find_offset(text, range.start).unwrap();
                            let end_ofs = find_offset(text, range.end).unwrap();
                            text.replace_range(start_ofs..end_ofs, &event.text);
                        }

                        let mut parser = Parser::new();
                        parser
                            .set_language(tree_sitter_openscad::language())
                            .expect("Error loading openscad grammar");
                        let tree = parser.parse(&text, None).unwrap();

                        let mut cursor = tree.walk();

                        show_node(text, &mut cursor, 0);

                        let diags = error_nodes(tree.walk()).into_iter().map(|node| Diagnostic {
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
                        });

                        connection
                            .sender
                            .send(Message::Notification(lsp_server::Notification::new(
                                "textDocument/publishDiagnostics".into(),
                                PublishDiagnosticsParams {
                                    uri: text_document.uri,
                                    diagnostics: diags.collect(),
                                    version: Some(text_document.version),
                                },
                            )))
                            .unwrap();

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
