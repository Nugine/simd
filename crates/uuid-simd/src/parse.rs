use crate::error::{Error, ERROR};
use crate::sa_hex::{self, unhex};
use crate::spec::SIMDExt;

use simd_abstraction::isa::SimdLoad;
use simd_abstraction::scalar::Bytes32;
use simd_abstraction::tools::{read, write};

#[inline(always)]
const fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline]
pub unsafe fn parse_simple_fallback(src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let mut flag = 0;
    for i in 0..16 {
        let h1 = unhex(read(src, i * 2));
        let h2 = unhex(read(src, i * 2 + 1));
        flag |= h1 | h2;
        write(dst, i, shl4(h1) | h2);
    }
    if flag == 0xff {
        return Err(ERROR);
    }
    Ok(())
}

#[inline]
pub unsafe fn parse_simple_simd<S: SIMDExt>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let a = s.v256_load_unaligned(src);
    let ans = sa_hex::decode_u8x32(s, a).map_err(|()| ERROR)?;
    s.v128_store_unaligned(dst, ans);
    Ok(())
}

#[inline]
pub unsafe fn parse_hyphenated_fallback(src: *const u8, dst: *mut u8) -> Result<(), Error> {
    match [read(src, 8), read(src, 13), read(src, 18), read(src, 23)] {
        [b'-', b'-', b'-', b'-'] => {}
        _ => return Err(ERROR),
    }

    let mut flag = 0;
    let positions: [usize; 8] = [0, 4, 9, 14, 19, 24, 28, 32];
    for (j, i) in positions.iter().copied().enumerate() {
        let h1 = unhex(read(src, i));
        let h2 = unhex(read(src, i + 1));
        let h3 = unhex(read(src, i + 2));
        let h4 = unhex(read(src, i + 3));
        flag |= h1 | h2 | h3 | h4;
        write(dst, j * 2, shl4(h1) | h2);
        write(dst, j * 2 + 1, shl4(h3) | h4);
    }
    if flag == 0xff {
        return Err(ERROR);
    }
    Ok(())
}

#[inline]
pub unsafe fn parse_hyphenated_simd<S: SIMDExt>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    match [read(src, 8), read(src, 13), read(src, 18), read(src, 23)] {
        [b'-', b'-', b'-', b'-'] => {}
        _ => return Err(ERROR),
    }

    const SWIZZLE: &Bytes32 = &Bytes32([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, //
        0x09, 0x0a, 0x0b, 0x0c, 0x0e, 0x0f, 0x80, 0x80, //
        0x03, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0a, 0x0b, //
        0x0c, 0x0d, 0x0e, 0x0f, 0x80, 0x80, 0x80, 0x80, //
    ]);

    let a0 = s.v256_load_unaligned(src);
    let a1 = s.u8x16x2_swizzle(a0, s.load(SWIZZLE));
    let a2 = s.i16x16_set_lane7(a1, src.add(16).cast::<i16>().read_unaligned());
    let a3 = s.i32x8_set_lane7(a2, src.add(32).cast::<i32>().read_unaligned());
    let ans = sa_hex::decode_u8x32(s, a3).map_err(|()| ERROR)?;
    s.v128_store_unaligned(dst, ans);
    Ok(())
}
