#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut compressed_data = Vec::<u8>::new();
    let mut writer = Cursor::new(&mut compressed_data);
    Packbits::default().write_to(&mut writer, data).unwrap();
});
