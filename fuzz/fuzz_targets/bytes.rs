#![no_main]

mod mock;

use libfuzzer_sys::fuzz_target;
use mock::MockTime;

fuzz_target!(|data: (MockTime, &[u8])| {
    let (time, format) = data;
    // Give each fuzzer input a 16kb buffer to write to.
    let mut buf = vec![0u8; 16 * 1024].into_boxed_slice();
    let _ignored = strftime::buffered::strftime(&time, format, &mut buf[..]);
    let _ignored = strftime::io::strftime(&time, format, &mut &mut buf[..]);
});
