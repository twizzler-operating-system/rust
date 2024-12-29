pub fn fill_bytes(mut bytes: &mut [u8]) {
    while !bytes.is_empty() {
        let res = twizzler_rt_abi::random::twz_rt_get_random(
            bytes,
            twizzler_rt_abi::random::GetRandomFlags::empty(),
        );
        if res == 0 {
            panic!("failed to fill entropy bytes");
        }
        bytes = &mut bytes[res..];
    }
}
