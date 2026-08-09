use std::collections::HashSet;
use std::error::Error;
use std::sync::LazyLock;
use xmltree::{Element, XMLNode};

const SELECTED_POINT_FILL: &str = "#294246";
const SELECTED_TEXT_FILL: &str = "#ffffff";

/// All valid SVG electrode IDs in normalized lowercase form.
pub static POINT_IDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "cz", "t8", "c4", "c3", "t7", "pz", "fz", "p8", "o2", "p7", "f7", "f8", "o1", "fp1", "fp2",
        "f4", "f3", "p3", "p4", "a1", "a2", "fpz", "af7", "af3", "afz", "af4", "af8", "f9", "f10",
        "f5", "f1", "f2", "f6", "ft9", "ft7", "fc5", "fc3", "fc1", "fcz", "fc2", "fc4", "fc6",
        "ft8", "ft10", "t9", "c5", "c1", "c2", "c6", "t10", "tp9", "tp7", "cp5", "cp3", "cp1",
        "cpz", "cp2", "cp4", "cp6", "tp8", "tp10", "p9", "p5", "p1", "p2", "p6", "p10", "po7",
        "po3", "poz", "po4", "po8", "oz", "nz", "iz",
    ])
});

pub fn parse_points(raw: String) -> (Vec<String>, Vec<String>) {
    raw.split_whitespace()
        .map(str::to_owned)
        .partition(|s| POINT_IDS.contains(s.as_str()))
}

pub fn draw_eeg_svg(xml: Vec<u8>, points: Vec<String>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut xml_document = Element::parse(xml.as_slice())?;
    let selected_points = points
        .into_iter()
        .map(|point| point.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    let mut found_circles = HashSet::new();
    let mut found_labels = HashSet::new();

    select_svg_points(
        &mut xml_document,
        &selected_points,
        &mut found_circles,
        &mut found_labels,
    );

    if let Some(point) = selected_points.difference(&found_circles).next() {
        return Err(format!("Could not find point: {point}").into());
    }

    if let Some(point) = selected_points.difference(&found_labels).next() {
        return Err(format!("Could not find label for point: {point}").into());
    }

    let mut result = Vec::new();
    xml_document.write(&mut result)?;

    Ok(result)
}

fn select_svg_points(
    element: &mut Element,
    selected_points: &HashSet<String>,
    found_circles: &mut HashSet<String>,
    found_labels: &mut HashSet<String>,
) {
    if element.name == "circle" {
        let selected_id = element
            .attributes
            .get("id")
            .filter(|id| selected_points.contains(id.as_str()))
            .cloned();

        if let Some(id) = selected_id {
            set_element_fill(element, SELECTED_POINT_FILL);
            found_circles.insert(id);
        }
    } else if element.name == "text" {
        let selected_label = normalized_element_text(element)
            .filter(|label| selected_points.contains(label.as_str()));

        if let Some(label) = selected_label {
            set_element_fill(element, SELECTED_TEXT_FILL);
            found_labels.insert(label);
        }
    }

    for child in &mut element.children {
        if let XMLNode::Element(child_element) = child {
            select_svg_points(child_element, selected_points, found_circles, found_labels);
        }
    }
}

fn normalized_element_text(element: &Element) -> Option<String> {
    element.children.iter().find_map(|child| match child {
        XMLNode::Element(child_element) => normalized_element_text(child_element),
        XMLNode::Text(text) if !text.trim().is_empty() => Some(text.trim().to_ascii_lowercase()),
        _ => None,
    })
}

#[cfg(test)]
fn find_xml_element_by_id<'a>(root: &'a mut Element, target_id: &str) -> Option<&'a mut Element> {
    if root.attributes.get("id").is_some_and(|id| id == target_id) {
        return Some(root);
    }

    for child in &mut root.children {
        if let XMLNode::Element(child_element) = child
            && let Some(found) = find_xml_element_by_id(child_element, target_id)
        {
            return Some(found);
        }
    }

    None
}

#[cfg(test)]
fn find_xml_text_by_value<'a>(root: &'a mut Element, target: &str) -> Option<&'a mut Element> {
    if root.name == "text" && element_contains_text(root, target) {
        return Some(root);
    }

    for child in &mut root.children {
        if let XMLNode::Element(child_element) = child
            && let Some(found) = find_xml_text_by_value(child_element, target)
        {
            return Some(found);
        }
    }

    None
}

#[cfg(test)]
fn element_contains_text(root: &Element, target: &str) -> bool {
    root.children.iter().any(|child| match child {
        XMLNode::Element(child_element) => element_contains_text(child_element, target),
        XMLNode::Text(text) => text.trim().eq_ignore_ascii_case(target),
        _ => false,
    })
}

fn set_element_fill(element: &mut Element, color: &str) {
    element
        .attributes
        .insert("fill".to_owned(), color.to_owned());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_eeg_svg_inverts_selected_point_colors() {
        let source = include_bytes!("../images/map.svg").to_vec();
        let result = draw_eeg_svg(source, vec!["cz".to_owned(), "FPZ".to_owned()]).unwrap();
        let mut document = Element::parse(result.as_slice()).unwrap();

        for point in ["cz", "fpz"] {
            {
                let circle = find_xml_element_by_id(&mut document, point).unwrap();
                assert_eq!(
                    circle.attributes.get("fill").map(String::as_str),
                    Some(SELECTED_POINT_FILL)
                );
            }

            {
                let label = find_xml_text_by_value(&mut document, point).unwrap();
                assert_eq!(
                    label.attributes.get("fill").map(String::as_str),
                    Some(SELECTED_TEXT_FILL)
                );
                assert!(!label.attributes.get("style").is_some_and(|style| {
                    style.split(';').any(|declaration| {
                        declaration
                            .split_once(':')
                            .is_some_and(|(property, _)| property.trim() == "fill")
                    })
                }));
            }
        }
    }
}
