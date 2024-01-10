use hex_simd::{AsOut, AsciiCase};

use core::mem::MaybeUninit;

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
fn as_str() {
    let src = "hello";
    let mut buf = [MaybeUninit::<u8>::uninit(); 10];
    let ans = hex_simd::encode_as_str(src.as_bytes(), buf.as_mut_slice().as_out(), AsciiCase::Lower).unwrap();
    assert_eq!(ans, "68656c6c6f");
}

#[cfg(feature = "alloc")]
#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn allocation() {
    {
        let src = "hello";

        let ans: String = hex_simd::encode_type(src, AsciiCase::Lower);
        assert_eq!(&*ans, "68656c6c6f");

        let ans: Vec<u8> = hex_simd::decode_type(ans).unwrap();
        assert_eq!(&*ans, src.as_bytes());
    }

    {
        let src = [1, 2, 3];
        let prefix = "0x";

        let mut encode_buf = prefix.to_owned();
        hex_simd::encode_append(src, &mut encode_buf, AsciiCase::Lower);

        assert_eq!(encode_buf, format!("{prefix}010203"));

        let mut decode_buf = b"123".to_vec();
        let src = &encode_buf[prefix.len()..];
        hex_simd::decode_append(src, &mut decode_buf).unwrap();

        assert_eq!(decode_buf, b"123\x01\x02\x03");
    }
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn random() {
    let ok_cases: Vec<Vec<u8>> = {
        let mut ans = Vec::new();

        for n in 0..128usize {
            dbgmsg!("generating ok case n = {}", n);

            let iter = (0..16).cycle().take(n).map(|x| char::from_digit(x, 16).unwrap() as u8);
            ans.push(iter.collect());
        }

        ans
    };

    let err_cases: Vec<Vec<u8>> = {
        vec![
            vec![0],
            vec![b'0', 0],
            vec![b'a', b'f', 0],
            vec![b'a', b'0', b'c', 0],
            vec![b'a', b'0', b'c', b'1', 0],
        ]
    };

    macro_rules! test_decode_encode {
        ($src: expr, $case: expr) => {{
            let mut decode_buf = vec![0; $src.len() / 2];
            let mut encode_buf = vec![0; $src.len()];
            let decode_buf = hex_simd::decode($src, decode_buf.as_out()).unwrap();
            let encode_buf = hex_simd::encode(decode_buf, encode_buf.as_out(), $case).unwrap();
            assert_eq!(encode_buf, $src);
        }};
    }

    macro_rules! test_decode_inplace_encode {
        ($src: expr, $case: expr) => {{
            let mut decode_buf = $src.to_owned();
            let mut encode_buf = vec![0; $src.len()];
            let decode_buf = hex_simd::decode_inplace(&mut decode_buf).unwrap();
            let encode_buf = hex_simd::encode(decode_buf, encode_buf.as_out(), $case).unwrap();
            assert_eq!(encode_buf, $src);
        }};
    }

    macro_rules! test_encode_decode {
        ($src: expr, $case: expr) => {{
            let mut encode_buf = vec![0; $src.len() * 2];
            let mut decode_buf = vec![0; $src.len()];
            let encode_buf = hex_simd::encode($src, encode_buf.as_out(), $case).unwrap();
            let decode_buf = hex_simd::decode(encode_buf, decode_buf.as_out()).unwrap();
            assert_eq!(decode_buf, $src);
        }};
    }

    macro_rules! test_encode_decode_inplace {
        ($src: expr, $case: expr) => {{
            let mut encode_buf = vec![0; $src.len() * 2];
            let encode_buf = hex_simd::encode($src, encode_buf.as_out(), $case).unwrap();
            let decode_buf = hex_simd::decode_inplace(encode_buf).unwrap();
            assert_eq!(decode_buf, $src);
        }};
    }

    for src in &ok_cases {
        // for (_, src) in ok_cases.iter().enumerate() {
        // dbgmsg!("ok case {}", i + 1);
        assert!(hex_simd::check(src).is_ok());
        if src.len() % 2 == 0 {
            test_decode_encode!(src, AsciiCase::Lower);
            test_decode_inplace_encode!(src, AsciiCase::Lower);
        } else {
            test_encode_decode!(src, AsciiCase::Upper);
            test_encode_decode_inplace!(src, AsciiCase::Lower);
        }
    }

    for src in &err_cases {
        // for (_, src) in err_cases.iter().enumerate() {
        // dbgmsg!("err case {}", i + 1);
        assert!(hex_simd::check(src).is_err());
        let mut buf = vec![0; src.len() / 2];
        assert!(hex_simd::decode(src, buf.as_out()).is_err(), "src = {src:?}");
    }

    for n in 0..128 {
        dbgmsg!("rand case n = {}", n);
        let bytes = rand_bytes(n);
        let src = bytes.as_slice();
        test_encode_decode!(src, AsciiCase::Lower);
        test_encode_decode_inplace!(src, AsciiCase::Upper);
    }
}
