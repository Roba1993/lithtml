use super::node::Node;
use super::options::FormattingOptions;
use super::span::SourceSpan;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::default::Default;
use std::fmt::Display;
use std::result::Result;

/// Normal: `<div></div>` or Void: `<meta/>`and `<meta>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Element<'s> {
    /// The name / tag of the element
    pub name: &'s str,

    /// The element variant, if it is of type void or not
    pub variant: ElementVariant,

    /// All of the elements attributes, except id and class
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(serialize_with = "ordered_map")]
    #[serde(default)]
    pub attributes: Attributes<'s>,

    /// All of the elements classes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub classes: Vec<&'s str>,

    /// All of the elements child nodes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub children: Vec<Node<'s>>,

    /// Span of the element in the parsed source
    #[serde(skip)]
    #[serde(default)]
    pub source_span: SourceSpan<'s>,
}

impl<'s> Element<'s> {
    pub fn fmt_opt<W>(&self, f: &mut W, o: &FormattingOptions, depth: usize) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        // write tabs for the depth
        o.fmt_depth(f, depth)?;

        // write node start
        write!(f, "<{}", self.name)?;

        // count length of attributes name, value, signs
        let attr_len: usize = self
            .attributes
            .iter()
            .map(|(k, v)| k.len() + v.map(|v| v.len()).unwrap_or(0) + 4)
            .sum();

        // calculate the length of this element
        let e_len = depth + 1 + self.name.len() + attr_len + 1;

        // print in one line or multiline with depth - depending on space
        let c_inline = if e_len > o.max_len && o.new_lines {
            let mut c_inline = String::new();
            c_inline.push('\n');
            o.fmt_depth(&mut c_inline, depth + o.tab_size as usize)?;
            c_inline
        } else {
            String::from(" ")
        };

        // print the attributes ordered
        let ordered_attributes: BTreeMap<_, _> = self.attributes.iter().collect();
        for (k, v) in ordered_attributes {
            match v {
                Some(v) => write!(f, "{0}{k}={1}{v}{1}", c_inline, o.quotes())?,
                None => write!(f, "{0}{k}", c_inline)?,
            }
        }

        // end tag - continue only when not void element
        match (
            e_len > o.max_len,
            self.variant == ElementVariant::Normal && !self.children.is_empty(),
        ) {
            (true, true) => {
                write!(f, "\n")?;
                o.fmt_depth(f, depth)?;
                write!(f, ">")?
            }
            (true, false) => {
                write!(f, "\n")?;
                o.fmt_depth(f, depth)?;
                write!(f, "/>")?;
                return Ok(());
            }
            (false, true) => write!(f, ">")?,
            (false, false) => {
                write!(f, "/>")?;
                return Ok(());
            }
        }

        // print single text children in the same line when not too long
        if let Some(text) = self.children.get(0).and_then(|c| c.text()) {
            if self.children.len() == 1
                && depth + o.tab_size as usize + text.len() + self.name.len() + 3 <= o.max_len
            {
                write!(f, "{}", text)?;
                write!(f, "</{0}>", self.name)?;
                return Ok(());
            }
        }

        // print the normal children
        for child in self.children.iter() {
            write!(f, "\n")?;
            child.fmt_opt(f, o, depth + o.tab_size as usize)?;
        }
        write!(f, "\n")?;
        o.fmt_depth(f, depth)?;
        write!(f, "</{0}>", self.name)?;

        Ok(())
    }
}

impl<'s> Display for Element<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_opt(f, &FormattingOptions::pretty(), 0)
    }
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
