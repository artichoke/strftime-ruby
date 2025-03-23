#![no_main]

use core::fmt;

use libfuzzer_sys::fuzz_target;

mod mock;

use mock::MockTime;

struct LimitedBuf<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> fmt::Write for LimitedBuf<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len().saturating_sub(self.pos);
        let to_copy = remaining.min(bytes.len());

        if to_copy == 0 {
            return Err(fmt::Error);
        }

        self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;

        if to_copy < bytes.len() {
            Err(fmt::Error) // signal that buffer was too small
        } else {
            Ok(())
        }
    }
}

fuzz_target!(|data: (MockTime, &str)| {
    let (time, format) = data;
    // Give each fuzzer input a 16kb buffer to write to.
    let mut buf = vec![0u8; 16 * 1024].into_boxed_slice();

    let mut writer = LimitedBuf {
        buf: &mut buf[..],
        pos: 0,
    };

    let _ignored = strftime::fmt::strftime(&time, format, &mut writer);
});
