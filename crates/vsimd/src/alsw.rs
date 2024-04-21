// ALSW: Avgr, Lookup, Saturating_add, Wrapping_add
// Inspired by <https://gist.github.com/aqrit/a2ccea48d7cac7e9d4d99f19d4759666>
//

use crate::pod::POD;
use crate::table::u8x16xn_lookup;
use crate::vector::{V128, V256};
use crate::Scalable;

use core::ops::Not;

#[inline]
#[must_use]
pub const fn lookup(lut: &[u8; 16], x: u8) -> u8 {
    if x < 0x80 {
        lut[(x & 0x0f) as usize]
    } else {
        0
    }
}

#[inline]
#[must_use]
pub const fn avgr(a: u8, b: u8) -> u8 {
    ((a as u16 + b as u16 + 1) >> 1) as u8
}

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
pub struct AlswLut<V> {
    pub hash: V,
    pub offset: V,
}

impl AlswLut<V128> {
    #[inline]
    #[must_use]
    pub const fn x2(self) -> AlswLut<V256> {
        AlswLut {
            hash: self.hash.x2(),
            offset: self.offset.x2(),
        }
    }
}

#[inline(always)]
pub fn check_ascii_xn<S: Scalable<V>, V: POD>(s: S, x: V, check: AlswLut<V>) -> bool {
    let shr3 = s.u32xn_shr::<3>(x);
    let h1 = s.u8xn_avgr(shr3, u8x16xn_lookup(s, check.hash, x));
    let o1 = u8x16xn_lookup(s, check.offset, h1);
    let c1 = s.i8xn_add_sat(x, o1);
    s.u8xn_highbit_any(c1).not()
}

#[inline(always)]
pub fn decode_ascii_xn<S: Scalable<V>, V: POD>(s: S, x: V, check: AlswLut<V>, decode: AlswLut<V>) -> (V, V) {
    let shr3 = s.u32xn_shr::<3>(x);

    let h1 = u8xn_avgr(s, shr3, u8x16xn_lookup(s, check.hash, x));
    let h2 = u8xn_avgr(s, shr3, u8x16xn_lookup(s, decode.hash, x));

    let o1 = u8x16xn_lookup(s, check.offset, h1);
    let o2 = u8x16xn_lookup(s, decode.offset, h2);

    let c1 = s.i8xn_add_sat(x, o1);
    let c2 = s.u8xn_add(x, o2);

    (c1, c2)
}

// FIXME: https://github.com/rust-lang/rust/issues/124216
// TODO: workaround for SSE2
#[inline(always)]
fn u8xn_avgr<S: Scalable<V>, V: POD>(s: S, a: V, b: V) -> V {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        use crate::isa::AVX2;
        use crate::tools::transmute_copy as tc;

        use core::arch::asm;

        #[cfg(target_arch = "x86")]
        use core::arch::x86::*;

        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::*;

        if matches_isa!(S, AVX2) && is_pod_type!(V, V256) {
            return unsafe { tc(&vpavgb(tc(&a), tc(&b))) };
        }

        #[target_feature(enable = "avx")]
        unsafe fn vpavgb(a: __m256i, mut b: __m256i) -> __m256i {
            asm!(
                "vpavgb {b}, {a}, {b}",
                options(pure, nomem, nostack),
                a = in(ymm_reg) a,
                b = inout(ymm_reg) b,
            );
            b
        }
    }

    s.u8xn_avgr(a, b)
}

#[macro_export]
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

            #[inline]
            #[must_use]
            const fn check_lut() -> AlswLut<V128> {
                AlswLut {
                    hash: V128::from_bytes(Self::CHECK_HASH),
                    offset: V128::from_bytes(Self::CHECK_OFFSET),
                }
            }

            #[inline]
            #[must_use]
            const fn decode_lut() -> AlswLut<V128> {
                AlswLut {
                    hash: V128::from_bytes(Self::DECODE_HASH),
                    offset: V128::from_bytes(Self::DECODE_OFFSET),
                }
            }

            #[cfg(test)]
            fn test_check() {
                let hash = &Self::CHECK_HASH;
                let offset = &Self::CHECK_OFFSET;

                let check = |c: u8| $crate::alsw::check(hash, offset, c);

                for c in 0..=255u8 {
                    assert_eq!(check(c) < 0x80, Self::decode(c) != 0xff);
                }
            }

            #[cfg(test)]
            fn test_decode() {
                let hash = &Self::DECODE_HASH;
                let offset = &Self::DECODE_OFFSET;

                let decode = |c: u8| $crate::alsw::decode(hash, offset, c);

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
