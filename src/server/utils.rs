use lsp_server::{ExtractError, Request, RequestId};
use lsp_types::Position;
use lsp_types::Range;
use tree_sitter::{Node, Point, TreeCursor};

macro_rules! log_to_console {
        ($($arg:tt)*) => {
            eprint!("[server] ");
            eprintln!($($arg)*);
        };
    }

macro_rules! err_to_console {
        ($($arg:tt)*) => {
            eprint!("[error] ");
            eprintln!($($arg)*);
        };
    }

pub(crate) fn find_offset(text: &str, pos: Position) -> Option<usize> {
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

pub(crate) fn to_point(p: Position) -> Point {
    Point {
        row: p.line as usize,
        column: p.character as usize,
    }
}

pub(crate) fn node_text<'a>(code: &'a str, node: &Node) -> &'a str {
    &code[node.byte_range()]
}

// The callback may move the cursor while executing, but it must always ultimately leave it in the
// same position it was in at the beginning.
pub(crate) fn for_each_child<'a>(
    cursor: &mut TreeCursor<'a>,
    mut cb: impl FnMut(&mut TreeCursor<'a>),
) {
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

pub(crate) fn error_nodes(mut cursor: TreeCursor) -> Vec<Node> {
    pub(crate) fn helper<'a>(ret: &mut Vec<Node<'a>>, cursor: &mut TreeCursor<'a>) {
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

pub(crate) fn cast_request<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
{
    req.extract(R::METHOD)
}

pub(crate) fn cast_notification<N>(
    notif: lsp_server::Notification,
) -> Result<N::Params, ExtractError<lsp_server::Notification>>
where
    N: lsp_types::notification::Notification,
{
    notif.extract(N::METHOD)
}

pub(crate) trait NodeExt {
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

pub(crate) trait KindExt {
    fn is_include_statement(&self) -> bool;
    fn is_comment(&self) -> bool;
    fn is_callable(&self) -> bool;
}

impl KindExt for str {
    fn is_include_statement(&self) -> bool {
        self == "include_statement" || self == "use_statement"
    }

    fn is_comment(&self) -> bool {
        self == "comment"
    }

    fn is_callable(&self) -> bool {
        self == "module_declaration" || self == "function_declaration"
    }
}
