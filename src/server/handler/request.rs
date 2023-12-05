use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Read, Write},
    process::{Command, Stdio},
    rc::Rc,
};

use lsp_server::{RequestId, Response, ResponseError};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionParams, CompletionResponse,
    DocumentFormattingParams, DocumentSymbolParams, DocumentSymbolResponse, Documentation,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams,
    InsertTextFormat, InsertTextMode, Location, MarkupContent, Range, RenameParams,
    SymbolInformation, TextEdit, WorkspaceEdit,
};

use tree_sitter_traversal::{traverse, Order};

use crate::{
    response_item::{Item, ItemKind},
    server::Server,
    utils::*,
};

// Request handlers.
impl Server {
    pub(crate) fn handle_rename(&mut self, id: RequestId, params: RenameParams) {
        let uri = params.text_document_position.text_document.uri;
        let ident_new_name = params.new_name;

        let file = match self.get_code(&uri) {
            Some(code) => code,
            _ => return,
        };
        file.borrow_mut().gen_top_level_items_if_needed();
        let bfile = file.borrow();

        let (ident_initial_name, parent_scope, ident_initial_node) = {
            let pos = params.text_document_position.position;
            let point = to_point(pos);
            let mut cursor = bfile.tree.root_node().walk();
            while cursor.goto_first_child_for_point(point).is_some() {}

            let node = cursor.node();

            if node.kind() != "identifier" {
                self.respond(Response {
                    id,
                    result: None,
                    error: Some(ResponseError {
                        code: -32600, // Invalid Request error
                        message: "No identifier at given position".to_string(),
                        data: None,
                    }),
                });
                return;
            }

            // unwrap here is fine because an identifier node should always have a parent scope
            let parent_scope = find_node_scope(node).unwrap();

            let kind = parent_scope.kind();
            let text = node_text(&bfile.code, &parent_scope);
            dbg!(text, kind);

            (node_text(&bfile.code, &node), parent_scope, node)
        };

        let mut node_iter = traverse(parent_scope.walk(), Order::Post);
        let mut changes = vec![];
        while let Some(node) = node_iter.next() {
            let is_identifier_instance =
                node.kind() != "identifier" || node_text(&bfile.code, &node) != ident_initial_name;
            if is_identifier_instance {
                continue;
            }

            let is_assignment = node
                .parent()
                .is_some_and(|node| node.kind() == "assignment");
            let is_assignment_in_subscope = is_assignment && node != ident_initial_node;
            if is_assignment_in_subscope {
                // Unwrap is ok because an identifier node whould always have a parent scope.
                let scope = find_node_scope(node).unwrap();
                // Consume iterator until it reaches the parent scope
                while node_iter.next().is_some_and(|next| scope != next) {}
                continue;
            }

            changes.push(TextEdit {
                range: Range {
                    start: to_position(node.start_position()),
                    end: to_position(node.end_position()),
                },
                new_text: ident_new_name.to_string(),
            });
        }

        let result = WorkspaceEdit {
            changes: Some({
                let mut h = HashMap::new();
                h.insert(uri, changes);
                h
            }),
            ..Default::default()
        };

        self.respond(Response {
            id,
            result: Some(serde_json::to_value(result).unwrap()),
            error: None,
        });
    }
    pub(crate) fn handle_hover(&mut self, id: RequestId, params: HoverParams) {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        file.borrow_mut().gen_top_level_items_if_needed();

        let point = to_point(pos);
        let bfile = file.borrow();
        let mut cursor = bfile.tree.root_node().walk();
        while cursor.goto_first_child_for_point(point).is_some() {}

        let node = cursor.node();

        let kind = node.kind();
        let name = String::from(node_text(&bfile.code, &node));

        let result = match kind {
            "identifier" => {
                let items = self.find_identities(
                    &file.borrow(),
                    &|item_name| item_name == name,
                    &node,
                    false,
                    0,
                );
                items.first().map(|item| Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value: item.borrow_mut().get_hover(),
                    }),
                    range: None,
                })
            }
            _ => None,
        };

        let result = result.map(|r| serde_json::to_value(r).unwrap());
        self.respond(Response {
            id,
            result,
            error: None,
        });
    }

    pub(crate) fn handle_definition(&mut self, id: RequestId, params: GotoDefinitionParams) {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        file.borrow_mut().gen_top_level_items_if_needed();

        let point = to_point(pos);
        let bfile = file.borrow();
        let mut cursor = bfile.tree.root_node().walk();
        while cursor.goto_first_child_for_point(point).is_some() {}

        let node = cursor.node();

        let kind = node.kind();
        let name = String::from(node_text(&bfile.code, &node));

        let result = match kind {
            "identifier" => {
                let items = self.find_identities(
                    &file.borrow(),
                    &|item_name| item_name == name,
                    &node,
                    false,
                    0,
                );
                let locs = items
                    .iter()
                    .filter(|item| item.borrow().name == name && item.borrow().url.is_some())
                    .map(|item| Location {
                        uri: item.borrow().url.as_ref().unwrap().clone(),
                        range: item.borrow().range,
                    })
                    .collect::<Vec<Location>>();
                Some(locs)
            }
            "include_path" => {
                let mut res = None;
                if let Some(incs) = &(file.borrow().includes) {
                    let include_path = name
                        .trim_start_matches(&['<', '\n'][..])
                        .trim_end_matches(&['>', '\n'][..]);

                    let mut inciter = incs.iter();
                    let loc = loop {
                        if let Some(url) = inciter.next() {
                            if url.path().ends_with(include_path) {
                                break Some(Location {
                                    uri: url.clone(),
                                    range: Range::default(),
                                });
                            }
                        } else {
                            break None;
                        }
                    };

                    if let Some(v) = loc {
                        res = Some(vec![v]);
                    }
                };
                res
            }
            _ => None,
        };

        let result = result.map(GotoDefinitionResponse::Array);
        let result = serde_json::to_value(result).unwrap();

        self.respond(Response {
            id,
            result: Some(result),
            error: None,
        });
    }

    pub(crate) fn handle_completion(&mut self, id: RequestId, params: CompletionParams) {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        file.borrow_mut().gen_top_level_items_if_needed();

        let mut point = to_point(pos);

        if point.column > 0 {
            point.column -= 1;
        } else {
            point.row -= 1;
        }

        let bfile = file.borrow();
        let mut cursor = bfile.tree.root_node().walk();

        while cursor.goto_first_child_for_point(point).is_some() {}

        let node = cursor.node();
        let name = node_text(&bfile.code, &node);

        let mut items = self.find_identities(&file.borrow(), &|_| true, &node, true, 0);

        let kind = node.kind();
        if let Some(parent) = &node.parent().and_then(|parent| parent.parent()) {
            let kind = parent.kind();
            let mut node = None;
            if kind == "arguments" {
                if let Some(callable) = parent.parent() {
                    let kind = callable.kind();
                    if kind == "module_call" || kind == "function_call" {
                        node = Some(callable);
                    }
                }
            }

            if kind == "module_call" || kind == "function_call" {
                node = Some(*parent);
            }

            if let Some(node) = node {
                node.child_by_field_name("name")
                    .map(|child| node_text(&bfile.code, &child))
                    .map(|name| {
                        let fun_items = self.find_identities(
                            &file.borrow(),
                            &|item_name| item_name == name,
                            &node,
                            false,
                            0,
                        );

                        if !fun_items.is_empty() {
                            let item = &fun_items[0];

                            let param_items = match &item.borrow().kind {
                                ItemKind::Module { params, .. } => {
                                    let mut result = vec![];
                                    for p in params {
                                        result.push(Rc::new(RefCell::new(Item {
                                            name: p.name.clone(),
                                            kind: ItemKind::Variable,
                                            range: p.range,
                                            url: Some(bfile.url.clone()),
                                            ..Default::default()
                                        })));
                                    }
                                    result
                                }
                                ItemKind::Function { flags: _, params } => {
                                    let mut result = vec![];
                                    for p in params {
                                        result.push(Rc::new(RefCell::new(Item {
                                            name: p.name.clone(),
                                            kind: ItemKind::Variable,
                                            range: p.range,
                                            url: Some(bfile.url.clone()),
                                            ..Default::default()
                                        })));
                                    }
                                    result
                                }
                                _ => {
                                    vec![]
                                }
                            };

                            items.extend(param_items);
                        }
                    });
            }
        }

        let result = if kind == "include_path"
            || node
                .prev_sibling()
                .map(|sib| {
                    if sib.kind() == "include" || sib.kind() == "use" {
                        Some(true)
                    } else {
                        None
                    }
                })
                .is_some()
        {
            CompletionResponse::List(CompletionList {
                is_incomplete: true,
                items: bfile
                    .get_include_completion(&node)
                    .iter()
                    .map(|file_name| CompletionItem {
                        label: file_name.clone(),
                        kind: Some(CompletionItemKind::FILE),
                        filter_text: Some(name.to_owned()),
                        insert_text: Some(file_name.clone()),
                        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                        insert_text_mode: Some(InsertTextMode::ADJUST_INDENTATION),
                        ..Default::default()
                    })
                    .collect(),
            })
        } else {
            CompletionResponse::List(CompletionList {
                is_incomplete: true,
                items: items
                    .iter()
                    .map(|item| {
                        let label = item.borrow_mut().get_label();
                        let snippet = item.borrow_mut().get_snippet();
                        CompletionItem {
                            label,
                            kind: Some(item.borrow().kind.completion_kind()),
                            filter_text: Some(item.borrow().name.to_owned()),
                            insert_text: Some(snippet),
                            insert_text_format: Some(match item.borrow().kind {
                                ItemKind::Variable => InsertTextFormat::PLAIN_TEXT,
                                _ => InsertTextFormat::SNIPPET,
                            }),
                            insert_text_mode: Some(InsertTextMode::ADJUST_INDENTATION),
                            documentation: item.borrow().hover.as_ref().map(|doc| {
                                Documentation::MarkupContent(MarkupContent {
                                    kind: lsp_types::MarkupKind::Markdown,
                                    value: doc.to_owned(),
                                })
                            }),
                            ..Default::default()
                        }
                    })
                    .collect(),
            })
        };

        let result = serde_json::to_value(result).unwrap();
        self.respond(Response {
            id,
            result: Some(result),
            error: None,
        });
    }

    pub(crate) fn handle_document_symbols(&mut self, id: RequestId, params: DocumentSymbolParams) {
        let uri = &params.text_document.uri;
        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        let mut bfile = file.borrow_mut();
        bfile.gen_top_level_items_if_needed();
        if let Some(items) = &bfile.root_items {
            let result: Vec<SymbolInformation> = items
                .iter()
                .filter_map(|item| {
                    item.borrow().url.as_ref().map(|url| {
                        #[allow(deprecated)]
                        SymbolInformation {
                            name: item.borrow().name.to_owned(),
                            kind: item.borrow().get_symbol_kind(),
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: url.clone(),
                                range: item.borrow().range,
                            },
                            container_name: None,
                        }
                    })
                })
                .collect();

            let result = DocumentSymbolResponse::Flat(result);

            let result = serde_json::to_value(result).unwrap();
            self.respond(Response {
                id,
                result: Some(result),
                error: None,
            });
        }
    }

    pub(crate) fn handle_formatting(&mut self, id: RequestId, params: DocumentFormattingParams) {
        let uri = &params.text_document.uri;

        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        let internal_err = |err: String| {
            self.respond(Response {
                id: id.clone(),
                result: None,
                error: Some(ResponseError {
                    code: -32603,
                    message: err,
                    data: None,
                }),
            });
        };

        let mut code = String::new();
        let mut last_pos = 0;
        for_each_child(&mut (file.borrow().tree.walk()), |cursor| {
            let node = cursor.node();

            let code_str = &file.borrow().code;

            if node.start_byte() > last_pos {
                let mut sub = &code_str[last_pos..node.start_byte()];
                sub = sub.trim_matches(' ');
                sub = sub.trim_matches('\t');
                code.push_str(sub);
            }

            if node.kind().is_include_statement() {
                code.push_str("#include <");
            }
            code.push_str(node_text(code_str, &node));

            last_pos = node.end_byte();
        });

        let path = uri.to_file_path().unwrap();
        let path = path.parent().unwrap();

        let child = match Command::new(&self.args.fmt_exe)
            .arg(format!("-style={}", self.args.fmt_style))
            .arg("-assume-filename=foo.scad")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(path)
            .spawn()
        {
            Ok(res) => res,
            Err(err) => {
                internal_err(format!("{}: {}", &self.args.fmt_exe, &err.to_string()));
                return;
            }
        };

        if let Err(why) = child.stdin.unwrap().write_all(code.as_bytes()) {
            internal_err(why.to_string());
            return;
        }

        let mut code = String::new();

        match child.stdout.unwrap().read_to_string(&mut code) {
            Err(why) => {
                internal_err(why.to_string());
            }
            Ok(size) => {
                if size > 0 {
                    code = code.replace("#include <", "");
                    let result = [TextEdit {
                        range: file.borrow().tree.root_node().lsp_range(),
                        new_text: code.to_owned(),
                    }];

                    let result = serde_json::to_value(result).unwrap();
                    self.respond(Response {
                        id,
                        result: Some(result),
                        error: None,
                    });
                }
            }
        }
    }
}
