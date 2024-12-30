pub fn fill_bytes(mut bytes: &mut [u8]) {
    while !bytes.is_empty() {
        let res = twizzler_rt_abi::random::twz_rt_get_random(
            // SAFETY: `MaybeUninit<T>` is guaranteed to be layout-compatible with `T`.
            &mut *(slice as *mut [u8] as *mut [MaybeUninit<u8>]),
            twizzler_rt_abi::random::GetRandomFlags::empty(),
        );
        if res == 0 {
            panic!("failed to fill entropy bytes");
        }
        bytes = &mut bytes[res..];
    }
}
