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
    for n in 0..256 {
        println!("n = {}", n);
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
            println!("    base64 = {:?}", base64);

            let encoded = base64::encode_config(&bytes, config);
            let encoded = encoded.as_bytes();

            let mut buf = vec![0u8; base64.encoded_length(n)];
            let buf = OutBuf::from_slice_mut(&mut buf);
            let ans = encode(&base64, &bytes, buf).unwrap();
            assert_eq!(ans, encoded);
            println!("    encoding ... ok");

            let mut buf = vec![0u8; n];
            let buf = OutBuf::from_slice_mut(&mut buf);
            let ans = decode(&base64, &*ans, buf).unwrap();
            assert_eq!(ans, bytes);
            println!("    decoding ... ok");
        }
    }
}
