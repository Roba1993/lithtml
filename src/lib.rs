//! [![github]](https://github.com/Roba1993/lithtml)
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! # lithtml
//! A lightweight and fast HTML/XHTML parser for Rust, designed to handle both full HTML documents and fragments.
//! This parser uses [Pest](https://pest.rs/) for parsing and is forked from [html-parser](https://github.com/mathiversen/html-parser).
//!
//! ## Features
//! - Parse html & xhtml (not xml processing instructions)
//! - Parse html-documents
//! - Parse html-fragments
//! - Parse empty documents
//! - Parse with the same api for both documents and fragments
//! - Parse custom, non-standard, elements; `<cat/>`, `<Cat/>` and `<C4-t/>`
//! - Removes comments
//! - Removes dangling elements
//! - Iterate over all nodes in the dom three
//!
//! ## Examples
//!
//! Parse html document and print as json & formatted dom
//! ```rust
//!     use lithtml::Dom;
//!
//!     fn main() {
//!         let html = r#"
//!             <!doctype html>
//!             <html lang="en">
//!                 <head>
//!                     <meta charset="utf-8">
//!                     <title>Html parser</title>
//!                 </head>
//!                 <body>
//!                     <h1 id="a" class="b c">Hello world</h1>
//!                     </h1> <!-- comments & dangling elements are ignored -->
//!                 </body>
//!             </html>"#;
//!
//!         let dom = Dom::parse(html).unwrap();
//!         println!("{}", dom.to_json_pretty().unwrap());
//!         println!("{}", dom);
//!     }
//! ```
//!
//! Parse html fragment and print as json & formatted fragment
//! ```rust
//!     use lithtml::Dom;
//!
//!     fn main() {
//!         let html = "<div id=cat />";
//!         let dom = Dom::parse(html).unwrap();
//!         println!("{}", dom.to_json_pretty().unwrap());
//!         println!("{}", dom);
//!     }
//! ```
//!
//! Print to json
//! ```rust
//!     use lithtml::{Dom, Result};
//!
//!     fn main() -> Result<()> {
//!         let html = "<div id=cat />";
//!         let json = Dom::parse(html)?.to_json_pretty()?;
//!         println!("{}", Dom::parse(html)?);
//!         Ok(())
//!     }
//! ```
//!
//! Create a dom manually and print it
//! ```rust
//! use lithtml::{Dom, Node, Result};
//!
//! fn main() -> Result<()> {
//!     let mut dom = Dom::new();
//!     dom.children.push(Node::Comment("Welcome to the test"));
//!     dom.children.push(Node::parse_json(
//!         r#"{
//!           "name": "div",
//!           "variant": "normal",
//!           "children": [
//!             {
//!               "name": "h1",
//!               "variant": "normal",
//!               "children": [
//!                 "Tjena världen!"
//!               ]
//!             },
//!             {
//!               "name": "p",
//!               "variant": "normal",
//!               "children": [
//!                 "Tänkte bara informera om att Sverige är bättre än Finland i ishockey."
//!               ]
//!             }
//!           ]
//!         }"#
//!     )?);
//!     dom.children.append(&mut Node::parse(
//!         r#"<div>Testing</div><p>Multiple elements from node</p>"#,
//!     )?);
//!
//!     println!("{}", dom);
//!     Ok(())
//! }
//! ```

#![allow(clippy::needless_doctest_main)]

mod dom;
mod error;
mod grammar;

use grammar::Rule;

pub use crate::dom::element::{Element, ElementVariant};
pub use crate::dom::node::Node;
pub use crate::dom::Dom;
pub use crate::dom::DomVariant;
pub use crate::error::Error;
pub use crate::error::Result;
