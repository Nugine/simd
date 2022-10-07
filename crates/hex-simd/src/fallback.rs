use crate::Error;

use vsimd::ascii::AsciiCase;
use vsimd::hex::unhex;
use vsimd::tools::{read, write};

#[inline(always)]
pub fn check(data: &[u8]) -> Result<(), Error> {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
    unsafe {
        if cfg!(target_feature = "sse2") {
            return crate::spec::x86::sse2_check(data);
        }
    }
    check_short(data)
}

#[inline(always)]
pub fn check_short(data: &[u8]) -> Result<(), Error> {
    // FIXME:
    // The ct version triggers incorrect auto-vectorization when avx2 is enabled.
    // https://github.com/Nugine/simd/issues/14
    // https://github.com/rust-lang/rust/issues/102709
    //

    if cfg!(target_feature = "avx2") {
        check_short_sc(data)
    } else {
        check_short_ct(data)
    }
}

#[inline(always)]
fn check_short_sc(data: &[u8]) -> Result<(), Error> {
    for &x in data {
        ensure!(unhex(x) != 0xff);
    }
    Ok(())
}

#[inline(always)]
fn check_short_ct(data: &[u8]) -> Result<(), Error> {
    let mut flag = 0;
    for &x in data {
        flag |= unhex(x);
    }
    ensure!(flag != 0xff);
    Ok(())
}

#[inline(always)]
unsafe fn encode_bits(src: *const u8, dst: *mut u8, charset: *const u8) {
    let x = src.read();
    let hi = read(charset, (x >> 4) as usize);
    let lo = read(charset, (x & 0x0f) as usize);
    write(dst, 0, hi);
    write(dst, 1, lo);
}

#[inline(always)]
pub unsafe fn encode(src: &[u8], dst: *mut u8, case: AsciiCase) {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
    {
        if cfg!(target_feature = "sse2") {
            crate::spec::x86::sse2_encode(src, dst, case);
            return;
        }
    }

    encode_long(src, dst, case);
}

pub fn charset(case: AsciiCase) -> &'static [u8; 16] {
    match case {
        AsciiCase::Lower => vsimd::hex::LOWER_CHARSET,
        AsciiCase::Upper => vsimd::hex::UPPER_CHARSET,
    }
}

unsafe fn encode_long(src: &[u8], mut dst: *mut u8, case: AsciiCase) {
    let charset = charset(case).as_ptr();

    let (mut src, len) = (src.as_ptr(), src.len());

    let end = src.add(len / 8 * 8);
    while src < end {
        let mut i = 0;
        while i < 8 {
            encode_bits(src, dst, charset);
            src = src.add(1);
            dst = dst.add(2);
            i += 1;
        }
    }
    encode_short(src, len % 8, dst, charset);
}

#[inline(always)]
pub unsafe fn encode_short(mut src: *const u8, len: usize, mut dst: *mut u8, charset: *const u8) {
    let end = src.add(len);
    while src < end {
        encode_bits(src, dst, charset);
        src = src.add(1);
        dst = dst.add(2);
    }
}

#[inline(always)]
fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline(always)]
unsafe fn decode_bits(src: *const u8, dst: *mut u8) -> u8 {
    let y1 = unhex(read(src, 0));
    let y2 = unhex(read(src, 1));
    let z = shl4(y1) | y2;
    dst.write(z);
    y1 | y2
}

#[inline(always)]
pub unsafe fn decode(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error> {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
    {
        if cfg!(target_feature = "sse2") {
            return crate::spec::x86::sse2_decode(src, len, dst);
        }
    }

    decode_long(src, len, dst)
}

#[inline(always)]
pub unsafe fn decode_long(mut src: *const u8, len: usize, mut dst: *mut u8) -> Result<(), Error> {
    let end = src.add(len / 16 * 16);
    while src < end {
        let mut flag = 0;
        let mut i = 0;
        while i < 8 {
            flag |= decode_bits(src, dst);
            src = src.add(2);
            dst = dst.add(1);
            i += 1;
        }
        ensure!(flag != 0xff);
    }
    decode_short(src, len % 16, dst)
}

#[inline(always)]
pub unsafe fn decode_short(mut src: *const u8, len: usize, mut dst: *mut u8) -> Result<(), Error> {
    let end = src.add(len);
    let mut flag = 0;
    while src < end {
        flag |= decode_bits(src, dst);
        src = src.add(2);
        dst = dst.add(1);
    }
    ensure!(flag != 0xff);
    Ok(())
}
