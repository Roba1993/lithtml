use super::node::Node;
use super::span::SourceSpan;
use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::default::Default;
use std::result::Result;

/// Normal: `<div></div>` or Void: `<meta/>`and `<meta>`
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// TODO: Align with: https://html.spec.whatwg.org/multipage/syntax.html#elements-2
pub enum ElementVariant {
    /// A normal element can have children, ex: <div></div>.
    Normal,
    /// A void element can't have children, ex: <meta /> and <meta>
    Void,
}

pub type Attributes<'s> = HashMap<&'s str, Option<&'s str>>;

/// Most of the parsed html nodes are elements, except for text
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Element<'s> {
    /// The name / tag of the element
    pub name: &'s str,

    /// The element variant, if it is of type void or not
    pub variant: ElementVariant,

    /// All of the elements attributes, except id and class
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(serialize_with = "ordered_map")]
    pub attributes: Attributes<'s>,

    /// All of the elements classes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<&'s str>,

    /// All of the elements child nodes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node<'s>>,

    /// Span of the element in the parsed source
    #[serde(skip)]
    pub source_span: SourceSpan<'s>,
}

impl<'s> Default for Element<'s> {
    fn default() -> Self {
        Self {
            name: "",
            variant: ElementVariant::Void,
            classes: vec![],
            attributes: HashMap::new(),
            children: vec![],
            source_span: SourceSpan::default(),
        }
    }
}

fn ordered_map<S: Serializer>(value: &Attributes, serializer: S) -> Result<S::Ok, S::Error> {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
