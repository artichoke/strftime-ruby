//! This module is a copy of the [`std::io::Write`] implementation, in order to
//! use it in a no-std context.
//!
//! [`std::io::Write`]: <https://doc.rust-lang.org/std/io/trait.Write.html>

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt;
use core::str;

use crate::Error;

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
        match self.inner.write_all(s.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.error = Err(e);
                Err(fmt::Error)
            }
        }
    }
}

/// Simplified copy of the [`std::io::Write`] trait.
///
/// [`std::io::Write`]: <https://doc.rust-lang.org/std/io/trait.Write.html>
pub(crate) trait Write {
    /// Write a buffer into this writer, returning how many bytes were written.
    fn write(&mut self, data: &[u8]) -> Result<usize, Error>;

    /// Attempts to write an entire buffer into this writer.
    fn write_all(&mut self, mut data: &[u8]) -> Result<(), Error> {
        while !data.is_empty() {
            match self.write(data)? {
                0 => return Err(Error::WriteZero),
                n => data = &data[n..],
            }
        }
        Ok(())
    }

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
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        let size = data.len().min(self.len());
        let (a, b) = core::mem::take(self).split_at_mut(size);
        a.copy_from_slice(&data[..size]);
        *self = b;
        Ok(size)
    }
}

pub(crate) struct FmtWrite<'a> {
    /// Inner writer.
    inner: &'a mut dyn fmt::Write,
}

impl<'a> FmtWrite<'a> {
    pub(crate) fn new(inner: &'a mut dyn fmt::Write) -> Self {
        Self { inner }
    }
}

impl Write for FmtWrite<'_> {
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        let data =
            str::from_utf8(data).expect("strftime::fmt::strftime should only receive UTF-8 data");
        self.inner.write_str(data)?;
        Ok(data.len())
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector, growing as
/// needed.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Write for Vec<u8> {
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.try_reserve(data.len())?;
        self.extend_from_slice(data);
        Ok(data.len())
    }
}

/// Wrapper for a [`std::io::Write`] writer.
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub(crate) struct IoWrite<'a> {
    /// Inner writer.
    inner: &'a mut dyn std::io::Write,
}

#[cfg(feature = "std")]
impl<'a> IoWrite<'a> {
    /// Construct a new `IoWrite`.
    pub(crate) fn new(inner: &'a mut dyn std::io::Write) -> Self {
        Self { inner }
    }
}

/// Write is implemented for `IoWrite` by writing to its inner writer.
#[cfg(feature = "std")]
impl Write for IoWrite<'_> {
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        Ok(self.inner.write(data)?)
    }

    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        Ok(self.inner.write_all(data)?)
    }

    fn write_fmt(&mut self, fmt_args: fmt::Arguments<'_>) -> Result<(), Error> {
        Ok(self.inner.write_fmt(fmt_args)?)
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

        let result = write!(&mut &mut [0u8; 1][..], "{S}");
        assert!(matches!(result, Err(Error::FmtError)));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_io_write() {
        let mut buf = Vec::new();

        let mut writer = IoWrite::new(&mut buf);
        writer.write_all(b"ok").unwrap();
        write!(writer, "{}", 1).unwrap();

        assert_eq!(buf, *b"ok1");
    }
}
