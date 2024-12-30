mod dom;
mod error;
mod extractor;
mod html;
mod scorer;
mod utils;

pub use dom::{RcDom, SerializableHandle};
pub use error::ReadabilityError;
pub use extractor::{
    extract, extract_content, extract_text, ExtractOptions, ParseOptions, Readable,
};
pub use scorer::{Scorer, ScorerOptions};
