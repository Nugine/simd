#![allow(missing_docs)]
#![allow(clippy::missing_safety_doc)]

macro_rules! unit_tests_for {
    ($feature:literal) => {
        #[cfg(target_feature = $feature)]
        #[test]
        fn test() {
            crate::tests::test(
                |base64, src, dst| unsafe { encode(base64, src, dst) },
                |base64, src, dst| unsafe { decode(base64, src, dst) },
                |base64, buf| unsafe { decode_inplace(base64, buf) },
                |data| unsafe { find_non_ascii_whitespace(data) },
            );
        }
    };
    (@wasm, $feature:literal) => {
        #[cfg(test)]
        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        #[cfg(test)]
        #[cfg(target_feature = $feature)]
        #[wasm_bindgen_test::wasm_bindgen_test]
        fn test() {
            crate::tests::test(
                |base64, src, dst| unsafe { encode(base64, src, dst) },
                |base64, src, dst| unsafe { decode(base64, src, dst) },
                |base64, buf| unsafe { decode_inplace(base64, buf) },
                |data| unsafe { find_non_ascii_whitespace(data) },
            );
        }
    };
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86 {
    pub mod avx2 {
        specialize_for!("avx2", simd_abstraction::arch::x86::AVX2);
        unit_tests_for!("avx2");
    }
    pub mod sse41 {
        specialize_for!("sse4.1", simd_abstraction::arch::x86::SSE41);
        unit_tests_for!("sse4.1");
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    pub mod simd128 {
        specialize_for!("simd128", simd_abstraction::arch::wasm::SIMD128);
        unit_tests_for!(@wasm, "simd128");
    }
}

#[cfg(all(
    feature = "unstable",
    any(target_arch = "arm", target_arch = "aarch64",)
))]
pub mod arm {
    pub mod neon {
        #[cfg(target_arch = "arm")]
        specialize_for!("neon", simd_abstraction::arch::arm::NEON);

        #[cfg(target_arch = "aarch64")]
        specialize_for!("neon", simd_abstraction::arch::aarch64::NEON);

        unit_tests_for!("neon");
    }
}
