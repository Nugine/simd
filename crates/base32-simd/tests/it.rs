use base32_simd::{AsOut, Base32};
use base32_simd::{BASE32, BASE32HEX, BASE32HEX_NO_PAD, BASE32_NO_PAD};

fn rand_bytes(n: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut bytes = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

#[cfg(miri)]
use std::io::Write as _;

macro_rules! dbgmsg {
    ($($fmt:tt)*) => {
        // println!($($fmt)*);
        // #[cfg(miri)]
        // std::io::stdout().flush().unwrap();
    };
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn basic() {
    let cases: &[(Base32, &str, &str)] = &[
        (BASE32, "", ""),
        (BASE32, "f", "MY======"),
        (BASE32, "fo", "MZXQ===="),
        (BASE32, "foo", "MZXW6==="),
        (BASE32, "foob", "MZXW6YQ="),
        (BASE32, "fooba", "MZXW6YTB"),
        (BASE32, "foobar", "MZXW6YTBOI======"),
        (BASE32HEX, "", ""),
        (BASE32HEX, "f", "CO======"),
        (BASE32HEX, "fo", "CPNG===="),
        (BASE32HEX, "foo", "CPNMU==="),
        (BASE32HEX, "foob", "CPNMUOG="),
        (BASE32HEX, "fooba", "CPNMUOJ1"),
        (BASE32HEX, "foobar", "CPNMUOJ1E8======"),
    ];

    let mut buf: Vec<u8> = Vec::new();
    for &(ref base32, input, output) in cases {
        dbgmsg!("base32 = {base32:?}, input = {input:?}, output = {output:?}");

        buf.clear();
        buf.resize(base32.encoded_length(input.len()), 0);

        let ans = base32.encode_as_str(input.as_bytes(), buf.as_out()).unwrap();
        assert_eq!(ans, output);

        buf.clear();
        buf.resize(base32.decoded_length(output.as_bytes()).unwrap(), 0);

        let ans = base32.decode(output.as_bytes(), buf.as_out()).unwrap();
        assert_eq!(ans, input.as_bytes());
    }
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn special() {
    // failed random cases
    let inputs: &[&[u8]] = &[
        &[
            0xC5, 0xB2, 0xFF, 0x01, 0xEA, 0xA1, 0xCE, 0x92, //
            0x3F, 0xB5, 0x08, 0xD8, 0xBB, 0xE2, 0x80, 0xD9, //
            0xC9, 0x8C, 0x5C, 0x18, 0x75, 0x3F, 0x12, 0xAE, //
            0xD7, 0xA5, //
        ],
        &[
            0x06, 0x3A, 0x87, 0x48, 0xAB, 0xD7, 0xAB, 0xF0, //
            0xAD, 0x85, 0x39, 0x50, 0x32, 0x23, 0x43, 0xEE, //
            0x3B, 0x79, 0xF6, 0x95, 0xC9, 0x9B, 0x63, 0xE2, //
            0xAD, 0x66, 0x68, 0xB5, 0xE0, 0x2B, 0x5A, 0x81, //
            0x5F, 0x46, 0xC2, 0x3B, //
        ],
    ];

    let base32 = BASE32;

    for &input in inputs {
        let mut buf: Vec<u8> = vec![0; base32.encoded_length(input.len())];

        let ans = base32.encode(input, buf.as_out()).unwrap();
        assert!(base32.check(ans).is_ok());

        let ans = base32.decode_inplace(&mut buf).unwrap();
        assert_eq!(ans, input);
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn allocation() {
    let src = "helloworld";
    let prefix = "data:;base32,";

    let mut encode_buf = prefix.to_owned();
    BASE32.encode_append(src, &mut encode_buf);

    assert_eq!(encode_buf, format!("{prefix}NBSWY3DPO5XXE3DE"));

    let mut decode_buf = b"123".to_vec();
    let src = &encode_buf[prefix.len()..];
    BASE32.decode_append(src, &mut decode_buf).unwrap();

    assert_eq!(decode_buf, b"123helloworld");
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn random() {
    dbgmsg!();
    for n in 0..128 {
        dbgmsg!("n = {}", n);
        let bytes = rand_bytes(n);

        let test_config = [
            BASE32,           //
            BASE32HEX,        //
            BASE32_NO_PAD,    //
            BASE32HEX_NO_PAD, //
        ];

        for base32 in test_config {
            dbgmsg!("base32 = {:?}", base32);

            let mut buf = vec![0u8; base32.encoded_length(n)];
            let encoded = base32.encode(&bytes, buf.as_out()).unwrap();
            assert!(base32.check(encoded).is_ok());

            let mut buf = encoded.to_owned();
            let ans = base32.decode_inplace(&mut buf).unwrap();
            assert_eq!(ans, bytes);

            let mut buf = vec![0u8; base32.decoded_length(encoded).unwrap()];
            let ans = base32.decode(encoded, buf.as_out()).unwrap();
            assert_eq!(ans, bytes);
        }
    }
}
