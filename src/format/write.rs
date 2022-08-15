//! This module is a copy of the [`std::io::Write`] implementation,
//! in order to use it in a no-std context.
//!
//! [`std::io::Write`]: <https://doc.rust-lang.org/std/io/trait.Write.html>

use core::fmt;

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
    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Error> {
        while !buf.is_empty() {
            match self.write(buf)? {
                0 => return Err(Error::WriteZero),
                n => buf = &buf[n..],
            }
        }
        Ok(())
    }

    /// Writes a formatted string into this writer, returning any error encountered.
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

/// Write is implemented for `&mut [u8]` by copying into the slice, overwriting its data.
impl Write for &mut [u8] {
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        let size = data.len().min(self.len());
        let (a, b) = core::mem::take(self).split_at_mut(size);
        a.copy_from_slice(&data[..size]);
        *self = b;
        Ok(size)
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector, growing as needed.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }
}
