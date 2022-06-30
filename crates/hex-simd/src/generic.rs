#![allow(unused_macros, clippy::missing_safety_doc, missing_docs)]

use crate::fallback;
use crate::sa_hex;
use crate::{AsciiCase, Error, OutBuf, ERROR};

use core::slice;

use simd_abstraction::tools::{Bytes16, Bytes32, Load};
use simd_abstraction::traits::SIMD256;

macro_rules! specialize_for {
    ($feature:literal, $ty: ty) => {
        use crate::{AsciiCase, Error, OutBuf};
        use simd_abstraction::traits::InstructionSet;

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn check(src: &[u8]) -> bool {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::check(s, src)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn encode<'s, 'd>(
            src: &'s [u8],
            dst: OutBuf<'d>,
            case: AsciiCase,
        ) -> Result<&'d mut [u8], Error> {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::encode(s, src, dst, case)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn decode<'s, 'd>(
            src: &'s [u8],
            dst: OutBuf<'d>,
        ) -> Result<&'d mut [u8], Error> {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::decode(s, src, dst)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn decode_inplace(buf: &mut [u8]) -> Result<&mut [u8], Error> {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::decode_inplace(s, buf)
        }
    };
}

#[inline]
pub fn check<S: SIMD256>(s: S, src: &[u8]) -> bool {
    let (prefix, chunks, suffix) = unsafe { src.align_to::<Bytes32>() };
    if !fallback::check(prefix) {
        return false;
    }
    for chunk in chunks {
        if !sa_hex::check_u8x32(s, s.load(chunk)) {
            return false;
        }
    }
    if !fallback::check(suffix) {
        return false;
    }
    true
}

#[inline]
pub fn encode<'s, 'd, S>(
    s: S,
    src: &'s [u8],
    mut dst: OutBuf<'d>,
    case: AsciiCase,
) -> Result<&'d mut [u8], Error>
where
    S: SIMD256,
{
    if dst.len() / 2 < src.len() {
        return Err(ERROR);
    }
    unsafe {
        let dst = dst.as_mut_ptr();
        encode_unchecked(s, src, dst, case);
        Ok(slice::from_raw_parts_mut(dst, src.len() * 2))
    }
}

unsafe fn encode_unchecked<S: SIMD256>(s: S, src: &[u8], dst: *mut u8, case: AsciiCase) {
    let (fallback_table, simd_lut) = match case {
        AsciiCase::Lower => (fallback::FULL_LOWER_TABLE, sa_hex::ENCODE_LOWER_LUT),
        AsciiCase::Upper => (fallback::FULL_UPPER_TABLE, sa_hex::ENCODE_UPPER_LUT),
    };
    let mut cur: *mut u8 = dst;
    let (prefix, chunks, suffix) = src.align_to::<Bytes16>();
    if !prefix.is_empty() {
        fallback::encode_unchecked(prefix, cur, fallback_table);
        cur = cur.add(prefix.len() * 2);
    }
    let lut = s.load(simd_lut);
    for chunk in chunks {
        let ans = sa_hex::encode_u8x16(s, s.load(chunk), lut);
        s.v256_store_unaligned(cur, ans);
        cur = cur.add(32);
    }
    if !suffix.is_empty() {
        fallback::encode_unchecked(suffix, cur, fallback_table);
    }
}

#[inline]
pub fn decode<'s, 'd, S>(s: S, src: &'s [u8], mut dst: OutBuf<'d>) -> Result<&'d mut [u8], Error>
where
    S: SIMD256,
{
    let n = src.len();
    let m = n / 2;
    if !(n % 2 == 0 && dst.len() >= m) {
        return Err(ERROR);
    }

    unsafe {
        let src = src.as_ptr();
        let dst = dst.as_mut_ptr();
        decode_unchecked(s, m, src, dst)?;
        Ok(slice::from_raw_parts_mut(dst, m))
    }
}

#[inline]
pub fn decode_inplace<S: SIMD256>(s: S, buf: &mut [u8]) -> Result<&mut [u8], Error> {
    let n = buf.len();
    let m = n / 2;
    if n % 2 != 0 {
        return Err(ERROR);
    }
    unsafe {
        let src = buf.as_ptr();
        let dst = buf.as_mut_ptr();
        decode_unchecked(s, m, src, dst)?;
        Ok(slice::from_raw_parts_mut(dst, m))
    }
}

/// `src` and `dst` may alias
unsafe fn decode_unchecked<S: SIMD256>(
    s: S,
    m: usize,
    mut src: *const u8,
    mut dst: *mut u8,
) -> Result<(), Error> {
    let mut cnt = m;
    while cnt >= 16 {
        let chunk = s.v256_load_unaligned(src);
        let ans = sa_hex::decode_u8x32(s, chunk).map_err(|()| ERROR)?;
        s.v128_store_unaligned(dst, ans);
        src = src.add(32);
        dst = dst.add(16);
        cnt -= 16;
    }
    fallback::decode_unchecked(cnt, src, dst)?;
    Ok(())
}
