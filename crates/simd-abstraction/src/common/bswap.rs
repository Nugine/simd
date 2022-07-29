#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "wasm32"))]
item_group! {
    use crate::scalar::Bytes16;

    pub(crate) const SHUFFLE_U32X4: &Bytes16 = &Bytes16([
        0x03, 0x02, 0x01, 0x00, 0x07, 0x06, 0x05, 0x04, //
        0x0b, 0x0a, 0x09, 0x08, 0x0f, 0x0e, 0x0d, 0x0c, //
    ]);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
item_group! {
    use crate::scalar::Bytes32;

    pub(crate) const SHUFFLE_U32X8: &Bytes32 = &Bytes32::double(SHUFFLE_U32X4.0);
}
