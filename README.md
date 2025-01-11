# lithtml
A lightweight and fast HTML/XHTML parser for Rust, designed to handle both full HTML documents and fragments.

This parser uses [Pest](https://pest.rs/) for parsing and is forked from [html-parser](https://github.com/mathiversen/html-parser).

## Features
- Parse html & xhtml (not xml processing instructions)
- Parse html-documents
- Parse html-fragments
- Parse empty documents
- Parse with the same api for both documents and fragments
- Parse custom, non-standard, elements; `<cat/>`, `<Cat/>` and `<C4-t/>`
- Removes comments
- Removes dangling elements
- Iterate over all nodes in the dom three

## Examples
Parse html document
```rust
    use lithtml::Dom;

    fn main() {
        let html = r#"
            <!doctype html>
            <html lang="en">
                <head>
                    <meta charset="utf-8">
                    <title>Html parser</title>
                </head>
                <body>
                    <h1 id="a" class="b c">Hello world</h1>
                    </h1> <!-- comments & dangling elements are ignored -->
                </body>
            </html>"#;

        assert!(Dom::parse(html).is_ok());
    }
```

Parse html fragment
```rust
    use lithtml::Dom;

    fn main() {
        let html = "<div id=cat />";
        assert!(Dom::parse(html).is_ok());
    }
```
