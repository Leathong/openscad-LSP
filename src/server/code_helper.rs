use std::{cell::RefCell, fs::read_to_string, io, rc::Rc};

use lsp_types::Url;
use tree_sitter::Node;

use crate::{
    code::ParsedCode,
    response_item::{Item, ItemKind},
    server::{Server, BUILTIN_PATH},
    utils::*,
};

// Code-related helpers.
impl Server {
    pub(crate) fn get_code(&mut self, uri: &Url) -> Option<Rc<RefCell<ParsedCode>>> {
        match self.code.get(uri) {
            Some(x) => Some(Rc::clone(x)),
            None => self.read_and_cache(uri.clone()).ok(),
        }
    }

    pub(crate) fn insert_code(&mut self, url: Url, code: String) -> Rc<RefCell<ParsedCode>> {
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

    pub(crate) fn find_identities(
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

    pub(crate) fn read_and_cache(&mut self, url: Url) -> io::Result<Rc<RefCell<ParsedCode>>> {
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
