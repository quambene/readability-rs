use log::LevelFilter;
use readability::{
    extractor::{extract, extract_with_scorer},
    scorer::Scorer,
};
use regex::Regex;
use rstest::rstest;
use std::{
    fs::File,
    io::{Cursor, Read},
    sync::Once,
};
use url::Url;

static LOGGER: Once = Once::new();

/// Use `cargo test -- --nocapture` to display logged warnings and debug messages.
pub fn init_logger() {
    LOGGER.call_once(|| {
        env_logger::builder()
            .filter_level(LevelFilter::Info)
            .filter_module("readability", LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .unwrap();
    });
}

#[rstest]
#[case(
    "./data/url/input.html",
    "./data/url/expected.html",
    "./data/url/expected.txt",
    "https://example.com"
)]
fn test_extract(
    #[case] input: &str,
    #[case] expected_content: &str,
    #[case] expected_text: &str,
    #[case] url: &str,
) {
    init_logger();

    let mut file = File::open(input).unwrap();
    let url = Url::parse(url).unwrap();
    let product = extract(&mut file, &url).unwrap();

    let mut file = File::open(expected_content).unwrap();
    let mut expected_content = String::new();
    file.read_to_string(&mut expected_content).unwrap();
    let expected_content = expected_content.replace(['\n', '\r'], "");
    assert_eq!(product.content, expected_content);

    let mut file = File::open(expected_text).unwrap();
    let mut expected_text = String::new();
    file.read_to_string(&mut expected_text).unwrap();
    assert_eq!(product.text, expected_text);
}

#[test]
fn test_extract_url() {
    test_extract(
        "./data/url/input.html",
        "./data/url/expected.html",
        "./data/url/expected.txt",
        "https://example.com",
    );
}

#[test]
fn test_extract_title() {
    let mut file = File::open("./data/title.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let product = extract(&mut file, &url).unwrap();
    assert_eq!(product.title, "This is title");
}

#[test]
fn test_fix_rel_links() {
    let mut file = File::open("./data/rel.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let product = extract(&mut file, &url).unwrap();
    assert_eq!(product.content, "<!DOCTYPE html><html><head><title>This is title</title></head><body><p><a href=\"https://example.com/poop\"> poop </a></p></body></html>");
}

#[test]
fn test_fix_img_links() {
    let mut file = File::open("./data/img.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let product = extract(&mut file, &url).unwrap();
    assert_eq!(product.content, "<!DOCTYPE html><html><head><title>This is title</title></head><body><p><img src=\"https://example.com/poop.png\"></p></body></html>");
}

#[test]
fn test_comment() {
    let mut file = File::open("./data/comment.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let product = extract(&mut file, &url).unwrap();
    assert_eq!(
        product.content,
        "<!DOCTYPE html><html><head><title>This is title</title></head><body></body></html>"
    );
}

#[test]
fn test_comment_custom() {
    let mut file = File::open("./data/comment.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let scorer = Scorer {
        unlikely_candidates: &Regex::new(
            "combx|community|disqus|extra|foot|header|menu|remark|rss|shoutbox|sidebar|sponsor|ad-break|agegate|pagination|pager|popup|tweet|twitter|ssba",
        )
        .unwrap(),
        ..Default::default()
    };
    let product = extract_with_scorer(&mut file, &url, &scorer).unwrap();
    assert_eq!(
        product.content,
        "My div with more than 25 characters.<p>My paragraph with more than 25 characters.</p>"
    );
}

#[test]
fn test_extract_malformed() {
    let html = r#"
        <!DOCTYPE html>
        <html>
            <head><title>Malformed HTML</title></head>
            <body>
                <h1>Header without closing tag
                <p>Paragraph with <b>bold</p>
            </body>
        "#;
    let url = Url::parse("https://example.com").unwrap();
    let mut input = Cursor::new(html);

    let result = extract(&mut input, &url);
    assert!(result.is_err());
}

#[test]
fn test_extract_empty() {
    let html: &str = "";
    let url = Url::parse("https://example.com").unwrap();
    let mut input = Cursor::new(html);

    let result = extract(&mut input, &url);
    assert!(result.is_err());
}

#[test]
fn test_extract_basic() {
    let html = r#"
        <!DOCTYPE html>
        <html>
            <head><title>Test Title</title></head>
            <body>
                <h1>Welcome</h1>
                <p>This is a test paragraph.</p>
            </body>
        </html>
        "#;
    let url = Url::parse("https://example.com").unwrap();
    let mut input = Cursor::new(html);

    let result = extract(&mut input, &url).unwrap();
    assert_eq!(result.title, "Test Title");
    assert_eq!(result.content, "<p>This is a test paragraph.</p>");
    assert_eq!(result.text, "This is a test paragraph.");
}

#[test]
fn test_extract_large_html() {
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head><title>Large HTML Test</title></head>
            <body>
                {}
            </body>
        </html>
        "#,
        "<p>Repeated content.</p>".repeat(1000)
    );
    let url = Url::parse("https://example.com").unwrap();
    let mut input = Cursor::new(html);

    let result = extract(&mut input, &url).unwrap();
    assert_eq!(result.title, "Large HTML Test");
    assert_eq!(result.text.matches("Repeated content.").count(), 1000);
}
