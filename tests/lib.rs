use log::LevelFilter;
use readability::{
    Scorer, {extract, extract_with_scorer},
};
use regex::Regex;
use rstest::rstest;
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
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
#[case::comments("comments", "https://example.com")]
#[case::url("url", "https://example.com")]
fn test_extract(#[case] test_name: &str, #[case] url: &str) {
    init_logger();

    let data_path = Path::new("./data").join(test_name);
    let input_path = data_path.join("input.html");
    let expected_content_path = data_path.join("expected.html");
    let expected_text_path = data_path.join("expected.txt");
    let expected_title_path = data_path.join("expected_title.txt");

    let mut file = File::open(input_path).unwrap();
    let url = Url::parse(url).unwrap();
    let product = extract(&mut file, &url).unwrap();

    let mut file = File::open(expected_content_path).unwrap();
    let mut expected_content = String::new();
    file.read_to_string(&mut expected_content).unwrap();
    let expected_content = expected_content.replace(['\n', '\r'], "");
    assert_eq!(product.content, expected_content);

    let mut file = File::open(expected_text_path).unwrap();
    let mut expected_text = String::new();
    file.read_to_string(&mut expected_text).unwrap();
    assert_eq!(product.text, expected_text);

    let mut file = File::open(expected_title_path).unwrap();
    let mut expected_title = String::new();
    file.read_to_string(&mut expected_title).unwrap();
    assert_eq!(product.title, expected_title);
}

#[rstest]
#[case::comments("comments", "https://example.com")]
fn test_extract_with_scorer(#[case] test_name: &str, #[case] url: &str) {
    init_logger();

    let data_path = Path::new("./data").join(test_name);
    let input_path = data_path.join("input.html");
    let expected_content_path = data_path.join("expected_with_scorer.html");
    let expected_text_path = data_path.join("expected_with_scorer.txt");
    let expected_title_path = data_path.join("expected_title.txt");

    let mut file = File::open(input_path).unwrap();
    let url = Url::parse(url).unwrap();
    let scorer = Scorer {
        unlikely_candidates: &Regex::new(
            "combx|community|disqus|extra|foot|header|menu|remark|rss|shoutbox|sidebar|sponsor|ad-break|agegate|pagination|pager|popup|tweet|twitter|ssba",
        )
        .unwrap(),
        ..Default::default()
    };
    let product = extract_with_scorer(&mut file, &url, &scorer).unwrap();

    let mut file = File::open(expected_content_path).unwrap();
    let mut expected_content = String::new();
    file.read_to_string(&mut expected_content).unwrap();
    let expected_content = expected_content.replace(['\n', '\r'], "");
    assert_eq!(product.content, expected_content);

    let mut file = File::open(expected_text_path).unwrap();
    let mut expected_text = String::new();
    file.read_to_string(&mut expected_text).unwrap();
    assert_eq!(product.text, expected_text);

    let mut file = File::open(expected_title_path).unwrap();
    let mut expected_title = String::new();
    file.read_to_string(&mut expected_title).unwrap();
    assert_eq!(product.title, expected_title);
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
