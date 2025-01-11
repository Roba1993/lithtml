use indoc::indoc;
use lithtml::Dom;

#[test]
fn it_can_parse_simple() {
    let html = indoc!(
        r#"
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Document</title>
                    <style>
                        body {
                            background: black;
                        }

                        h1 {
                            color: white;
                        }
                    </style>
                </head>
                <body>
                    <h1>Hello world</h1>
                    <!-- There should be more text here -->
                    <script>
                        const title = document.querySelector("h1")
                        title.innerText = "Hello from script"
                    </script>
                </body>
            </html>
        "#
    );
    assert!(Dom::parse(html).is_ok());
}
