#![cfg_attr(feature = "unstable", feature(arm_target_feature))]

use rand::RngCore;

pub fn rand_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

use simd_abstraction::ascii::{is_ascii_ct_fallback, is_ascii_ct_simd};

simd_abstraction::simd_dispatch! {
    is_ascii_ct = fn(data: &[u8]) -> bool,
    fallback    = is_ascii_ct_fallback,
    simd        = is_ascii_ct_simd,
}

#[inline(always)]
pub fn is_ascii_ct(data: &[u8]) -> bool {
    is_ascii_ct::auto_indirect(data)
}
