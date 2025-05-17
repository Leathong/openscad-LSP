use std::{cell::RefCell, fs::read_to_string, io, rc::Rc};

use lsp_types::Url;
use tree_sitter::Node;

use crate::{
    parse_code::ParsedCode,
    response_item::{Item, ItemKind},
    server::Server,
    utils::*,
};

// Code-related helpers.
impl Server {
    pub(crate) fn get_code(&mut self, uri: &Url) -> Option<Rc<RefCell<ParsedCode>>> {
        match self.codes.get(uri) {
            Some(x) => Some(Rc::clone(x)),
            None => self.read_and_cache(uri.clone()).ok(),
        }
    }

    pub(crate) fn insert_code(&mut self, url: Url, code: String) -> Rc<RefCell<ParsedCode>> {
        while self.codes.len() > 1000 {
            self.codes.pop_front();
        }

        let rc = Rc::new(RefCell::new(ParsedCode::new(
            code,
            url.clone(),
            self.library_locations.clone(),
        )));
        self.codes.insert(url, rc.clone());
        rc
    }

    pub(crate) fn find_identities(
        &mut self,
        code: &ParsedCode,
        comparator: &dyn Fn(&str) -> bool,
        start_node: &Node,
        findall: bool,
        depth: i32,
    ) -> Vec<Rc<RefCell<Item>>> {
        let mut result: Vec<Rc<RefCell<Item>>> = vec![];
        if depth >= self.args.depth {
            return result;
        }

        let mut include_vec = vec![];
        if depth == 0 {
            include_vec.push(self.builtin_url.clone())
        }
        if let Some(incs) = &code.includes {
            include_vec.extend(incs.clone());
        }

        let mut node = *start_node;
        let mut parent = start_node.parent();

        'outer: while parent.is_some() {
            let is_top_level_node = parent.unwrap().parent().is_none();

            loop {
                if node.kind().is_include_statement() {
                    code.get_include_url(&node).map(|inc| {
                        include_vec.push(inc);
                    });
                }

                if let Some(mut item) = Item::parse(&code.code, &node) {
                    match &item.kind {
                        ItemKind::Module { params, .. } => {
                            for p in params {
                                if comparator(&p.name) {
                                    result.push(Rc::new(RefCell::new(Item {
                                        name: p.name.clone(),
                                        kind: ItemKind::Variable,
                                        range: p.range,
                                        url: Some(code.url.clone()),
                                        ..Default::default()
                                    })));
                                    if !findall {
                                        return result;
                                    }
                                }
                            }
                        }
                        ItemKind::Function { flags: _, params } => {
                            for p in params {
                                if comparator(&p.name) {
                                    result.push(Rc::new(RefCell::new(Item {
                                        name: p.name.clone(),
                                        kind: ItemKind::Variable,
                                        range: p.range,
                                        url: Some(code.url.clone()),
                                        ..Default::default()
                                    })));
                                    if !findall {
                                        return result;
                                    }
                                }
                            }
                        }
                        _ => {}
                    };

                    if !is_top_level_node && comparator(&item.name) {
                        item.url = Some(code.url.clone());
                        result.push(Rc::new(RefCell::new(item)));
                        if !findall {
                            return result;
                        }
                    }
                }

                if is_top_level_node {
                    break 'outer;
                } else if node.prev_sibling().is_none() {
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
                if comparator(&item.borrow().name) {
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

            if let Ok(mut inccode) = inccode.try_borrow_mut() {
                inccode.gen_top_level_items_if_needed();
                result.extend(self.find_identities(
                    &inccode,
                    &comparator,
                    &inccode.tree.root_node(),
                    findall,
                    depth + 1,
                ));
            }

            if !result.is_empty() && !findall {
                return result;
            }
        }

        result
    }

    pub(crate) fn read_and_cache(&mut self, url: Url) -> io::Result<Rc<RefCell<ParsedCode>>> {
        let text = read_to_string(url.to_file_path().unwrap())?;

        match self.codes.entry(url.clone()) {
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
