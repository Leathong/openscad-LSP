#![allow(clippy::option_map_unit_fn)]

use std::{
    cell::RefCell,
    env,
    error::Error,
    fs::read_to_string,
    io::{self, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
    vec,
};

use lazy_static::lazy_static;
use linked_hash_map::LinkedHashMap;
use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response, ResponseError};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        DidSaveTextDocument,
    },
    request::{Completion, DocumentSymbolRequest, Formatting, GotoDefinition, HoverRequest},
    CompletionItem, CompletionItemKind, CompletionList, CompletionParams, CompletionResponse,
    Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentFormattingParams, DocumentSymbolParams, DocumentSymbolResponse, Documentation,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams,
    HoverProviderCapability, InsertTextFormat, InsertTextMode, Location, MarkupContent, OneOf,
    Position, PublishDiagnosticsParams, Range, ServerCapabilities, SymbolInformation, SymbolKind,
    TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
    Url,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use tree_sitter::{InputEdit, Language, Node, Point, Tree, TreeCursor};

use clap::Parser;

const BUILTINS_SCAD: &str = include_str!("builtins.scad");

const KEYWORDS: &[(&str, &str)] = &[
    ("else", "else {  $0\n}"),
    ("false", "false"),
    ("for", "for (${1:LOOP}) {\n  $0\n}"),
    ("function", "function ${1:NAME}(${2:ARGS}) = $0;"),
    ("if", "if (${1:COND}) {\n  $0\n}"),
    ("include", "include <${1:PATH}>$0"),
    ("intersection_for", "intersection_for(${1:LOOP}) {\n  $0\n}"),
    ("let", "let (${1:VARS}) $0"),
    ("module", "module ${1:NAME}(${2:ARGS}) {\n  $0\n}"),
    ("true", "true"),
    ("use", "use <${1:PATH}>;$0"),
];

const BUILTIN_PATH: &str = "file://builtin";

const LOG_PREFIX: &str = "[server] ";
const ERR_PREFIX: &str = "[error] ";

macro_rules! log_to_console {
    ($($arg:tt)*) => {
        eprint!("{}", LOG_PREFIX);
        eprintln!($($arg)*);
    };
}

macro_rules! err_to_console {
    ($($arg:tt)*) => {
        eprint!("{}", ERR_PREFIX);
        eprintln!($($arg)*);
    };
}

fn find_offset(text: &str, pos: Position) -> Option<usize> {
    let mut offset = 0;
    for _ in 0..pos.line {
        offset += text[offset..].find('\n')? + 1;
    }

    let mut chars = text[offset..].chars();
    for _ in 0..pos.character {
        offset += chars.next()?.len_utf8();
    }
    Some(offset)
}

fn to_point(p: Position) -> Point {
    Point {
        row: p.line as usize,
        column: p.character as usize,
    }
}

fn node_text<'a>(code: &'a str, node: &Node) -> &'a str {
    &code[node.byte_range()]
}

// The callback may move the cursor while executing, but it must always ultimately leave it in the
// same position it was in at the beginning.
fn for_each_child<'a>(cursor: &mut TreeCursor<'a>, mut cb: impl FnMut(&mut TreeCursor<'a>)) {
    if cursor.goto_first_child() {
        loop {
            cb(cursor);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

fn error_nodes(mut cursor: TreeCursor) -> Vec<Node> {
    fn helper<'a>(ret: &mut Vec<Node<'a>>, cursor: &mut TreeCursor<'a>) {
        let node = cursor.node();
        if node.is_error() || node.is_missing() {
            ret.push(node);
        }
        for_each_child(cursor, |cursor| {
            helper(ret, cursor);
        });
    }

    let mut ret = vec![];
    helper(&mut ret, &mut cursor);
    ret
}

fn cast_request<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
{
    req.extract(R::METHOD)
}

fn cast_notification<N>(
    notif: lsp_server::Notification,
) -> Result<N::Params, ExtractError<lsp_server::Notification>>
where
    N: lsp_types::notification::Notification,
{
    notif.extract(N::METHOD)
}

#[derive(Clone, Debug)]
struct Param {
    name: String,
    default: Option<String>,
    range: Range,
}

impl Param {
    fn parse_declaration(code: &str, node: &Node) -> Vec<Param> {
        node.children(&mut node.walk())
            .filter_map(|child| match child.kind() {
                "identifier" => Some(Param {
                    name: node_text(code, &child).to_owned(),
                    default: None,
                    range: child.lsp_range(),
                }),
                "assignment" => child.child_by_field_name("left").and_then(|left| {
                    child.child_by_field_name("right").map(|right| Param {
                        name: node_text(code, &left).to_owned(),
                        default: Some(node_text(code, &right).to_owned()),
                        range: right.lsp_range(),
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

enum ItemKind {
    Variable,
    Function(Vec<Param>),
    Keyword(String),
    Module { group: bool, params: Vec<Param> },
}

impl Default for ItemKind {
    fn default() -> Self {
        ItemKind::Variable
    }
}

impl ItemKind {
    fn completion_kind(&self) -> CompletionItemKind {
        match self {
            ItemKind::Variable => CompletionItemKind::VARIABLE,
            ItemKind::Function(_) => CompletionItemKind::FUNCTION,
            ItemKind::Keyword(_) => CompletionItemKind::KEYWORD,
            ItemKind::Module { .. } => CompletionItemKind::MODULE,
        }
    }
}

#[derive(Default)]
struct Item {
    name: String,
    kind: ItemKind,
    range: Range,
    url: Option<Url>,
    doc: Option<String>,
    hover: Option<String>,
    label: Option<String>,
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
                    format!("{}({}) $0", self.name, params)
                } else {
                    format!("{}({});$0", self.name, params)
                }
            }
        }
    }

    fn make_hover(&self) -> String {
        let mut label = match &self.label {
            Some(label) => label.to_owned(),
            None => self.make_label(),
        };
        label = match self.kind {
            ItemKind::Function(_) => format!("```scad\nfunction {}\n```", label),
            ItemKind::Module {
                group: _,
                params: _,
            } => format!("```scad\nmodule {}\n```", label),
            _ => format!("```scad\n{}\n```", label),
        };
        if let Some(doc) = &self.doc {
            label = format!("{}\n---\n\n{}\n", label, doc);
        }
        // print!("{}", &label);
        label
    }

    fn make_label(&self) -> String {
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

        let lable = match &self.kind {
            ItemKind::Variable => self.name.to_owned(),
            ItemKind::Function(params) => format!("{}({})", self.name, format_params(params)),
            ItemKind::Keyword(_) => self.name.clone(),
            ItemKind::Module { params, .. } => {
                format!("{}({})", self.name, format_params(params))
            }
        };

        lable
    }

    fn parse(code: &str, node: &Node) -> Option<Self> {
        let extract_name = |name| {
            node.child_by_field_name(name)
                .map(|child| node_text(code, &child).to_owned())
        };

        match node.kind() {
            "module_declaration" => {
                let group = if let Some(child) = node
                    .child_by_field_name("body")
                    .and_then(|body| body.named_child(0))
                {
                    let body = node_text(code, &child);
                    child.kind().is_comment() && (body == "/* group */" || body == "// group")
                } else {
                    false
                };
                Some(Self {
                    name: extract_name("name")?,
                    kind: ItemKind::Module {
                        group,
                        params: node
                            .child_by_field_name("parameters")
                            .map_or(vec![], |params| Param::parse_declaration(code, &params)),
                    },
                    range: node.lsp_range(),
                    ..Default::default()
                })
            }
            "function_declaration" => Some(Self {
                name: extract_name("name")?,
                kind: ItemKind::Function(
                    node.child_by_field_name("parameters")
                        .map_or(vec![], |params| Param::parse_declaration(code, &params)),
                ),
                range: node.lsp_range(),
                ..Default::default()
            }),
            "assignment" => Some(Self {
                name: extract_name("left")?,
                kind: ItemKind::Variable,
                range: node.lsp_range(),
                ..Default::default()
            }),
            _ => None,
        }
    }

    fn get_symbol_kind(&self) -> SymbolKind {
        match self.kind {
            ItemKind::Function(_) => SymbolKind::FUNCTION,
            ItemKind::Module {
                group: _,
                params: _,
            } => SymbolKind::MODULE,
            ItemKind::Variable => SymbolKind::VARIABLE,
            ItemKind::Keyword(_) => SymbolKind::KEY,
        }
    }
}

struct ParsedCode {
    parser: tree_sitter::Parser,
    code: String,
    tree: Tree,
    url: Url,
    root_items: Option<Vec<Rc<Item>>>,
    includes: Option<Vec<Url>>,
    is_builtin: bool,
    changed: bool,
    libs: Rc<RefCell<Vec<Url>>>,
}

trait KindExt {
    fn is_include_statement(&self) -> bool;
    fn is_comment(&self) -> bool;
}

impl KindExt for str {
    fn is_include_statement(&self) -> bool {
        self == "include_statement" || self == "use_statement"
    }

    fn is_comment(&self) -> bool {
        self == "comment"
    }
}

trait NodeExt {
    fn lsp_range(&self) -> Range;
}

impl NodeExt for Node<'_> {
    fn lsp_range(&self) -> Range {
        let r = self.range();
        Range {
            start: Position {
                line: r.start_point.row as u32,
                character: r.start_point.column as u32,
            },
            end: Position {
                line: r.end_point.row as u32,
                character: r.end_point.column as u32,
            },
        }
    }
}

impl ParsedCode {
    fn new(lang: Language, code: String, url: Url, libs: Rc<RefCell<Vec<Url>>>) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(lang)
            .expect("Error loading openscad grammar");
        let tree = parser.parse(&code, None).unwrap();
        Self {
            parser,
            code,
            tree,
            url,
            root_items: None,
            includes: None,
            is_builtin: false,
            libs,
            changed: true,
        }
    }

    fn edit(&mut self, events: &[TextDocumentContentChangeEvent]) {
        let mut old_tree = Some(&mut self.tree);
        for event in events {
            if let Some(range) = event.range {
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
                        row: range.start.line as usize,
                        column: range.start.character as usize + event.text.len(),
                    },
                };

                old_tree.as_mut().unwrap().edit(&InputEdit {
                    start_byte: start_ofs,
                    old_end_byte: end_ofs,
                    new_end_byte: start_ofs + event.text.len(),
                    start_position: to_point(range.start),
                    old_end_position: to_point(range.end),
                    new_end_position,
                });
            } else {
                old_tree = None;
                self.code = event.text.clone();
                break;
            }
        }

        let old_tree = old_tree.map(|t| &(*t));
        let new_tree = self.parser.parse(&self.code, old_tree).unwrap();
        self.tree = new_tree;

        self.changed = true;
    }

    fn gen_items_if_needed(&mut self) {
        if self.root_items.is_some() && !self.changed {
            return;
        }
        self.changed = false;
        self.gen_items();
    }

    fn extract_doc(&self, doc: &str) -> String {
        lazy_static! {
            static ref DOC_RE: Regex =
                Regex::new(r"(?m)(^\s*//+)|(^\s*/\*+\n?)|(^\s*\*+/)|(^\s*\*+)").unwrap();
        };

        DOC_RE
            .replace_all(doc, "")
            .replace(' ', "\u{00a0}")
            .replace('\n', "  \n")
    }

    fn gen_items(&mut self) {
        let mut cursor: TreeCursor = self.tree.walk();
        let mut ret: Vec<Item> = vec![];
        let mut inc = vec![];

        let mut doc: Option<String> = None;
        let mut doc_node: Option<Node> = None;
        let mut last_code_line: usize = 0;

        for_each_child(&mut cursor, |cursor| {
            let node = &cursor.node();
            if node.kind().is_comment() {
                if last_code_line > 0 && node.start_position().row == last_code_line {
                    let last = ret.last_mut().unwrap();
                    let doc_str = node_text(&self.code, node);
                    let newdoc = self.extract_doc(doc_str);

                    if last.doc.is_some() {
                        last.doc.as_mut().unwrap().push_str("  \n");
                        last.doc.as_mut().unwrap().push_str(&newdoc);
                    } else {
                        let mut doc = "".to_owned();
                        doc.push_str("  \n");
                        doc.push_str(&newdoc);
                        last.doc = Some(doc);
                    }
                    last.label = Some(last.make_label());
                    last.hover = Some(last.make_hover());
                    return;
                }

                if doc_node.is_some()
                    && node.end_position().row - doc_node.unwrap().end_position().row <= 1
                {
                    if let Some(doc_str) = &mut doc {
                        doc_str.push('\n');
                        doc_str.push_str(node_text(&self.code, node));
                    }
                } else {
                    doc = Some(node_text(&self.code, node).to_owned());
                }
                doc_node = Some(*node);
            } else {
                if let Some(mut item) = Item::parse(&self.code, node) {
                    item.url = Some(self.url.clone());
                    item.doc = doc.as_ref().map(|doc| self.extract_doc(doc));
                    item.label = Some(item.make_label());
                    item.hover = Some(item.make_hover());
                    last_code_line = item.range.start.line as usize;
                    ret.push(item);
                } else if node.kind().is_include_statement() {
                    self.get_include_url(node).map(|url| {
                        inc.push(url);
                    });
                }

                doc = None;
                doc_node = None;
            }
        });

        if self.is_builtin {
            ret.extend(KEYWORDS.iter().map(|&(name, comp)| Item {
                name: name.to_owned(),
                kind: ItemKind::Keyword(comp.to_owned()),
                ..Default::default()
            }));
        }

        let mut items = vec![];
        for item in ret {
            items.push(Rc::new(item));
        }

        self.root_items = Some(items);
        self.includes = Some(inc);
    }

    fn get_include_url(&self, incstat_node: &Node) -> Option<Url> {
        let mut res = None;
        let include_path = node_text(&self.code, &incstat_node.child(1).unwrap())
            .trim_start_matches(&['<', '\n'][..])
            .trim_end_matches(&['>', '\n'][..]);

        if include_path.is_empty() {
            return None;
        }

        let url = self.url.join(include_path).unwrap();
        if let Ok(path) = url.to_file_path() {
            if path.exists() {
                res = Some(url);
                return res;
            }
        }
        for lib in self.libs.borrow().iter() {
            let url = lib.join(include_path).unwrap();
            if let Ok(path) = url.to_file_path() {
                if path.exists() {
                    res = Some(url);
                    return res;
                }
            }
        }
        res
    }

    fn get_include_completion(&self, inc_path: &Node) -> Vec<String> {
        let mut result = vec![];
        let path = node_text(&self.code, inc_path)
            .trim_start_matches(&['<', '\n'][..])
            .trim_end_matches(&['>', '\n'][..]);

        let dir;
        let mut filename = String::from("");

        if path.ends_with('/') {
            dir = path;
        } else {
            let path_buf = PathBuf::from(path);
            path_buf.file_name().map(|name| {
                filename = String::from(name.to_str().unwrap());
            });
            dir = path.trim_end_matches(&filename);
        }

        let mut inc_dirs = vec![];
        let inc_dir = self.url.to_file_path().unwrap().parent().unwrap().join(dir);
        if inc_dir.exists() && inc_dir.is_dir() {
            inc_dirs.push(inc_dir);
        }

        for lib in self.libs.borrow().iter() {
            let dirpath = lib.join(dir).unwrap().to_file_path().unwrap();
            if dirpath.exists() && dirpath.is_dir() {
                inc_dirs.push(dirpath);
            }
        }

        for inc_dir in inc_dirs {
            if let Ok(paths) = inc_dir.read_dir() {
                for file in paths {
                    let name = file.as_ref().unwrap().file_name();
                    if name
                        .to_str()
                        .unwrap()
                        .to_lowercase()
                        .starts_with(&filename.to_lowercase())
                    {
                        if file.as_ref().unwrap().path().is_dir() {
                            result.push(String::from(name.to_str().unwrap()) + "/");
                        } else {
                            result.push(String::from(name.to_str().unwrap()));
                        }
                    }
                }
            }
        }

        result
    }
}

enum LoopAction {
    Exit,
    Continue,
}

struct Server {
    library_locations: Rc<RefCell<Vec<Url>>>,

    connection: Connection,
    code: LinkedHashMap<Url, Rc<RefCell<ParsedCode>>>,
    args: Cli,
}

// Request handlers.
impl Server {
    fn handle_hover(&mut self, id: RequestId, params: HoverParams) {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        file.borrow_mut().gen_items_if_needed();

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
                    true,
                );
                items.first().map(|item| Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value: match &item.hover {
                            Some(hover) => hover.to_owned(),
                            None => item.make_hover(),
                        },
                    }),
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

    fn handle_definition(&mut self, id: RequestId, params: GotoDefinitionParams) {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        file.borrow_mut().gen_items_if_needed();

        let point = to_point(pos);
        let bfile = file.borrow();
        let mut cursor = bfile.tree.root_node().walk();
        while cursor.goto_first_child_for_point(point).is_some() {}

        let node = cursor.node();

        let kind = node.kind();
        let name = String::from(node_text(&bfile.code, &node));

        let result = match kind {
            "identifier" => {
                let namecp = name.clone();
                let items = self.find_identities(
                    &file.borrow(),
                    &|item_name| item_name == namecp,
                    &node,
                    false,
                    true,
                );
                let locs = items
                    .iter()
                    .filter(|item| item.name == name && item.url.is_some())
                    .map(|item| Location {
                        uri: item.url.as_ref().unwrap().clone(),
                        range: item.range,
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
        let result = serde_json::to_value(&result).unwrap();

        self.respond(Response {
            id,
            result: Some(result),
            error: None,
        });
    }

    fn handle_completion(&mut self, id: RequestId, params: CompletionParams) {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        file.borrow_mut().gen_items_if_needed();

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

        // print!("{:?} {:?}\n", name, &id);

        let mut items = self.find_identities(&file.borrow(), &|_| true, &node, true, true);

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
                            true,
                        );

                        if !fun_items.is_empty() {
                            let item = &fun_items[0];

                            let param_items = match &item.kind {
                                ItemKind::Module { params, .. } => {
                                    let mut result = vec![];
                                    for p in params {
                                        result.push(Rc::new(Item {
                                            name: p.name.clone(),
                                            kind: ItemKind::Variable,
                                            range: p.range,
                                            url: Some(bfile.url.clone()),
                                            ..Default::default()
                                        }));
                                    }
                                    result
                                }
                                ItemKind::Function(params) => {
                                    let mut result = vec![];
                                    for p in params {
                                        result.push(Rc::new(Item {
                                            name: p.name.clone(),
                                            kind: ItemKind::Variable,
                                            range: p.range,
                                            url: Some(bfile.url.clone()),
                                            ..Default::default()
                                        }));
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

        let result = match kind {
            "include_path" => CompletionResponse::List(CompletionList {
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
            }),
            _ => CompletionResponse::List(CompletionList {
                is_incomplete: true,
                items: items
                    .iter()
                    .map(|item| CompletionItem {
                        label: match &item.label {
                            Some(label) => label.to_owned(),
                            None => item.make_label(),
                        },
                        kind: Some(item.kind.completion_kind()),
                        filter_text: Some(item.name.to_owned()),
                        insert_text: Some(item.make_snippet()),
                        insert_text_format: Some(match &item.kind {
                            ItemKind::Variable => InsertTextFormat::PLAIN_TEXT,
                            _ => InsertTextFormat::SNIPPET,
                        }),
                        insert_text_mode: Some(InsertTextMode::ADJUST_INDENTATION),
                        documentation: item.hover.as_ref().map(|doc| {
                            Documentation::MarkupContent(MarkupContent {
                                kind: lsp_types::MarkupKind::Markdown,
                                value: doc.to_owned(),
                            })
                        }),
                        ..Default::default()
                    })
                    .collect(),
            }),
        };

        let result = serde_json::to_value(&result).unwrap();
        self.respond(Response {
            id,
            result: Some(result),
            error: None,
        });
    }

    fn handle_document_symbols(&mut self, id: RequestId, params: DocumentSymbolParams) {
        let uri = &params.text_document.uri;
        let file = match self.get_code(uri) {
            Some(code) => code,
            _ => return,
        };

        let mut bfile = file.borrow_mut();
        bfile.gen_items_if_needed();
        if let Some(items) = &bfile.root_items {
            let result: Vec<SymbolInformation> = items
                .iter()
                .map(|item| {
                    #[allow(deprecated)]
                    SymbolInformation {
                        name: item.name.to_owned(),
                        kind: item.get_symbol_kind(),
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: item.url.clone().unwrap(),
                            range: item.range,
                        },
                        container_name: None,
                    }
                })
                .collect();

            let result = DocumentSymbolResponse::Flat(result);

            let result = serde_json::to_value(&result).unwrap();
            self.respond(Response {
                id,
                result: Some(result),
                error: None,
            });
        }
    }

    fn handle_formatting(&mut self, id: RequestId, params: DocumentFormattingParams) {
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

        let child = match Command::new(&self.args.fmt_exe)
            .arg(format!("-style={}", self.args.fmt_style))
            .arg("-assume-filename=foo.scad")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
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

                    let result = serde_json::to_value(&result).unwrap();
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

// Notification handlers.
impl Server {
    fn handle_did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        let DidOpenTextDocumentParams { text_document: doc } = params;
        if self.code.contains_key(&doc.uri) {
            return;
        }
        self.insert_code(doc.uri, doc.text);
    }

    fn handle_did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document,
            content_changes,
        } = params;

        let pc = match self.code.get_mut(&text_document.uri) {
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

    fn handle_did_change_config(&mut self, params: DidChangeConfigurationParams) {
        #[derive(Deserialize)]
        struct Openscad {
            search_paths: Option<String>,
            fmt_style: Option<String>,
            fmt_exe: Option<String>,
        }

        #[derive(Deserialize)]
        struct Settings {
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
                .unwrap();

            self.extend_libs(paths);

            if let Some(style) = settings.openscad.fmt_style {
                if !style.trim().is_empty() && self.args.fmt_style != style {
                    self.args.fmt_style = style;
                }
            }

            if let Some(fmt_exe) = settings.openscad.fmt_exe {
                if !fmt_exe.trim().is_empty() && self.args.fmt_exe != fmt_exe {
                    self.args.fmt_exe = fmt_exe;
                }
            }
        }
    }

    fn handle_did_save_text_document(&mut self, _params: DidSaveTextDocumentParams) {}

    fn handle_did_close_text_document(&mut self, _params: DidCloseTextDocumentParams) {}
}

// Code-related helpers.
impl Server {
    fn get_code(&mut self, uri: &Url) -> Option<Rc<RefCell<ParsedCode>>> {
        match self.code.get(uri) {
            Some(x) => Some(Rc::clone(x)),
            None => self.read_and_cache(uri.clone()).ok(),
        }
    }

    fn insert_code(&mut self, url: Url, code: String) -> Rc<RefCell<ParsedCode>> {
        while self.code.len() > 100 {
            self.code.pop_front();
        }

        let rc = Rc::new(RefCell::new(ParsedCode::new(
            tree_sitter_openscad::language(),
            code,
            url.clone(),
            self.library_locations.clone(),
        )));
        self.code.insert(url, rc.clone());
        rc
    }

    fn find_identities(
        &mut self,
        code: &ParsedCode,
        comparator: &dyn Fn(&str) -> bool,
        start_node: &Node,
        findall: bool,
        inc_builtin: bool,
    ) -> Vec<Rc<Item>> {
        let mut result = vec![];
        let mut start_pos = start_node.start_byte();
        let mut include_vec = vec![];
        if inc_builtin {
            include_vec.push(Url::parse(BUILTIN_PATH).unwrap())
        }
        if let Some(incs) = &code.includes {
            include_vec.extend(incs.clone());
        }

        let mut should_process_param = false;

        let mut node = *start_node;
        let mut parent = start_node.parent();

        while parent.is_some() {
            let should_continue = parent.unwrap().parent().is_some();

            loop {
                if node.kind().is_include_statement() {
                    code.get_include_url(&node).map(|inc| {
                        include_vec.push(inc);
                    });
                }

                match node.kind() {
                    "module_declaration" | "function_declaration" => {
                        if node.end_byte() > start_pos {
                            should_process_param = true;
                            start_pos = code.tree.root_node().end_byte();
                        }
                    }
                    _ => (),
                }

                if node.start_byte() < start_pos {
                    if let Some(mut item) = Item::parse(&code.code, &node) {
                        if should_process_param {
                            match &item.kind {
                                ItemKind::Module { params, .. } => {
                                    should_process_param = false;
                                    for p in params {
                                        if comparator(&p.name) {
                                            result.push(Rc::new(Item {
                                                name: p.name.clone(),
                                                kind: ItemKind::Variable,
                                                range: p.range,
                                                url: Some(code.url.clone()),
                                                ..Default::default()
                                            }));
                                            if !findall {
                                                return result;
                                            }
                                        }
                                    }
                                }
                                ItemKind::Function(params) => {
                                    should_process_param = false;
                                    for p in params {
                                        if comparator(&p.name) {
                                            result.push(Rc::new(Item {
                                                name: p.name.clone(),
                                                kind: ItemKind::Variable,
                                                range: p.range,
                                                url: Some(code.url.clone()),
                                                ..Default::default()
                                            }));
                                            if !findall {
                                                return result;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            };
                        }

                        if should_continue && comparator(&item.name) {
                            item.url = Some(code.url.clone());
                            result.push(Rc::new(item));
                            if !findall {
                                return result;
                            }
                        }
                    }
                }

                if !should_continue || node.prev_sibling().is_none() {
                    node = parent.unwrap();
                    parent = node.parent();
                    break;
                } else {
                    node = node.prev_sibling().unwrap();
                }
            }
        }

        if let Some(items) = &code.root_items {
            for item in items {
                if comparator(&item.name) {
                    result.push(item.clone());
                    if !findall {
                        return result;
                    }
                }
            }
        }

        for inc in include_vec {
            let inccode = match self.get_code(&inc) {
                Some(code) => code,
                _ => return result,
            };

            let mut inccode = inccode.borrow_mut();
            inccode.gen_items_if_needed();
            result.extend(self.find_identities(
                &inccode,
                &comparator,
                &inccode.tree.root_node(),
                findall,
                false,
            ));
            if !result.is_empty() && !findall {
                return result;
            }
        }

        result
    }

    fn read_and_cache(&mut self, url: Url) -> io::Result<Rc<RefCell<ParsedCode>>> {
        let text = read_to_string(url.to_file_path().unwrap())?;

        match self.code.entry(url.clone()) {
            linked_hash_map::Entry::Occupied(o) => {
                if o.get().borrow().code != text {
                    Ok(self.insert_code(url, text))
                } else {
                    Ok(Rc::clone(o.get()))
                }
            }
            linked_hash_map::Entry::Vacant(_) => Ok(self.insert_code(url, text)),
        }
    }
}

// Miscellaneous high-level logic.
impl Server {
    fn user_defined_library_locations() -> Vec<String> {
        match env::var("OPENSCADPATH") {
            Ok(path) => env::split_paths(&path)
                .filter_map(|buf| buf.into_os_string().into_string().ok())
                .collect(),
            Err(_) => vec![],
        }
    }

    fn built_in_library_location() -> Option<String> {
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

    fn installation_library_location() -> Option<String> {
        // TODO: Figure out the other cases.
        if cfg!(target_os = "windows") {
            Some("C:\\Program Files\\OpenSCAD\\libraries\\".into())
        } else if cfg!(target_os = "macos") {
            Some("/Applications/OpenSCAD.app/Contents/Resources/libraries/".into())
        } else {
            Some("/usr/share/openscad/libraries/".into())
        }
    }

    fn make_library_locations(&mut self) {
        let mut ret = Self::user_defined_library_locations();
        ret.extend(Self::built_in_library_location());
        ret.extend(Self::installation_library_location());

        self.extend_libs(ret);
    }

    fn extend_libs(&mut self, userlibs: Vec<String>) {
        let ret: Vec<Url> = userlibs
            .into_iter()
            .map(|lib| shellexpand::tilde(&lib).to_string())
            .filter_map(|p| {
                if p.is_empty() {
                    return None;
                }

                if let Ok(uri) = Url::parse(&format!("file://{}", p)) {
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

    fn new(connection: Connection, args: Cli) -> Self {
        let mut instance = Self {
            library_locations: Rc::new(RefCell::new(vec![])),
            connection,
            code: Default::default(),
            args,
        };
        let code = BUILTINS_SCAD.to_owned();
        let url = Url::parse(BUILTIN_PATH).unwrap();
        let rc = instance.insert_code(url, code);
        rc.borrow_mut().is_builtin = true;

        instance.make_library_locations();

        instance
    }

    fn notify(&self, notif: lsp_server::Notification) {
        self.connection
            .sender
            .send(Message::Notification(notif))
            .unwrap()
    }

    fn respond(&self, mut resp: Response) {
        // log_to_console!("{:?}\n\n", &resp);
        if resp.result.is_none() {
            resp.result = Some(json!("{}"))
        }
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
                                    err_to_console!("method: {method} error: {error}\n");
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
                                    err_to_console!("method: {method} error: {error}\n");
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

    fn main_loop(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
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

#[derive(Parser)]
#[clap(name = "OpenSCAD-LSP")]
#[clap(author, version, about)]
struct Cli {
    #[clap(short, long, default_value_t = String::from("3245"))]
    port: String,

    #[clap(long, default_value_t = String::from("127.0.0.1"))]
    ip: String,

    #[clap(long, default_value_t = String::from("Microsoft"), help = "LLVM, GNU, Google, Chromium, Microsoft, Mozilla, WebKit, file")]
    fmt_style: String,

    #[clap(long, default_value_t = String::from("clang-format"), help = "clang format executable file path")]
    fmt_exe: String,

    #[clap(long, help = "use stdio instead of tcp")]
    stdio: bool,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let args = Cli::parse();

    let (connection, io_threads) = if args.stdio {
        log_to_console!("start with stdio");
        Connection::stdio()
    } else {
        log_to_console!("start with socket");
        match Connection::listen(format!("{}:{}", args.ip, args.port)) {
            Ok(res) => res,
            Err(err) => {
                err_to_console!("{}", err);
                return Ok(()); // return an error from main will print it to stderr
            }
        }
    };

    log_to_console!("start sucess");
    let mut server = Server::new(connection, args);
    server.main_loop()?;
    io_threads.join()?;

    err_to_console!("exit");
    Ok(())
}
