use std::{borrow::Cow, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadabilityError {
    #[error("Can't parse url: {0:?}")]
    ParseUrl(#[from] url::ParseError),
    #[error("Can't parse HTML: {0:?}")]
    ParseHtml(Vec<Cow<'static, str>>),
    #[error("Can't read/write HTML: {0:?}")]
    ReadWriteHtml(#[from] io::Error),
    #[error("Can't fetch url")]
    FetchUrl,
    #[error("Unexpected error")]
    Unexpected,
}
