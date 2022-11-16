use crate::{AsOut, AsciiCase, Error, Out};

use rand::RngCore;

fn rand_bytes(n: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_str() {
    use core::mem::MaybeUninit;
    let src = "hello";
    let mut buf = [MaybeUninit::<u8>::uninit(); 10];
    let ans = {
        let src = src.as_bytes();
        let dst = buf.as_mut_slice().as_out();
        let case = AsciiCase::Lower;
        crate::encode_as_str(src, dst, case)
    };
    assert_eq!(ans, "68656c6c6f");
}

#[cfg(feature = "alloc")]
#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_alloc() {
    {
        let src = "hello".as_bytes();

        let ans: String = crate::encode_type(src, AsciiCase::Lower);
        assert_eq!(&*ans, "68656c6c6f");

        let ans: Vec<u8> = crate::decode_type(ans.as_bytes()).unwrap();
        assert_eq!(&*ans, src);
    }

    {
        let src = [1, 2, 3];
        let prefix = "0x";

        let mut encode_buf = prefix.to_owned();
        crate::encode_append(&src, &mut encode_buf, AsciiCase::Lower);

        assert_eq!(encode_buf, format!("{prefix}010203"));

        let mut decode_buf = b"123".to_vec();
        let src = encode_buf[prefix.len()..].as_bytes();
        crate::decode_append(src, &mut decode_buf).unwrap();

        assert_eq!(decode_buf, b"123\x01\x02\x03");
    }
}

#[cfg(miri)]
use std::io::Write as _;

macro_rules! dbgmsg {
    ($($fmt:tt)*) => {
        println!($($fmt)*);
        #[cfg(miri)]
        std::io::stdout().flush().unwrap();
    };
}

#[allow(clippy::type_complexity)]
fn safety_unit_test(
    check: fn(&[u8]) -> Result<(), Error>,
    decode: for<'s, 'd> fn(&'s [u8], Out<'d, [u8]>) -> Result<&'d mut [u8], Error>,
    encode: for<'s, 'd> fn(&'s [u8], Out<'d, [u8]>, AsciiCase) -> &'d mut [u8],
    decode_inplace: fn(&mut [u8]) -> Result<&mut [u8], Error>,
) {
    println!();

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
            let decode_buf = decode($src, decode_buf.as_out()).unwrap();
            let encode_buf = encode(decode_buf, encode_buf.as_out(), $case);
            assert_eq!(encode_buf, $src);
        }};
    }

    macro_rules! test_decode_inplace_encode {
        ($src: expr, $case: expr) => {{
            let mut decode_buf = $src.to_owned();
            let mut encode_buf = vec![0; $src.len()];
            let decode_buf = decode_inplace(&mut decode_buf).unwrap();
            let encode_buf = encode(decode_buf, encode_buf.as_out(), $case);
            assert_eq!(encode_buf, $src);
        }};
    }

    macro_rules! test_encode_decode {
        ($src: expr, $case: expr) => {{
            let mut encode_buf = vec![0; $src.len() * 2];
            let mut decode_buf = vec![0; $src.len()];
            let encode_buf = encode($src, encode_buf.as_out(), $case);
            let decode_buf = decode(encode_buf, decode_buf.as_out()).unwrap();
            assert_eq!(decode_buf, $src);
        }};
    }

    macro_rules! test_encode_decode_inplace {
        ($src: expr, $case: expr) => {{
            let mut encode_buf = vec![0; $src.len() * 2];
            let encode_buf = encode($src, encode_buf.as_out(), $case);
            let decode_buf = decode_inplace(encode_buf).unwrap();
            assert_eq!(decode_buf, $src);
        }};
    }

    for (i, src) in ok_cases.iter().enumerate() {
        dbgmsg!("ok case {}", i + 1);
        assert!(check(src).is_ok());
        if src.len() % 2 == 0 {
            test_decode_encode!(src, AsciiCase::Lower);
            test_decode_inplace_encode!(src, AsciiCase::Lower);
        } else {
            test_encode_decode!(src, AsciiCase::Upper);
            test_encode_decode_inplace!(src, AsciiCase::Lower);
        }
    }

    for (i, src) in err_cases.iter().enumerate() {
        dbgmsg!("err case {}", i + 1);
        assert!(check(src).is_err());
        let mut buf = vec![0; src.len() / 2];
        assert!(decode(src, buf.as_out()).is_err(), "src = {src:?}");
    }

    for n in 0..128 {
        dbgmsg!("rand case n = {}", n);
        let bytes = rand_bytes(n);
        let src = bytes.as_slice();
        test_encode_decode!(src, AsciiCase::Lower);
        test_encode_decode_inplace!(src, AsciiCase::Upper);
    }
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_safety() {
    safety_unit_test(crate::check, crate::decode, crate::encode, crate::decode_inplace);
}
