use crate::{Base64, Error, OutBuf};

use rand::RngCore;

fn rand_bytes(n: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

pub fn test(
    encode: impl for<'s, 'd> Fn(&'_ Base64, &'s [u8], OutBuf<'d, u8>) -> Result<&'d mut [u8], Error>,
    decode: impl for<'s, 'd> Fn(&'_ Base64, &'s [u8], OutBuf<'d, u8>) -> Result<&'d mut [u8], Error>,
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

            let mut buf = vec![0u8; base64.encoded_length(n)];
            let buf = OutBuf::from_slice_mut(&mut buf);
            let ans = encode(&base64, &bytes, buf).unwrap();
            assert_eq!(ans, encoded);
            dbg_msg!(@3 "encoding ... ok");

            let mut buf = vec![0u8; n];
            let buf = OutBuf::from_slice_mut(&mut buf);
            let ans = decode(&base64, &*ans, buf).unwrap();
            assert_eq!(ans, bytes);
            dbg_msg!(@3 "decoding ... ok");
        }
    }
}
