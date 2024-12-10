#[cfg(feature = "reqwest")]
use reqwest;
use std::borrow::Cow;
use std::io;
use thiserror::Error;
use url;

#[derive(Debug, Error)]
pub enum ReadabilityError {
    #[cfg(feature = "reqwest")]
    #[error("Network error: {0:?}")]
    Network(#[from] reqwest::Error),
    #[error("Can't parse url: {0:?}")]
    ParseUrl(#[from] url::ParseError),
    #[error("Can't parse HTML: {0:?}")]
    ParseHtml(Vec<Cow<'static, str>>),
    #[error("IO error: {0:?}")]
    IO(#[from] io::Error),
    #[error("Can't fetch url '{0}'")]
    FetchUrl(String),
    #[error("Unexpected error")]
    Unexpected,
}
