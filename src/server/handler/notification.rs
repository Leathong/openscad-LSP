use std::{env, path::PathBuf};

use lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    PublishDiagnosticsParams,
};
use serde::Deserialize;

use crate::{server::Server, utils::*};

// Notification handlers.
impl Server {
    pub(crate) fn handle_did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        let DidOpenTextDocumentParams { text_document: doc } = params;
        if self.codes.contains_key(&doc.uri) {
            return;
        }
        self.insert_code(doc.uri, doc.text);
    }

    pub(crate) fn handle_did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document,
            content_changes,
        } = params;

        let pc = match self.codes.get_refresh(&text_document.uri) {
            Some(x) => x,
            None => {
                err_to_console!("unknown document {}", text_document.uri);
                return;
            }
        };

        pc.borrow_mut().edit(&content_changes);

        let mut diags: Vec<_> = error_nodes(pc.borrow().tree.walk())
            .into_iter()
            .map(|node| Diagnostic {
                range: node.lsp_range(),
                severity: Some(DiagnosticSeverity::ERROR),
                message: if node.is_missing() {
                    format!("missing {}", node.kind())
                } else {
                    "syntax error".to_owned()
                },
                ..Default::default()
            })
            .collect();

        if content_changes.len() == 1 {
            if let Some(range) = content_changes[0].range {
                let bpc = pc.borrow();
                let pos = to_point(range.start);
                let mut cursor = bpc.tree.root_node().walk();
                cursor.goto_first_child_for_point(pos);
                let node = cursor.node();
                let kind = node.kind();
                // let text = node_text(&bpc.code, &node);

                if kind.is_include_statement() && bpc.get_include_url(&node).is_none() {
                    let mut range = node.child(1).unwrap().lsp_range();
                    range.start.character += 1;
                    range.end.character -= 1;
                    diags.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "file not found!".to_owned(),
                        ..Default::default()
                    });
                }
            }
        }

        self.notify(lsp_server::Notification::new(
            "textDocument/publishDiagnostics".into(),
            PublishDiagnosticsParams {
                uri: text_document.uri,
                diagnostics: diags,
                version: Some(text_document.version),
            },
        ));
    }

    pub(crate) fn handle_did_change_config(&mut self, params: DidChangeConfigurationParams) {
        #[derive(Deserialize)]
        pub(crate) struct Openscad {
            search_paths: Option<String>,
            default_param: Option<bool>,
            indent: Option<String>,
            query_file: Option<PathBuf>,
        }

        #[derive(Deserialize)]
        pub(crate) struct Settings {
            openscad: Openscad,
        }

        let settings = match serde_json::from_value::<Settings>(params.settings) {
            Ok(settings) => Some(settings),
            Err(err) => {
                err_to_console!("{}", err.to_string());
                return;
            }
        };

        if let Some(settings) = settings {
            // self.extend_libs(settings.search_paths);
            let paths: Vec<String> = settings
                .openscad
                .search_paths
                .map(|paths| {
                    env::split_paths(&paths)
                        .filter_map(|buf| buf.into_os_string().into_string().ok())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            self.extend_libs(paths);

            if let Some(default_param) = settings.openscad.default_param {
                self.args.ignore_default = !default_param;
            }

            if self.args.indent.is_none() {
                self.args.indent = settings.openscad.indent;
            }

            if self.args.query_file.is_none() {
                self.fmt_query = Self::get_fmt_query(settings.openscad.query_file.clone());
            }
        }
    }

    pub(crate) fn handle_did_save_text_document(&mut self, _params: DidSaveTextDocumentParams) {}

    pub(crate) fn handle_did_close_text_document(&mut self, _params: DidCloseTextDocumentParams) {}
}
