use crate::Result;
use options::FormattingOptions;
use pest::{iterators::Pairs, Parser};
use serde::{Deserialize, Serialize};
use std::{default::Default, fmt::Display};

use crate::error::Error;
use crate::grammar::Grammar;
use crate::Rule;

pub mod element;
pub mod formatting;
pub mod node;
pub mod options;
pub mod span;

use node::Node;

/// Document, DocumentFragment or Empty
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DomVariant {
    /// This means that the parsed html had the representation of an html document. The doctype is optional but a document should only have one root node with the name of html.
    /// Example:
    /// ```text
    /// <!doctype html>
    /// <html>
    ///     <head></head>
    ///     <body>
    ///         <h1>Hello world</h1>
    ///     </body>
    /// </html>
    /// ```
    Document,
    /// A document fragment means that the parsed html did not have the representation of a document. A fragment can have multiple root children of any name except html, body or head.
    /// Example:
    /// ```text
    /// <h1>Hello world</h1>
    /// ```
    DocumentFragment,
    /// An empty dom means that the input was empty
    Empty,
}

/// **The main struct** & the result of the parsed html
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Dom<'s> {
    /// The type of the tree that was parsed
    pub tree_type: DomVariant,

    /// All of the root children in the tree
    #[serde(borrow, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node<'s>>,

    /// A collection of all warnings during parsing
    #[serde(skip_serializing)]
    pub warnings: Vec<String>,
}

impl<'s> Default for Dom<'s> {
    fn default() -> Self {
        Self {
            tree_type: DomVariant::Empty,
            children: vec![],
            warnings: vec![],
        }
    }
}

impl<'s> Dom<'s> {
    /// Create a new empty dom element
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a dom from a html string
    pub fn parse(input: &'s str) -> Result<Self> {
        let pairs = match Grammar::parse(Rule::html, input) {
            Ok(pairs) => pairs,
            Err(error) => return Err(formatting::error_msg(error)),
        };
        Self::build_dom(pairs)
    }

    /// Create the dom from a json string
    pub fn parse_json(json: &'s str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Output the dom as a json formatted string
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Output the dom as a pretty json formatted string
    pub fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Write the dom as a html string with the given formatting options
    pub fn fmt_opt<W>(&self, f: &mut W, o: &FormattingOptions) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        for child in self.children.iter() {
            child.fmt_opt(f, o, 0)?;
            write!(f, "\n")?;
        }
        Ok(())
    }

    fn build_dom(pairs: Pairs<'s, Rule>) -> Result<Self> {
        let mut dom = Self::default();

        // NOTE: The logic is roughly as follows:
        // 1) A document containing nothing but comments is DomVariant::Empty even though it will have
        //    children in this first pass.  We fix this in the next section.  This allows us to use
        //    DomVariant::Empty to indicate "we haven't decided the type yet".
        // 2) If the type is DomVariant::Empty _so far_, then it can be changed to DomVariant::Document
        //    or DomVariant::DocumentFragment.  DomVariant is only selected in this stage if we see a
        //    DOCTYPE tag.  Comments do not change the type.
        // 3) If the type is non-empty, we don't re-set the type.  We do look for conflicts between
        //    the type and the tokens in the next stage.
        for pair in pairs {
            match pair.as_rule() {
                // A <!DOCTYPE> tag means a full-fledged document.  Note that because of the way
                // the grammar is written, we will only get this token if the <!DOCTYPE> occurs
                // before any other tag; otherwise it will be parsed as a custom tag.
                Rule::doctype => {
                    if dom.tree_type == DomVariant::Empty {
                        dom.tree_type = DomVariant::Document;
                    }
                }

                // If we see an element, build the sub-tree and add it as a child.  If we don't
                // have a document type yet (i.e. "empty"), select DocumentFragment
                Rule::node_element => match Node::build_node_element(pair, &mut dom.warnings) {
                    Ok(el) => {
                        if let Some(node) = el {
                            if dom.tree_type == DomVariant::Empty {
                                dom.tree_type = DomVariant::DocumentFragment;
                            };
                            dom.children.push(node);
                        }
                    }
                    Err(error) => {
                        dom.warnings.push(format!("{}", error));
                    }
                },

                // Similar to an element, we add it as a child and select DocumentFragment if we
                // don't already have a document type.
                Rule::node_text => {
                    if dom.tree_type == DomVariant::Empty {
                        dom.tree_type = DomVariant::DocumentFragment;
                    }
                    let text = pair.as_str();
                    if !text.trim().is_empty() {
                        dom.children.push(Node::Text(text));
                    }
                }

                // Store comments as a child, but it doesn't affect the document type selection
                // until the next phase (validation).
                Rule::node_comment => {
                    dom.children.push(Node::Comment(pair.into_inner().as_str()));
                }

                // Ignore 'end of input', which then allows the catch-all unreachable!() arm to
                // function properly.
                Rule::EOI => (),

                // This should be unreachable, due to the way the grammar is written
                _ => unreachable!("[build dom] unknown rule: {:?}", pair.as_rule()),
            };
        }

        // Implement some checks on the generated dom's data and initial type.  The type may be
        // modified in this section.
        match dom.tree_type {
            // A DomVariant::Empty can only have comments. Anything else is an error.
            DomVariant::Empty => {
                for node in &dom.children {
                    if let Node::Comment(_) = node {
                        // An "empty" document, but it has comments - this is where we cleanup the
                        // earlier assumption that a document with only comments is "empty".
                        // Really, it is a "fragment".
                        dom.tree_type = DomVariant::DocumentFragment
                    } else {
                        // Anything else (i.e. Text() or Element() ) can't happen at the top level;
                        // if we had seen one, we would have set the document type above
                        unreachable!("[build dom] empty document with an Element {:?}", node)
                    }
                }
            }

            // A DomVariant::Document can only have comments and an <HTML> node at the top level.
            // Only one <HTML> tag is permitted.
            DomVariant::Document => {
                if dom
                    .children
                    .iter()
                    .filter(|x| match x {
                        Node::Element(el) if el.name.to_lowercase() == "html" => true,
                        _ => false,
                    })
                    .count()
                    > 1
                {
                    return Err(Error::Parsing(format!("Document with multiple HTML tags",)));
                }
            }

            // A DomVariant::DocumentFragment should not have <HEAD>, or <BODY> tags at the
            // top-level.  If we find an <HTML> tag, then we consider this a Document instead (if
            // it comes before any other elements, and if there is only one <HTML> tag).
            DomVariant::DocumentFragment => {
                let mut seen_html = false;
                let mut seen_elements = false;

                for node in &dom.children {
                    match node {
                        // Nodes other than <HTML> - reject <HEAD> and <BODY>
                        Node::Element(ref el) if el.name.to_lowercase() != "html" => {
                            if el.name == "head" || el.name == "body" {
                                return Err(Error::Parsing(format!(
                                    "A document fragment should not include {}",
                                    el.name
                                )));
                            }
                            seen_elements = true;
                        }
                        // <HTML> Nodes - one (before any other elements) is okay
                        Node::Element(ref el) if el.name.to_lowercase() == "html" => {
                            if seen_html || seen_elements {
                                return Err(Error::Parsing(format!(
                                    "A document fragment should not include {}",
                                    el.name
                                )));
                            };

                            // A fragment with just an <HTML> tag is a document
                            dom.tree_type = DomVariant::Document;
                            seen_html = true;
                        }
                        // Comment() and Text() nodes are permitted at the top-level of a
                        // DocumentFragment
                        _ => (),
                    }
                }
            }
        }

        // The result is the validated tree
        Ok(dom)
    }
}

impl<'s> Display for Dom<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_opt(f, &FormattingOptions::pretty())
    }
}
