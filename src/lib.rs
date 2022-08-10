#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::cast_possible_truncation)]
#![allow(unknown_lints)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! WIP

#![doc(html_root_url = "https://docs.rs/strftime-ruby/0.1.0")]

// Ensure code blocks in `README.md` compile
#[cfg(all(doctest, feature = "std"))]
#[doc = include_str!("../README.md")]
mod readme {}

mod format;
mod utils;
mod week;

use format::TimeFormatter;

use std::io;

use spinoso_time::tzrs::Time;

#[derive(Debug)]
pub enum FormatError {
    IoError(io::Error),
    InvalidFormat,
}

impl From<io::Error> for FormatError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

pub fn strftime(time: &Time, format: &str) -> Result<Vec<u8>, FormatError> {
    let mut buf = Vec::new();
    TimeFormatter::new(time, format).fmt(&mut buf)?;
    Ok(buf)
}
