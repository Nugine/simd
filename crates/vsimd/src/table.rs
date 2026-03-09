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

    // VSX vec_perm uses idx & 0x1f, so indices 0-15 select from lut (first vector),
    // and indices 16-31 select from the zero vector (second vector).
    // Lookup semantics: return lut[x & 0x0f] when x < 128, return 0 when x >= 128.
    // Strategy: do the lookup with masked indices, then zero out results for x >= 128.
    if matches_isa!(S, VSX) {
        let lo_nibble = s.and(x, s.u8xn_splat(0x0f));
        let result = s.u8x16xn_swizzle(lut, lo_nibble);
        // For x >= 128, viewed as signed i8, the value is negative.
        // i8xn_lt(x, 0) gives 0xFF for x >= 128, 0x00 for x < 128.
        let is_negative = s.i8xn_lt(x, s.u8xn_splat(0));
        // Zero out entries where x >= 128: result & !is_negative
        return s.andnot(result, is_negative);
    }

    unreachable!()
}
