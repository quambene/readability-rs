use crate::dom::{Handle, Node, NodeData};
use html5ever::Attribute;
use std::{rc::Rc, str::FromStr};
use tendril::StrTendril;

pub fn attr(attr_name: &str, attrs: &[Attribute]) -> Option<String> {
    for attr in attrs.iter() {
        if attr.name.local.as_ref() == attr_name {
            return Some(attr.value.to_string());
        }
    }
    None
}

pub fn set_attr(attr_name: &str, value: &str, handle: Handle) {
    if let NodeData::Element {
        name: _, ref attrs, ..
    } = handle.data
    {
        let attrs = &mut attrs.borrow_mut();
        if let Some(index) = attrs.iter().position(|attr| {
            let name = attr.name.local.as_ref();
            name == attr_name
        }) {
            if let Ok(value) = StrTendril::from_str(value) {
                attrs[index] = Attribute {
                    name: attrs[index].name.clone(),
                    value,
                }
            }
        }
    }
}

pub fn clean_attr(attr_name: &str, attrs: &mut Vec<Attribute>) {
    if let Some(index) = attrs.iter().position(|attr| {
        let name = attr.name.local.as_ref();
        name == attr_name
    }) {
        attrs.remove(index);
    }
}

pub fn get_tag_name(handle: Handle) -> Option<String> {
    match handle.data {
        NodeData::Element { ref name, .. } => Some(name.local.as_ref().to_lowercase().to_string()),
        _ => None,
    }
}

pub fn get_attr(name: &str, handle: Handle) -> Option<String> {
    match handle.data {
        NodeData::Element {
            name: _, ref attrs, ..
        } => attr(name, &attrs.borrow()),
        _ => None,
    }
}

pub fn is_empty(handle: Handle) -> bool {
    for child in handle.children.borrow().iter() {
        let c = child.clone();
        match c.data {
            NodeData::Text { ref contents } => {
                if contents.borrow().trim().len() > 0 {
                    return false;
                }
            }
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name.to_lowercase().as_ref() {
                    "li" | "dt" | "dd" | "p" | "div" => {
                        if !is_empty(child.clone()) {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
            _ => (),
        }
    }
    matches!(
        get_tag_name(handle.clone()).unwrap_or_default().as_ref(),
        "li" | "dt" | "dd" | "p" | "div" | "canvas"
    )
}

#[allow(dead_code)]
pub fn has_link(handle: Handle) -> bool {
    if "a" == &get_tag_name(handle.clone()).unwrap_or_default() {
        return true;
    }
    for child in handle.children.borrow().iter() {
        if has_link(child.clone()) {
            return true;
        }
    }
    false
}

pub fn text_len(handle: Handle) -> usize {
    let mut len = 0;
    for child in handle.children.borrow().iter() {
        let c = child.clone();
        match c.data {
            NodeData::Text { ref contents } => {
                len += contents.borrow().trim().chars().count();
            }
            NodeData::Element { .. } => {
                len += text_len(child.clone());
            }
            _ => (),
        }
    }
    len
}

pub fn find_node(handle: Handle, tag_name: &str, nodes: &mut Vec<Rc<Node>>) {
    for child in handle.children.borrow().iter() {
        let c = child.clone();
        if let NodeData::Element { ref name, .. } = c.data {
            let t = name.local.as_ref();
            if t.to_lowercase() == tag_name {
                nodes.push(child.clone());
            };
            find_node(child.clone(), tag_name, nodes)
        }
    }
}

pub fn has_nodes(handle: Handle, tag_names: &[&str]) -> bool {
    for child in handle.children.borrow().iter() {
        let tag_name: &str = &get_tag_name(child.clone()).unwrap_or_default();
        if tag_names.iter().any(|&n| n == tag_name) {
            return true;
        }
        if match child.clone().data {
            NodeData::Element { .. } => has_nodes(child.clone(), tag_names),
            _ => false,
        } {
            return true;
        }
    }
    false
}

pub fn text_children_count(handle: Handle) -> usize {
    let mut count = 0;
    for child in handle.children.borrow().iter() {
        let c = child.clone();
        if let NodeData::Text { ref contents } = c.data {
            let s = contents.borrow();
            if s.trim().len() >= 20 {
                count += 1
            }
        }
    }
    count
}