use crate::isa::{NEON, SSSE3, WASM128};
use crate::pod::POD;
use crate::Scalable;

#[inline(always)]
pub fn u8x16xn_lookup<S, V>(s: S, lut: V, x: V) -> V
where
    S: Scalable<V>,
    V: POD,
{
    if is_subtype!(S, SSSE3) {
        return s.u8x16xn_swizzle(lut, x);
    }

    if is_subtype!(S, NEON | WASM128) {
        let idx = s.and(x, s.u8xn_splat(0x8f));
        return s.u8x16xn_swizzle(lut, idx);
    }

    unreachable!()
}
