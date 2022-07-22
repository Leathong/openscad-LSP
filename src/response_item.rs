use lsp_types::{CompletionItemKind, Range, SymbolKind, Url};
use tree_sitter::Node;

use crate::utils::*;

#[derive(Clone, Debug)]
pub(crate) struct Param {
    pub name: String,
    pub default: Option<String>,
    pub range: Range,
}

impl Param {
    pub(crate) fn parse_declaration(code: &str, node: &Node) -> Vec<Param> {
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

    pub(crate) fn make_snippet(params: &[Param]) -> String {
        params
            .iter()
            .filter_map(|p| p.default.is_none().then(|| &p.name))
            .enumerate()
            .map(|(i, name)| format!("${{{}:{}}}", i + 1, name))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub(crate) enum ItemKind {
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
    pub(crate) fn completion_kind(&self) -> CompletionItemKind {
        match self {
            ItemKind::Variable => CompletionItemKind::VARIABLE,
            ItemKind::Function(_) => CompletionItemKind::FUNCTION,
            ItemKind::Keyword(_) => CompletionItemKind::KEYWORD,
            ItemKind::Module { .. } => CompletionItemKind::MODULE,
        }
    }
}

#[derive(Default)]
pub(crate) struct Item {
    pub name: String,
    pub kind: ItemKind,
    pub range: Range,
    pub url: Option<Url>,
    pub doc: Option<String>,
    pub hover: Option<String>,
    pub label: Option<String>,
    pub is_builtin: bool,
}

impl Item {
    pub(crate) fn make_snippet(&self) -> String {
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

    pub(crate) fn make_hover(&self) -> String {
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
            if self.is_builtin {
                label = format!("{}\n---\n\n{}\n", label, doc);
            } else {
                label = format!("{}\n---\n\n<pre>\n{}\n</pre>\n", label, doc);
            }
        }
        // print!("{}", &label);
        label
    }

    pub(crate) fn make_label(&self) -> String {
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

    pub(crate) fn parse(code: &str, node: &Node) -> Option<Self> {
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

    pub(crate) fn get_symbol_kind(&self) -> SymbolKind {
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
