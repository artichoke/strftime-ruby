//! Some useful types.

use super::write::Write;
use crate::Error;

/// A `GenericCursor` contains a generic slice from a buffer.
#[derive(Debug, Clone)]
pub(crate) struct GenericCursor<'a, T: ?Sized> {
    /// Slice representing the remaining data to be read.
    remaining: &'a T,
}

/// Alias for a cursor operating on a byte slice.
type ByteCursor<'a> = GenericCursor<'a, [u8]>;

/// Alias for a cursor operating on a string slice.
type StrCursor<'a> = GenericCursor<'a, str>;

impl<'a, T: ?Sized> GenericCursor<'a, T> {
    /// Construct a new `GenericCursor` from remaining data.
    pub(crate) fn new(remaining: &'a T) -> Self {
        Self { remaining }
    }
}

impl<'a> ByteCursor<'a> {
    /// Read exactly `count` bytes.
    fn read_exact(&mut self, count: usize) -> &'a [u8] {
        let (result, remaining) = self.remaining.split_at(count);
        self.remaining = remaining;
        result
    }
}

/// Methods for a generic cursor.
pub(crate) trait Cursor<'a> {
    /// Type of the generic slice.
    type Slice: AsRef<[u8]> + 'a + ?Sized;

    /// Returns remaining data.
    fn remaining(&self) -> &'a Self::Slice;

    /// Advances the cursor.
    fn advance(&mut self);

    /// Read data as long as the provided predicate is true.
    fn read_while<F: Fn(u8) -> bool>(&mut self, f: F) -> &'a Self::Slice;

    /// Read data until the provided predicate is true.
    fn read_until<F: Fn(u8) -> bool>(&mut self, f: F) -> &'a Self::Slice {
        self.read_while(|x| !f(x))
    }

    /// Returns the first byte.
    fn first_byte(&self) -> Option<u8> {
        self.remaining().as_ref().first().copied()
    }
}

impl<'a> Cursor<'a> for ByteCursor<'a> {
    type Slice = [u8];

    fn remaining(&self) -> &'a [u8] {
        self.remaining
    }

    fn advance(&mut self) {
        if let [_, tail @ ..] = self.remaining {
            self.remaining = tail;
        }
    }

    fn read_while<F: Fn(u8) -> bool>(&mut self, f: F) -> &'a [u8] {
        match self.remaining.iter().position(|&x| !f(x)) {
            None => self.read_exact(self.remaining.len()),
            Some(position) => self.read_exact(position),
        }
    }
}

impl<'a> Cursor<'a> for StrCursor<'a> {
    type Slice = str;

    fn remaining(&self) -> &'a str {
        self.remaining
    }

    fn advance(&mut self) {
        let mut chars = self.remaining.chars();
        chars.next();
        self.remaining = chars.as_str();
    }

    fn read_while<F: Fn(u8) -> bool>(&mut self, f: F) -> &'a str {
        let remaining_before = self.remaining;
        let mut chars = self.remaining.chars();

        loop {
            let old_chars = chars.clone();

            match chars.next() {
                None => return remaining_before,
                Some(c) => {
                    if c.is_ascii() && !f(c as u8) {
                        self.remaining = old_chars.as_str();
                        return &remaining_before[..remaining_before.len() - self.remaining.len()];
                    }
                }
            }
        }
    }
}

/// A `SizeLimiter` limits the maximum amount a writer can write.
pub(crate) struct SizeLimiter<'a, T> {
    /// Inner writer.
    inner: &'a mut T,
    /// Size limit.
    size_limit: usize,
    /// Current write count.
    count: usize,
}

impl<'a, T> SizeLimiter<'a, T> {
    /// Construct a new `SizeLimiter`.
    pub(crate) fn new(inner: &'a mut T, size_limit: usize) -> Self {
        Self {
            inner,
            size_limit,
            count: 0,
        }
    }
}

impl<'a, T: Write> Write for SizeLimiter<'a, T> {
    type Slice = T::Slice;

    fn write_all(&mut self, buf: &T::Slice) -> Result<(), Error> {
        let buf_size = buf.as_ref().len();

        // We can't write a subslice of the buffer,
        // since it may not be valid UTF-8.
        if self.count + buf_size > self.size_limit {
            return Err(Error::FormattedStringTooLarge);
        }

        self.inner.write_all(buf)?;
        self.count += buf_size;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_read_while() {
        let mut cursor = StrCursor::new("©⓪ßéfèç0€");
        assert_eq!(cursor.read_while(|c| c != b'f'), "©⓪ßé");
        assert_eq!(cursor.read_while(|c| c != b' '), "fèç0€");
    }

    #[test]
    fn test_str_read_until() {
        let mut cursor = StrCursor::new("©⓪ßéfèç0€");
        assert_eq!(cursor.read_until(|c| c == b'f'), "©⓪ßé");
        assert_eq!(cursor.read_until(|c| c == b' '), "fèç0€");
    }

    #[test]
    fn test_byte_read_while() {
        let mut cursor = ByteCursor::new("©⓪ßéfèç0€".as_bytes());
        assert_eq!(cursor.read_while(|c| c != b'f'), "©⓪ßé".as_bytes());
        assert_eq!(cursor.read_while(|c| c != b' '), "fèç0€".as_bytes());
    }

    #[test]
    fn test_byte_read_until() {
        let mut cursor = ByteCursor::new("©⓪ßéfèç0€".as_bytes());
        assert_eq!(cursor.read_until(|c| c == b'f'), "©⓪ßé".as_bytes());
        assert_eq!(cursor.read_until(|c| c == b' '), "fèç0€".as_bytes());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_size_limiter_utf_8() {
        use alloc::string::String;

        let mut buf = String::new();
        let mut f = SizeLimiter::new(&mut buf, 2);

        assert_eq!(write!(f, "a"), Ok(()));
        assert_eq!(write!(f, "€"), Err(Error::FormattedStringTooLarge));
        assert_eq!(buf, "a");
    }
}
