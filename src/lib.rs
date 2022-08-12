#![cfg_attr(not(feature = "std"), no_std)]
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
mod write;

use format::TimeFormatter;

use spinoso_time::tzrs::Time;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    InvalidFormat,
    WriteZero,
    FmtError,
}

pub mod buffered {
    use super::{Error, Time, TimeFormatter};

    pub fn strftime(time: &Time, format: &[u8], mut buf: &mut [u8]) -> Result<(), Error> {
        TimeFormatter::new(time, format).fmt(&mut buf)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod bytes {
    use super::{Error, Time, TimeFormatter};

    pub fn strftime(time: &Time, format: &[u8]) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        TimeFormatter::new(time, format).fmt(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod string {
    use super::{Error, Time, TimeFormatter};

    pub fn strftime(time: &Time, format: &str) -> Result<String, Error> {
        let mut buf = Vec::new();
        TimeFormatter::new(time, format).fmt(&mut buf)?;
        Ok(String::from_utf8(buf).expect("resulting string should be valid UTF-8"))
    }
}
