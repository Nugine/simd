use crate::{AsciiCase, Error, OutBuf};

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

fn ok_cases() -> Vec<Vec<u8>> {
    let mut ans = Vec::new();

    for n in 0..256usize {
        dbg_msg!(@1 "generating ok case n = {}", n);

        let iter = (0..16)
            .cycle()
            .take(n)
            .map(|x| char::from_digit(x, 16).unwrap() as u8);
        ans.push(iter.collect())
    }

    ans
}

fn err_cases() -> Vec<Vec<u8>> {
    vec![
        vec![0],
        vec![b'0', 0],
        vec![b'a', b'f', 0],
        vec![b'a', b'0', b'c', 0],
        vec![b'a', b'0', b'c', b'1', 0],
    ]
}

pub fn test(
    check: impl Fn(&[u8]) -> bool,
    decode: impl for<'s, 'd> Fn(&'s [u8], OutBuf<'d>) -> Result<&'d mut [u8], Error>,
    encode: impl for<'s, 'd> Fn(&'s [u8], OutBuf<'d>, AsciiCase) -> Result<&'d mut [u8], Error>,
    decode_inplace: impl Fn(&mut [u8]) -> Result<&mut [u8], Error>,
) {
    println!();

    let ok_cases = ok_cases();
    let err_cases = err_cases();

    macro_rules! test_decode_encode {
        ($src: expr, $case: expr) => {{
            let mut decode_buf = vec![0; $src.len() / 2];
            let mut encode_buf = vec![0; $src.len()];
            let decode_buf = OutBuf::new(&mut decode_buf);
            let encode_buf = OutBuf::new(&mut encode_buf);
            let decode_buf = decode($src, decode_buf).unwrap();
            let encode_buf = encode(decode_buf, encode_buf, $case).unwrap();
            assert_eq!(encode_buf, $src);
        }};
    }

    macro_rules! test_decode_inplace_encode {
        ($src: expr, $case: expr) => {{
            let mut decode_buf = $src.to_owned();
            let mut encode_buf = vec![0; $src.len()];
            let decode_buf = decode_inplace(&mut decode_buf).unwrap();
            let encode_buf = OutBuf::new(&mut encode_buf);
            let encode_buf = encode(decode_buf, encode_buf, $case).unwrap();
            assert_eq!(encode_buf, $src);
        }};
    }

    macro_rules! test_encode_decode {
        ($src: expr, $case: expr) => {{
            let mut encode_buf = vec![0; $src.len() * 2];
            let mut decode_buf = vec![0; $src.len()];
            let encode_buf = OutBuf::new(&mut encode_buf);
            let decode_buf = OutBuf::new(&mut decode_buf);
            let encode_buf = encode($src, encode_buf, $case).unwrap();
            let decode_buf = decode(encode_buf, decode_buf).unwrap();
            assert_eq!(decode_buf, $src);
        }};
    }

    macro_rules! test_encode_decode_inplace {
        ($src: expr, $case: expr) => {{
            let mut encode_buf = vec![0; $src.len() * 2];
            let encode_buf = OutBuf::new(&mut encode_buf);
            let encode_buf = encode($src, encode_buf, $case).unwrap();
            let decode_buf = decode_inplace(encode_buf).unwrap();
            assert_eq!(decode_buf, $src);
        }};
    }

    for (i, src) in ok_cases.iter().enumerate() {
        dbg_msg!(@1 "ok case {}", i + 1);
        assert!(check(src));
        if src.len() % 2 == 0 {
            test_decode_encode!(src, AsciiCase::Lower);
            test_decode_inplace_encode!(src, AsciiCase::Lower);
        } else {
            test_encode_decode!(src, AsciiCase::Upper);
            test_encode_decode_inplace!(src, AsciiCase::Upper);
        }
    }

    for (i, src) in err_cases.iter().enumerate() {
        dbg_msg!(@1 "err case {}", i + 1);
        assert!(!check(src));
        let mut buf = vec![0; src.len() / 2];
        let buf = OutBuf::new(&mut buf);
        assert!(decode(src, buf).is_err())
    }
}
