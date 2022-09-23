// ALSW: Avgr, Lookup, Saturating_add, Wrapping_add
// Inspired by <https://gist.github.com/aqrit/a2ccea48d7cac7e9d4d99f19d4759666>
//

use crate::algorithm::{avgr, lookup};
use crate::mask::u8x32_highbit_any;
use crate::table::u8x16x2_lookup;
use crate::{SIMD256, V128, V256};

use core::ops::Not;

#[inline]
#[must_use]
pub const fn hash(hash_lut: &[u8; 16], c: u8) -> u8 {
    avgr(0xE0 | (c >> 3), lookup(hash_lut, c))
}

#[inline]
#[must_use]
pub const fn check(hash_lut: &[u8; 16], offset: &[u8; 16], c: u8) -> u8 {
    let h = hash(hash_lut, c);
    let o = lookup(offset, h);
    (c as i8).saturating_add(o as i8) as u8
}

#[inline]
#[must_use]
pub const fn decode(hash_lut: &[u8; 16], offset: &[u8; 16], c: u8) -> u8 {
    let h = hash(hash_lut, c);
    let o = lookup(offset, h);
    c.wrapping_add(o)
}

#[derive(Debug, Clone, Copy)]
pub struct AlswLut {
    pub hash: V128,
    pub offset: V128,
}

impl AlswLut {
    #[inline]
    #[must_use]
    pub const fn x2(self) -> AlswLutX2 {
        AlswLutX2 {
            hash: V256::from_v128x2((self.hash, self.hash)),
            offset: V256::from_v128x2((self.offset, self.offset)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AlswLutX2 {
    pub hash: V256,
    pub offset: V256,
}

#[inline(always)]
pub fn check_ascii32<S: SIMD256>(s: S, x: V256, check: AlswLutX2) -> bool {
    let shr3 = s.u32x8_shr::<3>(x);
    let h1 = s.u8x32_avgr(shr3, u8x16x2_lookup(s, check.hash, x));
    let o1 = u8x16x2_lookup(s, check.offset, h1);
    let c1 = s.i8x32_add_sat(x, o1);
    u8x32_highbit_any(s, c1).not()
}

#[inline(always)]
pub fn decode_ascii32<S: SIMD256>(s: S, x: V256, check: AlswLutX2, decode: AlswLutX2) -> (V256, V256) {
    let shr3 = s.u32x8_shr::<3>(x);

    let h1 = s.u8x32_avgr(shr3, u8x16x2_lookup(s, check.hash, x));
    let h2 = s.u8x32_avgr(shr3, u8x16x2_lookup(s, decode.hash, x));

    let o1 = u8x16x2_lookup(s, check.offset, h1);
    let o2 = u8x16x2_lookup(s, decode.offset, h2);

    let c1 = s.i8x32_add_sat(x, o1);
    let c2 = s.u8x32_add(x, o2);

    (c1, c2)
}

macro_rules! impl_alsw {
    ($spec:ty) => {
        impl $spec {
            const CHECK_HASH: [u8; 16] = {
                let mut arr = [0; 16];
                let mut i = 0;
                while i < 16 {
                    let x: u8 = Self::check_hash(i as u8);
                    arr[i] = (x << 1) - 1;
                    i += 1;
                }
                arr
            };

            const CHECK_OFFSET: [u8; 16] = {
                let mut arr = [0x80; 16];
                let mut c: u8 = 255;
                loop {
                    if Self::decode(c) != 0xff {
                        let h = $crate::alsw::hash(&Self::CHECK_HASH, c);
                        arr[(h & 0x0f) as usize] = 0u8.wrapping_sub(c);
                    }
                    if c == 0 {
                        break;
                    }
                    c -= 1;
                }
                arr
            };

            const DECODE_HASH: [u8; 16] = {
                let mut arr = [0; 16];
                let mut i = 0;
                while i < 16 {
                    let x: u8 = Self::decode_hash(i as u8);
                    arr[i] = (x << 1) - 1;
                    i += 1;
                }
                arr
            };

            const DECODE_OFFSET: [u8; 16] = {
                let mut arr = [0x80; 16];
                let mut c: u8 = 255;
                loop {
                    let idx = Self::decode(c);
                    if idx != 0xff {
                        let h = $crate::alsw::hash(&Self::DECODE_HASH, c);
                        arr[(h & 0x0f) as usize] = idx.wrapping_sub(c);
                    }
                    if c == 0 {
                        break;
                    }
                    c -= 1;
                }
                arr
            };

            const fn check_lut() -> AlswLut {
                use $crate::V128;
                AlswLut {
                    hash: V128::from_bytes(Self::CHECK_HASH),
                    offset: V128::from_bytes(Self::CHECK_OFFSET),
                }
            }

            const fn decode_lut() -> AlswLut {
                use $crate::V128;
                AlswLut {
                    hash: V128::from_bytes(Self::DECODE_HASH),
                    offset: V128::from_bytes(Self::DECODE_OFFSET),
                }
            }

            #[cfg(test)]
            fn test_check() {
                let hash = &Self::CHECK_HASH;
                let offset = &Self::CHECK_OFFSET;

                let is_primary = |c: u8| Self::decode(c) != 0xff;

                let h = |c: u8| $crate::alsw::hash(hash, c);
                let check = |c: u8| $crate::alsw::check(hash, offset, c);

                $crate::algorithm::print_fn_table(is_primary, h);
                $crate::algorithm::print_fn_table(is_primary, check);

                for c in 0..=255u8 {
                    assert_eq!(check(c) < 0x80, is_primary(c));
                }
            }

            #[cfg(test)]
            fn test_decode() {
                let hash = &Self::DECODE_HASH;
                let offset = &Self::DECODE_OFFSET;

                let is_primary = |c: u8| Self::decode(c) != 0xff;

                let h = |c: u8| $crate::alsw::hash(hash, c);
                let decode = |c: u8| $crate::alsw::decode(hash, offset, c);

                $crate::algorithm::print_fn_table(is_primary, h);
                $crate::algorithm::print_fn_table(is_primary, decode);

                for c in 0..=255u8 {
                    let idx = Self::decode(c);
                    if idx != 0xff {
                        assert_eq!(decode(c), idx);
                    }
                }
            }
        }
    };
}
