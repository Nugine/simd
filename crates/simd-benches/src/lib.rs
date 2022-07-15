#![cfg_attr(feature = "unstable", feature(arm_target_feature))]

use rand::RngCore;

pub fn rand_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

use simd_abstraction::ascii::multiversion as sa_ascii_mv;

#[inline(always)]
pub fn is_ascii_ct(data: &[u8]) -> bool {
    sa_ascii_mv::is_ascii_ct::auto_indirect(data)
}
