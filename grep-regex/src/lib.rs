extern crate grep_matcher;
#[macro_use]
extern crate log;
extern crate regex;
extern crate regex_syntax;
extern crate thread_local;

pub use ast::AstAnalysis;
pub use error::{Error, ErrorKind};
pub use matcher::{RegexCaptures, RegexMatcher, RegexMatcherBuilder};

mod ast;
mod config;
mod crlf;
mod error;
mod literal;
mod matcher;
mod strip;
mod util;
mod word;
