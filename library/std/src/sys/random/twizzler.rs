pub fn fill_bytes(mut bytes: &mut [u8]) {
    while !bytes.is_empty() {
        let res = twizzler_runtime_api::get_runtime().get_random(bytes, twizzler_runtime_api::GetRandomFlags::empty()).expect("failed to fill entropy bytes");
        bytes = &mut bytes[res..];
    }
}
