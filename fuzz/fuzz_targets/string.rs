#![no_main]

mod mock;

use libfuzzer_sys::fuzz_target;
use mock::MockTime;

fuzz_target!(|data: (MockTime, &str)| {
    let (time, format) = data;
    strftime::string::strftime(&time, format);
});
