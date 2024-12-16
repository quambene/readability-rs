use crate::scorer::Candidate;
use markup5ever_rcdom::{Handle, NodeData};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct CandidateTag {
    pub name: String,
    pub attribute_id: Option<String>,
    pub score: f32,
}

impl CandidateTag {
    #[allow(dead_code)]
    pub fn new(name: &str, attribute_id: Option<&str>, score: f32) -> Self {
        Self {
            name: name.to_owned(),
            attribute_id: attribute_id.map(|attribute_id| attribute_id.to_owned()),
            score,
        }
    }
}

pub fn debug_candidates(candidates: &BTreeMap<String, Candidate>) -> Vec<CandidateTag> {
    candidates
        .values()
        .filter_map(debug_candidate)
        .collect::<Vec<_>>()
}

pub fn debug_candidate(candidate: &Candidate) -> Option<CandidateTag> {
    if let NodeData::Element { name, attrs, .. } = &candidate.node.data {
        Some(CandidateTag {
            name: name.local.to_string(),
            attribute_id: attrs.borrow().iter().find_map(|attribute| {
                if attribute.name.local.to_string() == "id" {
                    Some(attribute.value.to_string())
                } else {
                    None
                }
            }),
            score: candidate.score.get(),
        })
    } else {
        None
    }
}

pub fn debug_node(node: &Handle) -> Option<(&str, Vec<(String, String)>)> {
    if let NodeData::Element { name, attrs, .. } = &node.data {
        Some((
            name.local.as_ref(),
            attrs
                .borrow()
                .iter()
                .map(|attribute| {
                    (
                        attribute.name.local.to_string(),
                        attribute.value.to_string(),
                    )
                })
                .collect::<Vec<_>>(),
        ))
    } else {
        None
    }
}
