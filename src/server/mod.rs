#[macro_use]
pub(crate) mod utils;
pub(crate) mod code_helper;
pub(crate) mod handler;
pub(crate) mod parse_code;
pub(crate) mod response_item;

use std::error::Error;
use std::fs::read_to_string;
use std::{cell::RefCell, env, path::PathBuf, rc::Rc};
use directories::UserDirs

use linked_hash_map::LinkedHashMap;
use lsp_server::Connection;
use lsp_types::{
    HoverProviderCapability, OneOf, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};

use crate::parse_code::ParsedCode;
use crate::Cli;

const BUILTINS_SCAD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/builtins"));
const BUILTIN_PATH: &str = "/builtin";

pub(crate) struct Server {
    pub library_locations: Rc<RefCell<Vec<Url>>>,

    pub connection: Connection,
    pub codes: LinkedHashMap<Url, Rc<RefCell<ParsedCode>>>,
    pub args: Cli,

    builtin_url: Url,
}

pub(crate) enum LoopAction {
    Exit,
    Continue,
}

static mut GLOBAL_SERVER: Option<Server> = None;

// Miscellaneous high-level logic.
impl Server {
    pub(crate) fn create_server(connection: Connection, args: Cli) {
        unsafe {
            GLOBAL_SERVER = Some(Server::new(connection, args));
        }
    }

    pub(crate) fn get_server<'a>() -> &'a mut Server {
        unsafe {
            return GLOBAL_SERVER.as_mut().unwrap();
        }
    }

    fn new(connection: Connection, args: Cli) -> Self {
        let builtin_path = PathBuf::from(&args.builtin);

        let mut args = args;

        let mut code = BUILTINS_SCAD.to_owned();

        let mut external = false;
        match read_to_string(builtin_path) {
            Err(err) => {
                err_to_console!("read external builtin file error: {:?}", err);
                args.builtin = BUILTIN_PATH.to_owned();
            }
            Ok(builtin_str) => {
                code = builtin_str;
                external = true;
            }
        }

        let url = Url::parse(&format!("file://{}", &args.builtin)).unwrap();

        let mut instance = Self {
            library_locations: Rc::new(RefCell::new(vec![])),
            connection,
            codes: Default::default(),
            args,
            builtin_url: url.to_owned(),
        };
        let rc = instance.insert_code(url, code);

        rc.borrow_mut().is_builtin = true;
        rc.borrow_mut().external_builtin = external;

        instance.make_library_locations();

        instance
    }

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
            if let Some(userdir) = UserDirs::new() {
                return userdir.document_dir()?.join("\\OpenSCAD\\libraries\\").into_os_string().into_string().ok();
            } 
            else {
                "My Documents\\OpenSCAD\\libraries\\"
            }
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
