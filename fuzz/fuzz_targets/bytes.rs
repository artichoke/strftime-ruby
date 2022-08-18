#![no_main]

mod mock;

use libfuzzer_sys::fuzz_target;
use mock::MockTime;

fuzz_target!(|data: (MockTime, &[u8])| {
    let (time, format) = data;
    strftime::bytes::strftime(&time, format);
});
