use std::{cell::RefCell, path::PathBuf, rc::Rc};

use lazy_static::lazy_static;
use lsp_types::{TextDocumentContentChangeEvent, Url};
use tree_sitter::{InputEdit, Node, Point, Tree, TreeCursor};

use crate::response_item::{Item, ItemKind};
use crate::utils::*;
use regex::Regex;

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
    ("use", "use <${1:PATH}>$0"),
    ("each", "each ${1:LIST}$0"),
];

pub(crate) struct ParsedCode {
    pub parser: tree_sitter::Parser,
    pub code: String,
    pub tree: Tree,
    pub url: Url,
    pub root_items: Option<Vec<Rc<RefCell<Item>>>>,
    pub includes: Option<Vec<Url>>,
    pub is_builtin: bool,
    pub external_builtin: bool,
    pub changed: bool,
    pub libs: Rc<RefCell<Vec<Url>>>,
}

impl ParsedCode {
    pub(crate) fn new(code: String, url: Url, libs: Rc<RefCell<Vec<Url>>>) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_openscad::LANGUAGE.into())
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
            external_builtin: false,
            libs,
            changed: true,
        }
    }

    pub(crate) fn edit(&mut self, events: &[TextDocumentContentChangeEvent]) {
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

    pub(crate) fn gen_top_level_items_if_needed(&mut self) {
        if self.root_items.is_some() && !self.changed {
            return;
        }
        self.changed = false;
        self.gen_top_level_items();
    }

    pub(crate) fn extract_doc(&self, doc: &str, builtin: bool) -> String {
        lazy_static! {
            static ref DOC_RE: Regex =
                Regex::new(r"(?m)(^\s*//+)|(^\s*/\*+\n?)|(\*+/)|(^\s* )").unwrap();
            static ref BTI_RE: Regex = Regex::new(r"(?m)(^\s*/\*+\n?)|(\*+/)").unwrap();
        };

        if builtin {
            BTI_RE.replace_all(doc, "").to_string()
        } else {
            DOC_RE.replace_all(doc, "").to_string()
        }
    }

    pub(crate) fn gen_top_level_items(&mut self) {
        let mut cursor: TreeCursor = self.tree.walk();
        let mut ret: Vec<Item> = vec![];
        let mut inc = vec![];

        let mut doc: Option<String> = None;
        let mut doc_node: Option<Node> = None;
        let mut last_code_line: usize = 0;

        for_each_child(&mut cursor, |cursor| {
            let node = &cursor.node();
            let kind = node.kind();
            // log_to_console!("kind: {}", kind);
            if kind.is_comment() {
                if last_code_line > 0 && node.start_position().row == last_code_line {
                    let last = ret.last_mut().unwrap();
                    let doc_str = node_text(&self.code, node);
                    let newdoc = self.extract_doc(doc_str, self.is_builtin);

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
                    item.is_builtin = self.is_builtin;
                    if !self.is_builtin || self.external_builtin {
                        item.url = Some(self.url.clone());
                    }
                    item.doc = doc
                        .as_ref()
                        .map(|doc| self.extract_doc(doc, self.is_builtin));
                    item.label = Some(item.make_label());
                    item.hover = Some(item.make_hover());
                    last_code_line = item.range.start.line as usize;
                    // log_to_console!("name: {} kind: {}", item.name, kind);
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
        for mut item in ret {
            item.is_builtin = self.is_builtin;
            items.push(Rc::new(RefCell::new(item)));
        }

        self.root_items = Some(items);
        self.includes = Some(inc);
    }

    pub(crate) fn get_include_url(&self, incstat_node: &Node) -> Option<Url> {
        let mut res = None;
        let include_path = node_text(&self.code, &incstat_node.child(1).unwrap())
            .trim_start_matches(&['<', '\n'][..])
            .trim_end_matches(&['>', '\n'][..]);

        if include_path.is_empty() {
            return None;
        }

        let mut urls = vec![&self.url];
        let libs = self.libs.borrow();
        urls.extend(libs.iter());

        for url in urls {
            match url.join(include_path) {
                Ok(url) => {
                    if let Ok(path) = url.to_file_path() {
                        if path.exists() {
                            res = Some(url);
                            return res;
                        }
                    }
                }
                Err(err) => {
                    err_to_console!("{:?} {}", err.to_string(), include_path);
                }
            }
        }
        res
    }

    pub(crate) fn get_include_completion(&self, inc_path: &Node) -> Vec<String> {
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
