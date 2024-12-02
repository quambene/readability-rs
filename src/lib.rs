#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate markup5ever_rcdom;
extern crate regex;
#[cfg(feature = "reqwest")]
extern crate reqwest;
extern crate url;

pub mod dom;
pub mod error;
pub mod extractor;
pub mod scorer;
