use core::fmt;

use crate::write::Write;
use crate::Error;

/// A `Cursor` contains a slice of a buffer.
#[derive(Debug, Clone)]
pub(crate) struct Cursor<'a> {
    /// Slice representing the remaining data to be read
    remaining: &'a [u8],
}

impl<'a> Cursor<'a> {
    /// Construct a new `Cursor` from remaining data
    pub(crate) fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    /// Returns remaining data
    pub(crate) fn remaining(&self) -> &'a [u8] {
        self.remaining
    }

    /// Returns the next byte
    pub(crate) fn next(&mut self) -> Option<u8> {
        match self.remaining {
            [first, tail @ ..] => {
                self.remaining = tail;
                Some(*first)
            }
            _ => None,
        }
    }

    /// Read bytes if the remaining data is prefixed by the provided tag
    pub(crate) fn read_optional_tag(&mut self, tag: &[u8]) -> bool {
        if self.remaining.starts_with(tag) {
            self.read_exact(tag.len());
            true
        } else {
            false
        }
    }

    /// Read bytes as long as the provided predicate is true
    pub(crate) fn read_while<F: Fn(&u8) -> bool>(&mut self, f: F) -> &'a [u8] {
        match self.remaining.iter().position(|x| !f(x)) {
            None => self.read_exact(self.remaining.len()),
            Some(position) => self.read_exact(position),
        }
    }

    /// Read bytes until the provided predicate is true
    pub(crate) fn read_until<F: Fn(&u8) -> bool>(&mut self, f: F) -> &'a [u8] {
        match self.remaining.iter().position(f) {
            None => self.read_exact(self.remaining.len()),
            Some(position) => self.read_exact(position),
        }
    }

    /// Read exactly `count` bytes
    fn read_exact(&mut self, count: usize) -> &'a [u8] {
        let (result, remaining) = self.remaining.split_at(count);
        self.remaining = remaining;
        result
    }
}

/// Wrapper struct for converting an ASCII buffer to uppercase.
#[derive(Debug)]
pub(crate) struct Upper<'a>(&'a [u8]);

impl<'a> Upper<'a> {
    /// Construct a new `Upper` wrapper
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        assert!(buf.is_ascii());
        Self(buf)
    }
}

impl fmt::Display for Upper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &x in self.0 {
            fmt::Write::write_char(f, x.to_ascii_uppercase().into())?;
        }
        Ok(())
    }
}

/// Wrapper struct for converting an ASCII buffer to lowercase.
#[derive(Debug)]
pub(crate) struct Lower<'a>(&'a [u8]);

impl<'a> Lower<'a> {
    /// Construct a new `Lower` wrapper
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        assert!(buf.is_ascii());
        Self(buf)
    }
}

impl fmt::Display for Lower<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &x in self.0 {
            fmt::Write::write_char(f, x.to_ascii_lowercase().into())?;
        }
        Ok(())
    }
}

/// A `SizeLimiter` limits the maximum amount a writer can write.
pub(crate) struct SizeLimiter<'a> {
    /// Inner writer
    inner: &'a mut dyn Write,
    /// Size limit
    size_limit: usize,
    /// Current write count
    count: usize,
}

impl<'a> SizeLimiter<'a> {
    /// Construct a new `SizeLimiter`
    pub(crate) fn new(inner: &'a mut dyn Write, max_size: usize) -> Self {
        Self {
            inner,
            size_limit: max_size,
            count: 0,
        }
    }
}

impl<'a> Write for SizeLimiter<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if self.count == self.size_limit {
            return Err(Error::FormattedStringTooLarge);
        }

        let write_limit = buf.len().min(self.size_limit - self.count);
        let written = self.inner.write(&buf[..write_limit])?;
        self.count += written;
        Ok(written)
    }
}
