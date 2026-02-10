use vsimd::isa::detect;
use vsimd::isa::{NEON, SSE2, WASM128};
use vsimd::vector::V128;
use vsimd::SIMD128;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use vsimd::isa::SSSE3;

#[cfg(all(feature = "unstable", target_arch = "powerpc64"))]
use vsimd::isa::VSX;

use const_str::hex;

#[cfg(not(miri))]
#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn native_sum() {
    use vsimd::native::Native;

    let x: u32 = rand::random::<u32>() / 2;
    let y: u32 = rand::random::<u32>() / 2;

    const N: usize = 100;
    let a = [x; N];
    let b = [y; N];
    let mut c = [0; N];

    Native::detect().exec(|| {
        assert!(a.len() == N && b.len() == N && c.len() == N);
        for i in 0..N {
            c[i] = a[i] + b[i];
        }
    });

    assert!(c.iter().copied().all(|z| z == x + y));
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn u8x16_any_zero() {
    fn f(a: [u8; 16]) -> bool {
        let a = V128::from_bytes(a);
        if let Some(s) = detect::<SSE2>() {
            return s.u8x16_any_zero(a);
        }
        if let Some(s) = detect::<NEON>() {
            return s.u8x16_any_zero(a);
        }
        if let Some(s) = detect::<WASM128>() {
            return s.u8x16_any_zero(a);
        }
        a.as_bytes().iter().any(|&x| x == 0)
    }

    fn test(a: [u8; 16], expected: bool) {
        assert_eq!(f(a), expected);
    }

    test([0x00; 16], true);
    test([0xff; 16], false);
    test(hex!("00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F"), true);
    test(hex!("10 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F"), false);
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn u8x16_swizzle_out_of_range_produces_zero() {
    /// Reference implementation: index in 0..15 selects from `a`, anything else yields 0.
    fn swizzle_scalar(a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
        let mut out = [0u8; 16];
        for i in 0..16 {
            out[i] = if b[i] < 16 { a[b[i] as usize] } else { 0 };
        }
        out
    }

    fn f(a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
        let va = V128::from_bytes(a);
        let vb = V128::from_bytes(b);

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if let Some(s) = detect::<SSSE3>() {
            return *s.u8x16_swizzle(va, vb).as_bytes();
        }
        if let Some(s) = detect::<NEON>() {
            return *s.u8x16_swizzle(va, vb).as_bytes();
        }
        if let Some(s) = detect::<WASM128>() {
            return *s.u8x16_swizzle(va, vb).as_bytes();
        }
        #[cfg(all(feature = "unstable", target_arch = "powerpc64"))]
        if let Some(s) = detect::<VSX>() {
            return *s.u8x16_swizzle(va, vb).as_bytes();
        }

        swizzle_scalar(a, b)
    }

    fn test(a: [u8; 16], b: [u8; 16], expected: [u8; 16]) {
        let result = f(a, b);
        assert_eq!(result, expected, "a={a:02x?}, b={b:02x?}");
    }

    let data: [u8; 16] = [0x10, 0x21, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87,
                           0x98, 0xA9, 0xBA, 0xCB, 0xDC, 0xED, 0xFE, 0x0F];

    // Identity shuffle: indices 0..15 select each byte in order.
    test(
        data,
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        data,
    );

    // Reverse shuffle.
    test(
        data,
        [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
        [0x0F, 0xFE, 0xED, 0xDC, 0xCB, 0xBA, 0xA9, 0x98,
         0x87, 0x76, 0x65, 0x54, 0x43, 0x32, 0x21, 0x10],
    );

    // All out-of-range (0x80): every lane must be zero.
    test(data, [0x80; 16], [0x00; 16]);

    // All out-of-range (0xFF): every lane must be zero.
    test(data, [0xFF; 16], [0x00; 16]);

    // Boundary: index 16 is out of range, must produce zero.
    test(data, [16; 16], [0x00; 16]);

    // Mix of valid indices and 0x80 sentinels.
    test(
        data,
        [0, 0x80, 2, 0x80, 4, 0x80, 6, 0x80, 8, 0x80, 10, 0x80, 12, 0x80, 14, 0x80],
        [0x10, 0x00, 0x32, 0x00, 0x54, 0x00, 0x76, 0x00,
         0x98, 0x00, 0xBA, 0x00, 0xDC, 0x00, 0xFE, 0x00],
    );

    // Broadcast byte 0 everywhere, except lane 7 which is out of range.
    test(
        data,
        [0, 0, 0, 0, 0, 0, 0, 0x80, 0, 0, 0, 0, 0, 0, 0, 0],
        [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00,
         0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10],
    );

    // Various out-of-range values: 17, 32, 64, 128, 200, 255.
    test(
        data,
        [0, 17, 2, 32, 4, 64, 6, 128, 8, 200, 10, 255, 12, 15, 14, 0x80],
        [0x10, 0x00, 0x32, 0x00, 0x54, 0x00, 0x76, 0x00,
         0x98, 0x00, 0xBA, 0x00, 0xDC, 0x0F, 0xFE, 0x00],
    );
}
