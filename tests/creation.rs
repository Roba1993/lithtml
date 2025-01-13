use indoc::indoc;
use insta::assert_snapshot;
use lithtml::{Dom, Node, Result};

#[test]
fn it_can_create_artefacts() -> Result<()> {
    let mut dom = Dom::new();
    dom.children.push(Node::Comment("Welcome to the test"));
    dom.children.push(Node::parse_json(indoc!(
        r#"{
          "name": "div",
          "variant": "normal",
          "children": [
            {
              "name": "h1",
              "variant": "normal",
              "children": [
                "Tjena världen!"
              ]
            },
            {
              "name": "p",
              "variant": "normal",
              "children": [
                "Tänkte bara informera om att Sverige är bättre än Finland i ishockey."
              ]
            }
          ]
        }"#
    ))?);
    dom.children.append(&mut Node::parse(
        r#"<div>Testing</div><p>Multiple elements from node</p>"#,
    )?);

    assert_snapshot!(dom);
    Ok(())
}
