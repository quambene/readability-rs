mod dom;
mod error;
mod extractor;
mod scorer;

pub use error::ReadabilityError;
pub use extractor::{extract, extract_with_scorer, Product};
pub use scorer::Scorer;
