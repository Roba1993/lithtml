use std::{borrow::Cow, fmt::Display};

use crate::{
    grammar::{Grammar, Rule},
    ElementVariant, Error,
};

use super::{element::Element, formatting, options::FormattingOptions, span::SourceSpan, Result};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Node<'s> {
    Element(Element<'s>),
    #[serde(borrow)]
    Text(Cow<'s, str>),
    #[serde(borrow)]
    Comment(Cow<'s, str>),
}

impl<'s> Node<'s> {
    /// Get the text when it's a text node
    pub fn text(&self) -> Option<&str> {
        match self {
            Node::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Get the elemnt when it's a element node
    pub fn element(&self) -> Option<&Element> {
        match self {
            Node::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Get the comment when it's a comment node
    pub fn comment(&self) -> Option<&str> {
        match self {
            Node::Comment(t) => Some(t),
            _ => None,
        }
    }

    /// Create a new text node
    pub fn new_text(text: &'s str) -> Self {
        Self::Text(Cow::Borrowed(text))
    }

    /// Create a new comment node
    pub fn new_comment(comment: &'s str) -> Self {
        Self::Comment(Cow::Borrowed(comment))
    }

    /// Parse a dom from a html string
    pub fn parse(input: &'s str) -> Result<Vec<Self>> {
        let pairs = match Grammar::parse(Rule::html, input) {
            Ok(pairs) => pairs,
            Err(error) => return Err(formatting::error_msg(error)),
        };
        Self::build_nodes(pairs)
    }

    /// Create the node from a json string
    pub fn parse_json(json: &'s str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Output the node as a json formatted string
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Output the node as a pretty json formatted string
    pub fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn fmt_opt<W>(&self, f: &mut W, o: &FormattingOptions, depth: usize) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        match self {
            Node::Element(elem) => {
                elem.fmt_opt(f, o, depth)?;
            }
            Node::Text(text) => {
                o.fmt_depth(f, depth)?;
                write!(f, "{}", text.trim())?;
            }
            Node::Comment(comment) => {
                o.fmt_depth(f, depth)?;
                write!(f, "<!-- {comment} -->")?;
            }
        }

        Ok(())
    }

    fn build_nodes(pairs: Pairs<'s, Rule>) -> Result<Vec<Self>> {
        let mut nodes = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                // A <!DOCTYPE> tag means a full-fledged document.  Because it's a node, we don't use it
                Rule::doctype => (),

                // If we see an element, build the sub-tree and add it as a child.
                // Warnings are ignored
                Rule::node_element => match Self::build_node_element(pair, &mut Vec::new()) {
                    Ok(el) => {
                        if let Some(node) = el {
                            nodes.push(node);
                        }
                    }
                    Err(_) => {}
                },

                // Similar to an element, we add it as a child
                Rule::node_text => {
                    let text = pair.as_str();
                    if !text.trim().is_empty() {
                        nodes.push(Node::Text(Cow::Borrowed(text)));
                    }
                }

                // Store comments as a child
                Rule::node_comment => {
                    nodes.push(Node::Comment(Cow::Borrowed(pair.into_inner().as_str())));
                }

                // Ignore 'end of input', which then allows the catch-all unreachable!() arm to
                // function properly.
                Rule::EOI => (),

                // This should be unreachable, due to the way the grammar is written
                _ => unreachable!("[build nodes] unknown rule: {:?}", pair.as_rule()),
            };
        }

        // The result are validated nodes
        Ok(nodes)
    }

    pub(super) fn build_node_element(
        pair: Pair<'s, Rule>,
        warnings: &mut Vec<String>,
    ) -> Result<Option<Node<'s>>> {
        let source_span = {
            let pair_span = pair.as_span();
            let (start_line, start_column) = pair_span.start_pos().line_col();
            let (end_line, end_column) = pair_span.end_pos().line_col();

            SourceSpan::new(
                pair_span.as_str(),
                start_line,
                end_line,
                start_column,
                end_column,
            )
        };

        let mut element = Element {
            source_span,
            ..Element::default()
        };

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::node_element | Rule::el_raw_text => {
                    match Self::build_node_element(pair, warnings) {
                        Ok(el) => {
                            if let Some(child_element) = el {
                                element.children.push(child_element)
                            }
                        }
                        Err(error) => {
                            warnings.push(format!("{}", error));
                        }
                    }
                }
                Rule::node_text | Rule::el_raw_text_content => {
                    let text = pair.as_str();
                    if !text.trim().is_empty() {
                        element.children.push(Node::Text(Cow::Borrowed(text)));
                    }
                }
                Rule::node_comment => {
                    element
                        .children
                        .push(Node::Comment(Cow::Borrowed(pair.into_inner().as_str())));
                }
                // TODO: To enable some kind of validation we should probably align this with
                // https://html.spec.whatwg.org/multipage/syntax.html#elements-2
                // Also see element variants
                Rule::el_name | Rule::el_void_name | Rule::el_raw_text_name => {
                    element.name = Cow::Borrowed(pair.as_str());
                }
                Rule::attr => match Self::build_attribute(pair.into_inner()) {
                    Ok((attr_key, attr_value)) => {
                        match attr_key {
                            "class" => {
                                if let Some(classes) = attr_value {
                                    let classes = classes.split_whitespace().collect::<Vec<_>>();
                                    for class in classes {
                                        element.classes.push(Cow::Borrowed(class));
                                    }
                                }
                            }
                            _ => {
                                element.attributes.insert(
                                    Cow::Borrowed(attr_key),
                                    attr_value.map(|s| Cow::Borrowed(s)),
                                );
                            }
                        };
                    }
                    Err(error) => {
                        warnings.push(format!("{}", error));
                    }
                },
                Rule::el_normal_end | Rule::el_raw_text_end => {
                    element.variant = ElementVariant::Normal;
                    break;
                }
                Rule::el_dangling => (),
                Rule::EOI => (),
                _ => {
                    return Err(Error::Parsing(format!(
                        "Failed to create element at rule: {:?}",
                        pair.as_rule()
                    )))
                }
            }
        }
        if element.name != "" {
            Ok(Some(Node::Element(element)))
        } else {
            Ok(None)
        }
    }

    fn build_attribute(pairs: Pairs<'s, Rule>) -> Result<(&'s str, Option<&'s str>)> {
        let mut attribute = ("", None);
        for pair in pairs {
            match pair.as_rule() {
                Rule::attr_key => {
                    attribute.0 = pair.as_str().trim();
                }
                Rule::attr_non_quoted => {
                    attribute.1 = Some(pair.as_str().trim());
                }
                Rule::attr_quoted => {
                    let inner_pair = pair
                        .into_inner()
                        .into_iter()
                        .next()
                        .expect("attribute value");

                    match inner_pair.as_rule() {
                        Rule::attr_value => attribute.1 = Some(inner_pair.as_str()),
                        _ => {
                            return Err(Error::Parsing(format!(
                                "Failed to parse attr value: {:?}",
                                inner_pair.as_rule()
                            )))
                        }
                    }
                }
                _ => {
                    return Err(Error::Parsing(format!(
                        "Failed to parse attr: {:?}",
                        pair.as_rule()
                    )))
                }
            }
        }
        Ok(attribute)
    }
}

impl<'s> Display for Node<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_opt(f, &FormattingOptions::pretty(), 0)
    }
}

impl<'a> IntoIterator for &'a Node<'a> {
    type Item = &'a Node<'a>;
    type IntoIter = NodeIntoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        NodeIntoIterator {
            node: self,
            index: vec![],
        }
    }
}

pub struct NodeIntoIterator<'a> {
    node: &'a Node<'a>,
    // We add/remove to this vec each time we go up/down a node three
    index: Vec<(usize, &'a Node<'a>)>,
}

impl<'a> Iterator for NodeIntoIterator<'a> {
    type Item = &'a Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get first child
        let child = match self.node {
            Node::Element(ref e) => e.children.get(0),
            _ => None,
        };

        let result = match child {
            // If element has child, return child
            Some(child) => {
                self.index.push((0, self.node));
                self.node = child;
                Some(child)
            }
            // If element doesn't have a child, but is a child of another node
            None if self.index.len() > 0 => {
                let mut has_finished = false;
                let mut next_node = None;

                while !has_finished {
                    // Try to get the next sibling of the parent node
                    if let Some((sibling_index, parent)) = self.index.pop() {
                        let next_sibling = sibling_index + 1;
                        let sibling = if let Node::Element(ref e) = parent {
                            e.children.get(next_sibling)
                        } else {
                            None
                        };
                        if sibling.is_some() {
                            has_finished = true;
                            self.index.push((next_sibling, parent));
                            next_node = sibling;
                        } else {
                            continue;
                        }
                    // Break of there are no more parents
                    } else {
                        has_finished = true;
                    }
                }

                if let Some(next_node) = next_node {
                    self.node = next_node;
                }

                next_node
            }
            _ => None,
        };

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_utillity_functions() {
        let node = Node::Text(Cow::Borrowed("test"));

        assert_eq!(node.text(), Some("test"));
        assert_eq!(node.element(), None);
        assert_eq!(node.comment(), None);

        let node = Node::Element(Element::default());

        assert_eq!(node.text(), None);
        assert_eq!(node.element(), Some(&Element::default()));
        assert_eq!(node.comment(), None);

        let node = Node::Comment(Cow::Borrowed("test"));

        assert_eq!(node.text(), None);
        assert_eq!(node.element(), None);
        assert_eq!(node.comment(), Some("test"));
    }
}
