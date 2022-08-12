use std::fmt::{self, Write};
use std::io;

/// A `Cursor` contains a slice of a buffer.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    /// Slice representing the remaining data to be read
    remaining: &'a [u8],
}

impl<'a> Cursor<'a> {
    /// Construct a new `Cursor` from remaining data
    pub fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    /// Returns remaining data
    pub fn remaining(&self) -> &'a [u8] {
        self.remaining
    }

    /// Returns the next byte
    pub fn next(&mut self) -> Option<u8> {
        match self.remaining {
            [first, tail @ ..] => {
                self.remaining = tail;
                Some(*first)
            }
            _ => None,
        }
    }

    /// Read bytes if the remaining data is prefixed by the provided tag
    pub fn read_optional_tag(&mut self, tag: &[u8]) -> bool {
        if self.remaining.starts_with(tag) {
            self.read_exact(tag.len());
            true
        } else {
            false
        }
    }

    /// Read bytes as long as the provided predicate is true
    pub fn read_while<F: Fn(&u8) -> bool>(&mut self, f: F) -> &'a [u8] {
        match self.remaining.iter().position(|x| !f(x)) {
            None => self.read_exact(self.remaining.len()),
            Some(position) => self.read_exact(position),
        }
    }

    /// Read bytes until the provided predicate is true
    pub fn read_until<F: Fn(&u8) -> bool>(&mut self, f: F) -> &'a [u8] {
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

/// A `SizeLimiter` limits the maximum amount a writer can write.
pub struct SizeLimiter<'a> {
    /// Inner writer
    inner: &'a mut dyn io::Write,
    /// Size limit
    size_limit: usize,
    /// Current write count
    count: usize,
}

impl<'a> SizeLimiter<'a> {
    /// Construct a new `SizeLimiter`
    pub fn new(inner: &'a mut dyn io::Write, max_size: usize) -> Self {
        Self {
            inner,
            size_limit: max_size,
            count: 0,
        }
    }
}

impl<'a> io::Write for SizeLimiter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let write_limit = buf.len().min(self.size_limit - self.count);
        let written = self.inner.write(&buf[..write_limit])?;
        self.count += written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Wrapper struct for converting an ASCII buffer to uppercase.
#[derive(Debug)]
pub struct Upper<'a>(&'a [u8]);

impl<'a> Upper<'a> {
    /// Construct a new `Upper` wrapper
    pub fn new(buf: &'a [u8]) -> Self {
        Self(buf)
    }
}

impl fmt::Display for Upper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &x in self.0 {
            f.write_char(x.to_ascii_uppercase().into())?;
        }
        Ok(())
    }
}

/// Wrapper struct for converting an ASCII buffer to lowercase.
#[derive(Debug)]
pub struct Lower<'a>(&'a [u8]);

impl<'a> Lower<'a> {
    /// Construct a new `Lower` wrapper
    pub fn new(buf: &'a [u8]) -> Self {
        Self(buf)
    }
}

impl fmt::Display for Lower<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &x in self.0 {
            f.write_char(x.to_ascii_lowercase().into())?;
        }
        Ok(())
    }
}
