#![no_main]

mod mock;

use core::fmt;
use libfuzzer_sys::fuzz_target;
use mock::MockTime;

struct LimitedBuf<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> fmt::Write for LimitedBuf<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remaining_buf = &mut self.buf[self.pos..];

        if remaining_buf.len() < bytes.len() {
            // Signal that buffer was too small.
            return Err(fmt::Error);
        }

        remaining_buf[..bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();

        Ok(())
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
