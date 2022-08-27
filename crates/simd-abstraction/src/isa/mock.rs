use super::{SIMD256, SIMD512};

#[inline(always)]
pub fn simd256_vop1<S: SIMD256>(s: S, a: S::V256, f: impl Fn(S, S::V128) -> S::V128) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = (f(s, a.0), f(s, a.1));
    s.v256_from_v128x2(b.0, b.1)
}

#[inline(always)]
pub fn simd256_vop2<S: SIMD256>(s: S, a: S::V256, b: S::V256, f: impl Fn(S, S::V128, S::V128) -> S::V128) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = s.v256_to_v128x2(b);
    let c = (f(s, a.0, b.0), f(s, a.1, b.1));
    s.v256_from_v128x2(c.0, c.1)
}

#[inline(always)]
pub fn simd256_vop3<S: SIMD256>(
    s: S,
    a: S::V256,
    b: S::V256,
    c: S::V256,
    f: impl Fn(S, S::V128, S::V128, S::V128) -> S::V128,
) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = s.v256_to_v128x2(b);
    let c = s.v256_to_v128x2(c);
    let d = (f(s, a.0, b.0, c.0), f(s, a.1, b.1, c.1));
    s.v256_from_v128x2(d.0, d.1)
}

#[inline(always)]
pub fn simd256_double<S: SIMD256>(s: S, f: impl FnOnce() -> S::V128) -> S::V256 {
    let a = f();
    s.v256_from_v128x2(a, a)
}

#[inline(always)]
pub fn simd512_vop1<S: SIMD512>(s: S, a: S::V512, f: impl Fn(S, S::V256) -> S::V256) -> S::V512 {
    let a = s.v512_to_v256x2(a);
    let b = (f(s, a.0), f(s, a.1));
    s.v512_from_v256x2(b.0, b.1)
}

#[inline(always)]
pub fn simd512_vop2<S: SIMD512>(s: S, a: S::V512, b: S::V512, f: impl Fn(S, S::V256, S::V256) -> S::V256) -> S::V512 {
    let a = s.v512_to_v256x2(a);
    let b = s.v512_to_v256x2(b);
    let c = (f(s, a.0, b.0), f(s, a.1, b.1));
    s.v512_from_v256x2(c.0, c.1)
}

#[inline(always)]
pub fn simd512_double<S: SIMD512>(s: S, f: impl FnOnce() -> S::V256) -> S::V512 {
    let a = f();
    s.v512_from_v256x2(a, a)
}
