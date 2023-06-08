use base64_simd::{AsOut, Base64};
use base64_simd::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};

fn rand_bytes(n: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut bytes = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

// #[cfg(miri)]
// use std::io::Write as _;

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
    let cases: &[(Base64, &str, &str)] = &[
        (STANDARD, "", ""),
        (STANDARD, "f", "Zg=="),
        (STANDARD, "fo", "Zm8="),
        (STANDARD, "foo", "Zm9v"),
        (STANDARD, "foob", "Zm9vYg=="),
        (STANDARD, "fooba", "Zm9vYmE="),
        (STANDARD, "foobar", "Zm9vYmFy"),
    ];

    let mut buf: Vec<u8> = Vec::new();
    for &(ref base64, input, output) in cases {
        buf.clear();
        buf.resize(base64.encoded_length(input.len()), 0);

        let ans = base64.encode_as_str(input.as_bytes(), buf.as_out()).unwrap();
        assert_eq!(ans, output);

        buf.clear();
        buf.resize(base64.decoded_length(output.as_bytes()).unwrap(), 0);

        let ans = base64.decode(output.as_bytes(), buf.as_out()).unwrap();
        assert_eq!(ans, input.as_bytes());
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn allocation() {
    let src = "helloworld";
    let prefix = "data:;base64,";

    let mut encode_buf = prefix.to_owned();
    STANDARD.encode_append(src, &mut encode_buf);

    assert_eq!(encode_buf, format!("{prefix}aGVsbG93b3JsZA=="));

    let mut decode_buf = b"123".to_vec();
    let src = &encode_buf[prefix.len()..];
    STANDARD.decode_append(src, &mut decode_buf).unwrap();

    assert_eq!(decode_buf, b"123helloworld");
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn random() {
    use base64::Engine as _;

    dbgmsg!();
    for n in 0..128 {
        dbgmsg!("n = {}", n);
        let bytes = rand_bytes(n);

        let test_config = [
            STANDARD,        //
            URL_SAFE,        //
            STANDARD_NO_PAD, //
            URL_SAFE_NO_PAD, //
        ];

        let base_config = {
            use base64::engine::general_purpose as gp;

            [
                gp::STANDARD,        //
                gp::URL_SAFE,        //
                gp::STANDARD_NO_PAD, //
                gp::URL_SAFE_NO_PAD, //
            ]
        };

        for (base64, config) in test_config.into_iter().zip(base_config.into_iter()) {
            dbgmsg!("base64 = {:?}", base64);

            let encoded = config.encode(&bytes);
            let encoded = encoded.as_bytes();
            assert!(base64.check(encoded).is_ok());

            {
                let mut buf = vec![0u8; base64.encoded_length(n)];
                let ans = base64.encode(&bytes, buf.as_out()).unwrap();
                assert_eq!(ans, encoded);
                assert!(base64.check(ans).is_ok());
                dbgmsg!("encoding ... ok");
            }

            {
                let mut buf = encoded.to_owned();
                let ans = base64.decode_inplace(&mut buf).unwrap();
                assert_eq!(ans, bytes);
                dbgmsg!("decoding inplace ... ok");
            }

            {
                let mut buf = vec![0u8; n];
                let ans = base64.decode(encoded, buf.as_out()).unwrap();
                assert_eq!(ans, bytes);
                dbgmsg!("decoding ... ok");
            }
        }
    }
}

/// <https://eprint.iacr.org/2022/361>
#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn canonicity() {
    let test_vectors = [
        ("SGVsbG8=", Some("Hello")),
        ("SGVsbG9=", None),
        ("SGVsbG9", None),
        ("SGVsbA==", Some("Hell")),
        ("SGVsbA=", None),
        ("SGVsbA", None),
        ("SGVsbA====", None),
    ];

    let mut buf = [0u8; 64];

    for (encoded, expected) in test_vectors {
        let base64 = STANDARD;

        let is_valid = base64.check(encoded.as_bytes()).is_ok();
        let result = base64.decode(encoded.as_bytes(), buf.as_mut_slice().as_out());

        assert_eq!(is_valid, result.is_ok());
        match expected {
            Some(expected) => assert_eq!(result.unwrap(), expected.as_bytes()),
            None => assert!(result.is_err()),
        }
    }
}

// RUSTFLAGS=-Zsanitizer=address cargo test -p base64-simd --features=parallel -- --include-ignored parallel_encode
#[cfg(all(not(miri), feature = "parallel"))]
#[test]
#[ignore]
fn parallel_encode() {
    let mut buf1 = vec![0; 100_000];
    let mut buf2 = vec![0; 100_000];
    for n in 0..50_000 {
        let src = rand_bytes(n);
        for base64 in [base64_simd::STANDARD, base64_simd::STANDARD_NO_PAD] {
            let ans1 = base64.par_encode(&src, buf1.as_out()).unwrap();
            let ans2 = base64.encode(&src, buf2.as_out()).unwrap();
            assert!(ans1 == ans2, "n = {n}");
        }
    }
}

#[cfg(feature = "alloc")]
#[test]
fn precise_decoded_length() {
    // true positive
    let tp_cases: &[(&Base64, &str, usize)] = &[
        (&STANDARD, "", 0),            //
        (&STANDARD, "YQ==", 1),        //
        (&STANDARD, "YWI=", 2),        //
        (&STANDARD, "YWJj", 3),        //
        (&STANDARD_NO_PAD, "", 0),     //
        (&STANDARD_NO_PAD, "YQ", 1),   //
        (&STANDARD_NO_PAD, "YWI", 2),  //
        (&STANDARD_NO_PAD, "YWJj", 3), //
    ];

    // false negative
    let fn_cases: &[(&Base64, &str, usize)] = &[
        (&STANDARD, "====", 1),
        (&STANDARD, "==a=", 2),
        (&STANDARD, "===a", 3),
        (&STANDARD_NO_PAD, "==", 1),
        (&STANDARD_NO_PAD, "===", 2),
        (&STANDARD_NO_PAD, "====", 3),
    ];

    // true negative
    let tn_cases: &[(&Base64, &str)] = &[
        (&STANDARD, "Y"),            //
        (&STANDARD, "YQ"),           //
        (&STANDARD, "YWI"),          //
        (&STANDARD, "YW="),          //
        (&STANDARD, "Y="),           //
        (&STANDARD, "="),            //
        (&STANDARD, "=="),           //
        (&STANDARD, "==="),          //
        (&STANDARD_NO_PAD, "="),     //
        (&STANDARD_NO_PAD, "ABCDE"), //
    ];

    for &(base64, data, expected) in tp_cases {
        assert_eq!(base64.decoded_length(data.as_ref()).unwrap(), expected);
        assert_eq!(base64.decode_to_vec(data).unwrap().len(), expected);
    }

    for &(base64, data, expected) in fn_cases {
        assert_eq!(base64.decoded_length(data.as_ref()).unwrap(), expected);
        assert!(base64.decode_to_vec(data).is_err());
    }

    // There is no false positive!

    for (base64, data) in tn_cases {
        assert!(base64.decoded_length(data.as_ref()).is_err());
        assert!(base64.decode_to_vec(data).is_err());
    }

    {
        // See <https://github.com/marshallpierce/rust-base64/issues/212>
        use rand::Rng;

        let base64 = base64_simd::STANDARD_NO_PAD;
        let data = rand::thread_rng().gen::<[u8; 32]>();
        let encoded = base64.encode_to_string(data);

        let buf: &mut [u8] = &mut [0; 32];
        assert!(base64.decode(encoded.as_ref(), buf.as_out()).is_ok());
        assert_eq!(data, buf[..]);

        let buf: &mut [u8] = &mut [0; 64];
        assert!(base64.decode(encoded.as_ref(), buf.as_out()).is_ok());
        assert_eq!(data, buf[..32]);
    }
}

#[test]
fn estimated_decoded_length() {
    let cases = [
        (0, 0), //
        (1, 3), //
        (2, 3), //
        (3, 3), //
        (4, 3), //
        (5, 6), //
        (6, 6), //
        (7, 6), //
        (8, 6), //
    ];

    for (input, expected) in cases {
        assert_eq!(base64_simd::STANDARD.estimated_decoded_length(input), expected);
        assert_eq!(base64::decoded_len_estimate(input), expected);
    }

    // no panic
    let _ = base64_simd::STANDARD.estimated_decoded_length(usize::MAX);

    // let _ = base64::decoded_len_estimate(usize::MAX); // it panics
}
