#[cfg(feature = "reqwest")]
use reqwest;
use std::borrow::Cow;
use std::io;
use thiserror::Error;
use url;

#[derive(Debug, Error)]
pub enum ReadabilityError {
    #[cfg(feature = "reqwest")]
    #[error("Can't send request: {0:?}")]
    Request(#[from] reqwest::Error),
    #[error("Can't parse url: {0:?}")]
    ParseUrl(#[from] url::ParseError),
    #[error("Can't parse HTML: {0:?}")]
    ParseHtml(Vec<Cow<'static, str>>),
    #[error("Can't read HTML: {0:?}")]
    ReadHtml(#[from] io::Error),
    #[error("Can't fetch url")]
    FetchUrl,
    #[error("Unexpected error")]
    Unexpected,
}
