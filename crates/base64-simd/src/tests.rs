use crate::{Base64, Error, OutBuf};

use rand::RngCore;

fn rand_bytes(n: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

pub fn test(
    encode: impl for<'s, 'd> Fn(&'_ Base64, &'s [u8], OutBuf<'d>) -> Result<&'d mut [u8], Error>,
    decode: impl for<'s, 'd> Fn(&'_ Base64, &'s [u8], OutBuf<'d>) -> Result<&'d mut [u8], Error>,
    decode_inplace: impl for<'b> Fn(&'_ Base64, &'b mut [u8]) -> Result<&'b mut [u8], Error>,
    // find_non_ascii_whitespace: impl for<'a> Fn(&'a [u8]) -> usize,
) {
    #[cfg(miri)]
    use std::io::Write;

    #[cfg(miri)]
    macro_rules! dbg_msg {
        (@1 $($tt:tt)*) => {
            print!("\x1B[2K\x1B[80D");
            print!($($tt)*);
            let _ = std::io::stdout().flush();
        };
        (@2 $($tt:tt)*) => {
            print!("\x1B[1B\x1B[2K\x1B[80D");
            print!($($tt)*);
            print!("\x1B[80D\x1B[1A");
            let _ = std::io::stdout().flush();
        };
        (@3 $($tt:tt)*) => {
            print!("\x1B[2B\x1B[2K\x1B[80D");
            print!($($tt)*);
            print!("\x1B[80D\x1B[2A");
            let _ = std::io::stdout().flush();
        };
    }

    #[cfg(not(miri))]
    macro_rules! dbg_msg {
        (@1 $($tt:tt)*) => {
            println!($($tt)*);
        };
        (@2 $($tt:tt)*) => {
            println!($($tt)*);
        };
        (@3 $($tt:tt)*) => {
            println!($($tt)*);
        };
    }

    // canonicity tests
    // <https://eprint.iacr.org/2022/361>
    {
        let test_vectors = [
            ("SGVsbG8=", Some("Hello")),
            ("SGVsbG9=", None),
            ("SGVsbG9", None),
            ("SGVsbA==", Some("Hell")),
            ("SGVsbA=", None),
            ("SGVsbA", None),
            ("SGVsbA====", None),
        ];

        for (encoded, expected) in test_vectors {
            let result: _ = Base64::STANDARD.decode_to_boxed_bytes(encoded.as_bytes());
            match expected {
                Some(expected) => assert_eq!(&*result.unwrap(), expected.as_bytes()),
                None => assert!(result.is_err()),
            }
        }
    }

    // {
    //     println!();
    //     for n in 1..128 {
    //         dbg_msg!(@1 "n = {}", n);

    //         let mut bytes = vec![b'a'; n];

    //         for c in [b'\r', b' '] {
    //             dbg_msg!(@2 "c = 0x{:>02x}", c);

    //             let pos = rand::thread_rng().gen_range(0..bytes.len());
    //             bytes[pos] = c;

    //             let expected = bytes
    //                 .iter()
    //                 .position(|c| c.is_ascii_whitespace())
    //                 .unwrap_or(bytes.len());
    //             let ans = find_non_ascii_whitespace(&bytes);
    //             assert_eq!(ans, expected);

    //             bytes[pos] = b'a';

    //             dbg_msg!(@3 "find_non_ascii_whitespace ... ok");
    //         }
    //     }
    // }

    println!();
    for n in 0..128 {
        dbg_msg!(@1 "n = {}", n);
        let bytes = rand_bytes(n);

        let test_config = [
            Base64::STANDARD,
            Base64::URL_SAFE,
            Base64::STANDARD_NO_PAD,
            Base64::URL_SAFE_NO_PAD,
        ];
        let base_config = [
            base64::STANDARD,
            base64::URL_SAFE,
            base64::STANDARD_NO_PAD,
            base64::URL_SAFE_NO_PAD,
        ];

        for (base64, config) in test_config.into_iter().zip(base_config.into_iter()) {
            dbg_msg!(@2 "base64 = {:?}", base64);

            let encoded = base64::encode_config(&bytes, config);
            let encoded = encoded.as_bytes();

            {
                let mut buf = vec![0u8; base64.encoded_length(n)];
                let buf = OutBuf::new(&mut buf);
                let ans = encode(&base64, &bytes, buf).unwrap();
                assert_eq!(ans, encoded);
                dbg_msg!(@3 "encoding ... ok");
            }

            {
                let mut buf = encoded.to_owned();
                let ans = decode_inplace(&base64, &mut buf).unwrap();
                assert_eq!(ans, bytes);
                dbg_msg!(@3 "decoding inplace ... ok");
            }

            {
                let mut buf = vec![0u8; n];
                let buf = OutBuf::new(&mut buf);
                let ans = decode(&base64, encoded, buf).unwrap();
                assert_eq!(ans, bytes);
                dbg_msg!(@3 "decoding ... ok");
            }
        }

        // {
        //     let expected = bytes
        //         .iter()
        //         .position(|c| c.is_ascii_whitespace())
        //         .unwrap_or(bytes.len());
        //     let ans = find_non_ascii_whitespace(&bytes);
        //     assert_eq!(ans, expected);
        //     dbg_msg!(@3 "find_non_ascii_whitespace ... ok");
        // }
    }
}
