use indoc::indoc;
use insta::{assert_json_snapshot, assert_snapshot};
use lithtml::{Dom, Result};

#[test]
fn it_can_output_json() -> Result<()> {
    assert!(Dom::parse("<div/>")?.to_json().is_ok());
    Ok(())
}

#[test]
fn it_can_output_json_pretty() -> Result<()> {
    assert!(Dom::parse("<div/>")?.to_json_pretty().is_ok());
    Ok(())
}

#[test]
fn it_can_output_complex_html_as_json() -> Result<()> {
    let html = indoc!(
        "<html lang=\"sv\">
        <head>
            <title>Här kan man va</title>
        </head>
            <body>
                <h1>Tjena världen!</h1>
                <p>Tänkte bara informera om att Sverige är bättre än Finland i ishockey.</p>
            </body>
        </html>"
    );
    let dom = Dom::parse(html)?;
    assert_json_snapshot!(dom);
    Ok(())
}

#[test]
fn it_can_output_complex_html_as_pretty_html() -> Result<()> {
    let html = indoc!(
        r#"<html lang="de">
        <head>
            <title>Hier ist ein Titel</title>
        </head>
            <body>
                <h1>Hello World!</h1>
                <!-- Don't take the next sentence too serious -->
                <p>Deutschland ist definitiv wesentlich besser als Schweden in Eishockey.</p>
                <!-- Testing long attributes -->
                <div long_attribute="Hallo Welt" other_long_attribute="Es ist wirklich schön"></div>
                <!-- Testing quotes -->
                <div cat="she says: 'mjau mjau'" horse='horse says:"pffff"' />
            </body>
        </html>"#
    );
    let dom = Dom::parse(html)?;
    let gen_html = dom.to_string();
    let dom_dom = Dom::parse(&gen_html)?;
    assert_snapshot!(dom_dom);
    Ok(())
}

#[test]
fn it_handles_text_correct() -> Result<()> {
    let html = indoc!(
        "<div>
            <a href='javascript:void();'>
				<span>B </span>
				Budget
			</a>
				<div>###BUDGET_INFO###</div>
		</div>"
    );
    let mut new_html = Dom::parse(html)?.to_string();
    new_html = Dom::parse(&new_html)?.to_string();
    new_html = Dom::parse(&new_html)?.to_string();
    assert_snapshot!(new_html);
    Ok(())
}
