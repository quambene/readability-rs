use crate::scorer::Candidate;
use markup5ever_rcdom::NodeData;
use std::collections::BTreeMap;

pub fn debug_candidates(
    candidates: &BTreeMap<String, Candidate>,
) -> Vec<(&str, Vec<(String, String)>, f32)> {
    candidates
        .values()
        .filter_map(debug_candidate)
        .collect::<Vec<_>>()
}

pub fn debug_candidate(candidate: &Candidate) -> Option<(&str, Vec<(String, String)>, f32)> {
    if let NodeData::Element { name, attrs, .. } = &candidate.node.data {
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
            candidate.score.get(),
        ))
    } else {
        None
    }
}
