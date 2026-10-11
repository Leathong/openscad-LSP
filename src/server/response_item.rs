use lazy_static::lazy_static;
use lsp_types::{CompletionItemKind, Range, SymbolKind, Uri};
use regex::Regex;
use tree_sitter::Node;

use crate::utils::*;

use crate::{Cli, DocFormat};

struct BuiltinFlags {}
impl BuiltinFlags {
    const IS_OPERATOR: u16 = 1;
    const IGNORE_PARAM_NAME: u16 = 1 << 1;
}

#[derive(Clone, Debug)]
pub(crate) struct Param {
    pub name: String,
    pub default: Option<String>,
    pub range: Range,
}

impl Param {
    pub(crate) fn parse_declaration(code: &str, node: &Node) -> Vec<Self> {
        node.children(&mut node.walk())
            .filter_map(|child| {
                let kind = child.kind();
                if !kind.is_parameter() {
                    return None;
                }

                let child = child.child(0).unwrap();
                let kind = child.kind();
                match kind {
                    "identifier" => Some(Self {
                        name: node_text(code, &child).to_owned(),
                        default: None,
                        range: child.lsp_range(),
                    }),
                    "assignment" => child.child_by_field_name("name").and_then(|left| {
                        child.child_by_field_name("value").map(|right| Self {
                            name: node_text(code, &left).to_owned(),
                            default: Some(node_text(code, &right).to_owned()),
                            range: left.lsp_range(),
                        })
                    }),
                    "special_variable" => None,
                    _ => None,
                }
            })
            .collect()
    }

    pub(crate) fn make_snippet(params: &[Self], ignore_name: bool, args: &Cli) -> String {
        params
            .iter()
            .filter(|p| p.default.is_none() || !args.ignore_default)
            .enumerate()
            .map(|(i, p)| {
                if !args.ignore_default && p.default.as_ref().is_some() {
                    return format!("{} = {}", p.name, p.default.as_ref().unwrap());
                }

                if ignore_name {
                    format!("${{{}:{}}}", i + 1, p.name)
                } else {
                    format!("{} = ${{{}:{}}}", p.name, i + 1, p.name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Default)]
pub(crate) enum ItemKind {
    #[default]
    Variable,
    Function {
        flags: u16,
        params: Vec<Param>,
    },
    Keyword(String),
    Module {
        flags: u16,
        params: Vec<Param>,
    },
}

impl ItemKind {
    pub(crate) const fn completion_kind(&self) -> CompletionItemKind {
        match self {
            Self::Variable => CompletionItemKind::Variable,
            Self::Function { .. } => CompletionItemKind::Function,
            Self::Keyword(_) => CompletionItemKind::Keyword,
            Self::Module { .. } => CompletionItemKind::Module,
        }
    }
}

#[derive(Default)]
pub(crate) struct Item {
    pub name: String,
    pub kind: ItemKind,
    pub range: Range,
    pub url: Option<Uri>,
    pub is_builtin: bool,
    /// True for items produced by `gen_top_level_items` from source declarations
    /// (not keywords, parameters or local items). Completion only attaches
    /// documentation to these, matching the previous behaviour.
    pub is_top_level: bool,

    pub(crate) doc: Option<String>,
    pub(crate) hover: Option<(DocFormat, String)>,
    pub(crate) label: Option<String>,
    pub(crate) snippet: Option<String>,
}

/// Removes the common leading whitespace of all non-blank lines and turns
/// whitespace-only lines into empty lines. Widths are counted in chars.
fn dedent(doc: &str) -> String {
    let indent = doc
        .split('\n')
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);
    doc.split('\n')
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                l.chars().skip(indent).collect()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl Item {
    pub(crate) fn get_snippet(&mut self, args: &Cli) -> String {
        if self.snippet.is_none() {
            self.snippet = Some(self.make_snippet(args));
        }
        self.snippet.as_ref().unwrap().to_owned()
    }

    pub(crate) fn get_hover(&mut self, fmt: DocFormat) -> String {
        match &self.hover {
            Some((cached_fmt, text)) if *cached_fmt == fmt => text.to_owned(),
            _ => {
                let text = self.make_hover(fmt);
                self.hover = Some((fmt, text.clone()));
                text
            }
        }
    }

    pub(crate) fn get_label(&mut self) -> String {
        if self.label.is_none() {
            self.label = Some(self.make_label());
        }
        self.label.as_ref().unwrap().to_owned()
    }

    pub(crate) fn make_snippet(&mut self, args: &Cli) -> String {
        let snippet = match &self.kind {
            ItemKind::Variable => self.name.clone(),
            ItemKind::Function { flags, params } => {
                format!(
                    "{}({});$0",
                    self.name,
                    Param::make_snippet(params, BuiltinFlags::IGNORE_PARAM_NAME & flags != 0, args)
                )
            }
            ItemKind::Keyword(comp) => comp.clone(),
            ItemKind::Module { params, flags } => {
                let params =
                    Param::make_snippet(params, BuiltinFlags::IGNORE_PARAM_NAME & flags != 0, args);
                if BuiltinFlags::IS_OPERATOR & flags != 0 {
                    format!("{}({}) $0", self.name, params)
                } else {
                    format!("{}({});$0", self.name, params)
                }
            }
        };
        self.snippet = Some(snippet.to_owned());
        snippet
    }

    pub(crate) fn make_hover(&self, fmt: DocFormat) -> String {
        let mut label = match &self.label {
            Some(label) => label.to_owned(),
            None => self.make_label(),
        };
        label = match self.kind {
            ItemKind::Function { .. } => format!("```scad\nfunction {label}\n```"),
            ItemKind::Module { .. } => format!("```scad\nmodule {label}\n```"),
            _ => format!("```scad\n{label}\n```"),
        };
        if let Some(doc) = &self.doc {
            // Builtins are always raw Markdown, regardless of `fmt`.
            let use_pre = match fmt {
                DocFormat::Pre => true,
                DocFormat::Markdown => false,
            };
            if self.is_builtin {
                label = format!("{label}\n---\n\n{doc}\n");
            } else if use_pre {
                label = format!("{label}\n---\n\n<pre>\n{doc}\n</pre>\n");
            } else {
                label = format!("{label}\n---\n\n{}\n", dedent(doc));
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

        match &self.kind {
            ItemKind::Variable => self.name.to_owned(),
            ItemKind::Function { flags: _, params } => {
                format!("{}({})", self.name, format_params(params))
            }
            ItemKind::Keyword(_) => self.name.clone(),
            ItemKind::Module { params, .. } => {
                format!("{}({})", self.name, format_params(params))
            }
        }
    }

    pub(crate) fn parse(code: &str, node: &Node) -> Option<Self> {
        lazy_static! {
            static ref FLAG_RE: Regex =
                Regex::new(r"(?m)builtin_flags\((?P<flags>[01]{16})\)").unwrap();
        };

        let extract_name = |node: &Node, name| {
            // log_to_console!("{} {:?}", name, res);
            node.child_by_field_name(name)
                .map(|child| node_text(code, &child).to_owned())
        };

        let kind = node.kind();
        // log_to_console!("{}", kind);
        match kind {
            "module_item" => {
                let flags: u16 = if let Some(child) = node
                    .child_by_field_name("body")
                    .and_then(|body| body.named_child(0))
                {
                    let body = node_text(code, &child);
                    if let Some(cap) = &FLAG_RE.captures(body) {
                        let flag_str = &cap["flags"];
                        u16::from_str_radix(flag_str, 2).unwrap()
                    } else {
                        0
                    }
                } else {
                    0
                };
                Some(Self {
                    name: extract_name(node, "name")?,
                    kind: ItemKind::Module {
                        flags,
                        params: node
                            .child_by_field_name("parameters")
                            .map_or(vec![], |params| Param::parse_declaration(code, &params)),
                    },
                    range: node.lsp_range(),
                    ..Default::default()
                })
            }
            "function_item" => {
                let flags = if let Some(child) = node.child(4) {
                    let body = node_text(code, &child);
                    // log_to_console!("{}", &body);
                    if let Some(cap) = &FLAG_RE.captures(body) {
                        let flag_str = &cap["flags"];
                        u16::from_str_radix(flag_str, 2).unwrap()
                    } else {
                        0
                    }
                } else {
                    0
                };
                Some(Self {
                    name: extract_name(node, "name")?,
                    kind: ItemKind::Function {
                        flags,
                        params: node
                            .child_by_field_name("parameters")
                            .map_or(vec![], |params| Param::parse_declaration(code, &params)),
                    },
                    range: node.lsp_range(),
                    ..Default::default()
                })
            }
            "var_declaration" => {
                let node = node.named_child(0)?;
                Some(Self {
                    name: extract_name(&node, "name")?,
                    kind: ItemKind::Variable,
                    range: node.lsp_range(),
                    ..Default::default()
                })
            }
            _ => None,
        }
    }

    pub(crate) const fn get_symbol_kind(&self) -> SymbolKind {
        match self.kind {
            ItemKind::Function { .. } => SymbolKind::Function,
            ItemKind::Module { .. } => SymbolKind::Module,
            ItemKind::Variable => SymbolKind::Variable,
            ItemKind::Keyword(_) => SymbolKind::Key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn func(doc: Option<&str>, is_builtin: bool) -> Item {
        Item {
            name: "f".into(),
            kind: ItemKind::Function {
                flags: 0,
                params: vec![],
            },
            doc: doc.map(str::to_owned),
            is_builtin,
            ..Default::default()
        }
    }

    const HEADER: &str = "```scad\nfunction f()\n```";

    #[test]
    fn dedent_uniform_and_mixed() {
        assert_eq!(dedent("  a\n  b"), "a\nb");
        assert_eq!(dedent("  a\n    b\n  c"), "a\n  b\nc");
    }

    #[test]
    fn dedent_blank_lines_and_trailing_comment_prefix() {
        assert_eq!(dedent(" a\n\n b\n"), "a\n\nb\n");
        assert_eq!(dedent(" a\n   \n b"), "a\n\nb");
        // trailing same-line comment prefix produced by gen_top_level_items
        assert_eq!(dedent("  \n x"), "\nx");
    }

    #[test]
    fn dedent_empty_and_no_indent() {
        assert_eq!(dedent(""), "");
        assert_eq!(dedent("a\n b"), "a\n b");
    }

    #[test]
    fn hover_without_doc_is_header_only() {
        for fmt in [DocFormat::Pre, DocFormat::Markdown] {
            assert_eq!(func(None, false).make_hover(fmt), HEADER);
        }
    }

    #[test]
    fn hover_pre_wraps_non_builtin() {
        assert_eq!(
            func(Some(" doc"), false).make_hover(DocFormat::Pre),
            format!("{HEADER}\n---\n\n<pre>\n doc\n</pre>\n")
        );
    }

    #[test]
    fn hover_markdown_dedents_non_builtin() {
        assert_eq!(
            func(Some(" doc\n more"), false).make_hover(DocFormat::Markdown),
            format!("{HEADER}\n---\n\ndoc\nmore\n")
        );
    }

    #[test]
    fn hover_builtin_ignores_format() {
        let expected = format!("{HEADER}\n---\n\n  raw\n");
        for fmt in [DocFormat::Pre, DocFormat::Markdown] {
            assert_eq!(func(Some("  raw"), true).make_hover(fmt), expected);
        }
    }

    #[test]
    fn get_hover_recomputes_when_format_changes() {
        let mut item = func(Some(" doc"), false);
        assert!(item.get_hover(DocFormat::Pre).contains("<pre>"));
        assert!(!item.get_hover(DocFormat::Markdown).contains("<pre>"));
        assert!(item.get_hover(DocFormat::Pre).contains("<pre>"));
    }
}
