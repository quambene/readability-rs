extern crate readability;
extern crate regex;
extern crate url;

use readability::{extractor, scorer::Scorer};
use regex::Regex;
use std::fs::File;
use url::Url;

#[test]
fn test_extract_title() {
    let mut file = File::open("./data/title.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let product = extractor::extract(&mut file, &url).unwrap();
    assert_eq!(product.title, "This is title");
}

#[test]
fn test_fix_rel_links() {
    let mut file = File::open("./data/rel.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let product = extractor::extract(&mut file, &url).unwrap();
    assert_eq!(product.content, "<!DOCTYPE html><html><head><title>This is title</title></head><body><p><a href=\"https://example.com/poop\"> poop </a></p></body></html>");
}

#[test]
fn test_fix_img_links() {
    let mut file = File::open("./data/img.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let product = extractor::extract(&mut file, &url).unwrap();
    assert_eq!(product.content, "<!DOCTYPE html><html><head><title>This is title</title></head><body><p><img src=\"https://example.com/poop.png\"></p></body></html>");
}

#[test]
fn test_comment() {
    let mut file = File::open("./data/comment.html").unwrap();
    let url = Url::parse("https://example.com").unwrap();
    let product = extractor::extract(&mut file, &url).unwrap();
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
    let product = extractor::extract_with_scorer(&mut file, &url, &scorer).unwrap();
    assert_eq!(
        product.content,
        r#"<!DOCTYPE html><html><head><title>This is title</title></head><body><div class="comment">My comment with more than 20 characters.</div></body></html>"#
    );
}
