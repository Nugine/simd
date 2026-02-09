use crate::isa::{NEON, SSSE3, VSX, WASM128};
use crate::pod::POD;
use crate::Scalable;

#[inline(always)]
pub fn u8x16xn_lookup<S, V>(s: S, lut: V, x: V) -> V
where
    S: Scalable<V>,
    V: POD,
{
    if matches_isa!(S, SSSE3) {
        return s.u8x16xn_swizzle(lut, x);
    }

    if matches_isa!(S, NEON | WASM128) {
        let idx = s.and(x, s.u8xn_splat(0x8f));
        return s.u8x16xn_swizzle(lut, idx);
    }

    // VSX vec_perm uses idx & 0x1f, so indices 16-31 select from zero vector.
    // Lookup semantics: return lut[x & 0x0f] when x < 128, return 0 when x >= 128.
    // We check only the high bit (0x80) to match SSSE3/NEON/WASM behavior.
    if matches_isa!(S, VSX) {
        let hi_bit = s.and(x, s.u8xn_splat(0x80));
        let lo_nibble = s.and(x, s.u8xn_splat(0x0f));
        // If hi_bit != 0, the byte had value >= 128 and should return 0.
        let needs_zero = s.u8xn_eq(hi_bit, s.u8xn_splat(0));
        // needs_zero: 0xff if hi_bit==0 (valid), 0x00 if hi_bit!=0 (should zero)
        let force_oob = s.andnot(s.u8xn_splat(0x10), needs_zero);
        // force_oob: 0x10 if should zero, 0x00 if valid
        let idx = s.or(lo_nibble, force_oob);
        return s.u8x16xn_swizzle(lut, idx);
    }

    unreachable!()
}
