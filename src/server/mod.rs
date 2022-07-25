#[macro_use]
pub(crate) mod utils;
pub(crate) mod code_helper;
pub(crate) mod handler;
pub(crate) mod parse_code;
pub(crate) mod response_item;

use std::error::Error;
use std::fs::read_to_string;
use std::{cell::RefCell, env, path::PathBuf, rc::Rc};

use linked_hash_map::LinkedHashMap;
use lsp_server::{Connection, ExtractError, Message, Response};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        DidSaveTextDocument,
    },
    request::{Completion, DocumentSymbolRequest, Formatting, GotoDefinition, HoverRequest},
    HoverProviderCapability, OneOf, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};

use serde_json::json;

use crate::parse_code::ParsedCode;
use crate::utils::*;
use crate::Cli;

const BUILTINS_SCAD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/builtins.scad"));
pub(crate) const BUILTIN_PATH: &str = "file://builtin";

pub(crate) struct Server {
    pub library_locations: Rc<RefCell<Vec<Url>>>,

    pub connection: Connection,
    pub code: LinkedHashMap<Url, Rc<RefCell<ParsedCode>>>,
    pub args: Cli,
}

pub(crate) enum LoopAction {
    Exit,
    Continue,
}

// Miscellaneous high-level logic.
impl Server {
    pub(crate) fn user_defined_library_locations() -> Vec<String> {
        match env::var("OPENSCADPATH") {
            Ok(path) => env::split_paths(&path)
                .filter_map(|buf| buf.into_os_string().into_string().ok())
                .collect(),
            Err(_) => vec![],
        }
    }

    pub(crate) fn built_in_library_location() -> Option<String> {
        let user_library_rel_path = if cfg!(target_os = "windows") {
            "My Documents\\OpenSCAD\\libraries\\"
        } else if cfg!(target_os = "macos") {
            "Documents/OpenSCAD/libraries/"
        } else {
            ".local/share/OpenSCAD/libraries/"
        };
        home::home_dir()?
            .join(user_library_rel_path)
            .into_os_string()
            .into_string()
            .ok()
    }

    pub(crate) fn installation_library_location() -> Option<String> {
        // TODO: Figure out the other cases.
        if cfg!(target_os = "windows") {
            Some("C:\\Program Files\\OpenSCAD\\libraries\\".into())
        } else if cfg!(target_os = "macos") {
            Some("/Applications/OpenSCAD.app/Contents/Resources/libraries/".into())
        } else {
            Some("/usr/share/openscad/libraries/".into())
        }
    }

    pub(crate) fn make_library_locations(&mut self) {
        let mut ret = Self::user_defined_library_locations();
        ret.extend(Self::built_in_library_location());
        ret.extend(Self::installation_library_location());

        self.extend_libs(ret);
    }

    pub(crate) fn extend_libs(&mut self, userlibs: Vec<String>) {
        let ret: Vec<Url> = userlibs
            .into_iter()
            .map(|lib| shellexpand::tilde(&lib).to_string())
            .filter_map(|p| {
                if p.is_empty() {
                    return None;
                }

                let mut path = format!("file://{}", p);
                if !path.ends_with('/') {
                    path.push('/');
                }

                if let Ok(uri) = Url::parse(&path) {
                    if let Ok(path) = uri.to_file_path() {
                        if path.exists() {
                            return Some(uri);
                        }
                    }
                };

                None
            })
            .collect();

        if !ret.is_empty() {
            eprintln!();
            log_to_console!("search paths:");

            for lib in ret {
                log_to_console!("{}", &lib);
                if !self.library_locations.borrow().contains(&lib) {
                    self.library_locations.borrow_mut().push(lib);
                }
            }

            eprintln!();
        }
    }

    pub(crate) fn new(connection: Connection, args: Cli) -> Self {
        let builtin_path = PathBuf::from(&args.builtin);

        let mut instance = Self {
            library_locations: Rc::new(RefCell::new(vec![])),
            connection,
            code: Default::default(),
            args,
        };
        let mut code = BUILTINS_SCAD.to_owned();
        let mut url = Url::parse(BUILTIN_PATH).unwrap();

        let mut external = false;
        match read_to_string(&builtin_path) {
            Err(err) => {
                err_to_console!("read external builtin file error: {:?}", err);
            }
            Ok(builtin_str) => {
                code = builtin_str;
                url = Url::parse(&format!("file://{}", &builtin_path.to_str().unwrap())).unwrap();
                external = true;
            }
        }

        let rc = instance.insert_code(Url::parse(BUILTIN_PATH).unwrap(), code);
        rc.borrow_mut().url = url;
        rc.borrow_mut().is_builtin = true;
        rc.borrow_mut().external_builtin = external;

        instance.make_library_locations();

        instance
    }

    pub(crate) fn notify(&self, notif: lsp_server::Notification) {
        self.connection
            .sender
            .send(Message::Notification(notif))
            .unwrap()
    }

    pub(crate) fn respond(&self, mut resp: Response) {
        // log_to_console!("{:?}\n\n", &resp);
        if resp.result.is_none() {
            resp.result = Some(json!("{}"))
        }
        self.connection
            .sender
            .send(Message::Response(resp))
            .unwrap()
    }

    pub(crate) fn handle_message(
        &mut self,
        msg: Message,
    ) -> Result<LoopAction, Box<dyn Error + Sync + Send>> {
        match msg {
            Message::Request(req) => {
                if self.connection.handle_shutdown(&req)? {
                    return Ok(LoopAction::Exit);
                }

                macro_rules! proc_req {
                    ($request:ident, $req_type:ty, $method:ident) => {
                        match cast_request::<$req_type>($request) {
                            Ok((id, params)) => {
                                self.$method(id, params);
                                return Ok(LoopAction::Continue);
                            }
                            Err(error) => match error {
                                ExtractError::MethodMismatch(req) => req,
                                ExtractError::JsonError { method, error } => {
                                    err_to_console!("method: {} error: {}\n", method, error);
                                    return Ok(LoopAction::Continue);
                                }
                            },
                        }
                    };
                }

                let req = proc_req!(req, HoverRequest, handle_hover);
                let req = proc_req!(req, Completion, handle_completion);
                let req = proc_req!(req, GotoDefinition, handle_definition);
                let req = proc_req!(req, DocumentSymbolRequest, handle_document_symbols);
                let req = proc_req!(req, Formatting, handle_formatting);
                err_to_console!("unknown request: {:?}", req);
            }
            Message::Response(resp) => {
                err_to_console!("got response: {:?}", resp);
            }
            Message::Notification(noti) => {
                macro_rules! proc {
                    ($noti:ident, $noti_type:ty, $method:ident) => {
                        match cast_notification::<$noti_type>($noti) {
                            Ok(params) => {
                                self.$method(params);
                                return Ok(LoopAction::Continue);
                            }
                            Err(error) => match error {
                                ExtractError::MethodMismatch(noti) => noti,
                                ExtractError::JsonError { method, error } => {
                                    err_to_console!("method: {} error: {}\n", method, error);
                                    return Ok(LoopAction::Exit);
                                }
                            },
                        }
                    };
                }

                let noti = proc!(noti, DidOpenTextDocument, handle_did_open_text_document);
                let noti = proc!(noti, DidChangeTextDocument, handle_did_change_text_document);
                let noti = proc!(noti, DidSaveTextDocument, handle_did_save_text_document);
                let noti = proc!(noti, DidCloseTextDocument, handle_did_close_text_document);
                let noti = proc!(noti, DidChangeConfiguration, handle_did_change_config);

                err_to_console!("unknown notification: {:?}", noti);
            }
        }
        Ok(LoopAction::Continue)
    }

    pub(crate) fn main_loop(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        let caps = serde_json::to_value(&ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            completion_provider: Some(Default::default()),
            definition_provider: Some(OneOf::Left(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
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
