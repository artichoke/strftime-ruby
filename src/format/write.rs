//! This module defines a custom version of the [`std::io::Write`] trait, in order to
//! use it in a no-std context.
//!
//! [`std::io::Write`]: <https://doc.rust-lang.org/std/io/trait.Write.html>

use core::fmt;

use crate::Error;

/// Conversion from a string slice.
pub(crate) trait FromStr {
    /// Creates a value from a string slice.
    fn from_str(s: &str) -> &Self;
}

impl FromStr for str {
    fn from_str(s: &str) -> &Self {
        s
    }
}

impl FromStr for [u8] {
    fn from_str(s: &str) -> &Self {
        s.as_bytes()
    }
}

/// An `Adapter` implements [`core::fmt::Write`] from a [`Write`] object,
/// storing write errors instead of discarding them.
struct Adapter<'a, T: ?Sized> {
    /// Inner writer.
    inner: &'a mut T,
    /// Write result.
    error: Result<(), Error>,
}

impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.inner.write_all(T::Slice::from_str(s)) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.error = Err(e);
                Err(fmt::Error)
            }
        }
    }
}

/// Custom version of the [`std::io::Write`] trait.
///
/// [`std::io::Write`]: <https://doc.rust-lang.org/std/io/trait.Write.html>
pub(crate) trait Write {
    /// Type of the inner buffer.
    type Slice: AsRef<[u8]> + ?Sized + FromStr;

    /// Attempts to write an entire buffer into this writer.
    fn write_all(&mut self, data: &Self::Slice) -> Result<(), Error>;

    /// Writes a formatted string into this writer, returning any error
    /// encountered.
    fn write_fmt(&mut self, fmt_args: fmt::Arguments<'_>) -> Result<(), Error> {
        let mut output = Adapter {
            inner: self,
            error: Ok(()),
        };

        match fmt::write(&mut output, fmt_args) {
            Ok(()) => Ok(()),
            Err(_) if output.error.is_err() => output.error,
            Err(_) => Err(Error::FmtError),
        }
    }
}

/// Write is implemented for `&mut [u8]` by copying into the slice, overwriting
/// its data.
impl Write for &mut [u8] {
    type Slice = [u8];

    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        let size = data.len().min(self.len());
        let (a, b) = core::mem::take(self).split_at_mut(size);
        a.copy_from_slice(&data[..size]);
        *self = b;

        if size == data.len() {
            Ok(())
        } else {
            Err(Error::WriteZero)
        }
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector, growing as
/// needed.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Write for alloc::vec::Vec<u8> {
    type Slice = [u8];

    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        self.try_reserve(data.len())?;
        self.extend_from_slice(data);
        Ok(())
    }
}

/// Write is implemented for `String` by appending to the string, growing as
/// needed.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Write for alloc::string::String {
    type Slice = str;

    fn write_all(&mut self, data: &str) -> Result<(), Error> {
        self.try_reserve(data.len())?;
        self.push_str(data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_error() {
        use core::fmt;

        struct S;

        impl fmt::Display for S {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                Err(fmt::Error)
            }
        }

        let mut buf = [0u8; 1];
        assert_eq!(write!(&mut &mut buf[..], "{S}"), Err(Error::FmtError));
    }
}
