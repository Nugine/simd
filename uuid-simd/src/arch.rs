#![allow(clippy::missing_safety_doc, missing_docs)]

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86 {
    pub mod avx2 {
        specialize_for!("avx2", simd_abstraction::arch::x86::AVX2);

        #[cfg(target_feature = "avx2")]
        #[test]
        fn test_parse() {
            crate::tests::test_parse_ok(|s| unsafe { parse(s.as_bytes()) });
            crate::tests::test_parse_err(|s| unsafe { parse(s.as_bytes()) });
        }

        #[cfg(target_feature = "avx2")]
        #[test]
        fn test_format() {
            crate::tests::test_format_simple(|src, upper| unsafe {
                format_simple(src, upper) //
            });
            crate::tests::test_format_hypenated(|src, upper| unsafe {
                format_hyphenated(src, upper) //
            });
        }
    }

    pub mod sse41 {
        specialize_for!("sse4.1", simd_abstraction::arch::x86::SSE41);

        #[cfg(target_feature = "sse4.1")]
        #[test]
        fn test_parse() {
            crate::tests::test_parse_ok(|s| unsafe { parse(s.as_bytes()) });
            crate::tests::test_parse_err(|s| unsafe { parse(s.as_bytes()) });
        }

        #[cfg(target_feature = "sse4.1")]
        #[test]
        fn test_format() {
            crate::tests::test_format_simple(|src, upper| unsafe {
                format_simple(src, upper) //
            });
            crate::tests::test_format_hypenated(|src, upper| unsafe {
                format_hyphenated(src, upper) //
            });
        }
    }
}

#[cfg(all(
    feature = "unstable",
    any(target_arch = "arm", target_arch = "aarch64")
))]
pub mod arm {
    pub mod neon {
        #[cfg(target_arch = "arm")]
        specialize_for!("neon", simd_abstraction::arch::arm::NEON);

        #[cfg(target_arch = "aarch64")]
        specialize_for!("neon", simd_abstraction::arch::aarch64::NEON);

        #[cfg(target_feature = "neon")]
        #[test]
        fn test_parse() {
            crate::tests::test_parse_ok(|s| unsafe { parse(s.as_bytes()) });
            crate::tests::test_parse_err(|s| unsafe { parse(s.as_bytes()) });
        }

        #[cfg(target_feature = "neon")]
        #[test]
        fn test_format() {
            crate::tests::test_format_simple(|src, upper| unsafe {
                format_simple(src, upper) //
            });
            crate::tests::test_format_hypenated(|src, upper| unsafe {
                format_hyphenated(src, upper) //
            });
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    pub mod simd128 {
        specialize_for!("simd128", simd_abstraction::arch::wasm::SIMD128);

        #[cfg(test)]
        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        #[cfg(test)]
        #[cfg(target_feature = "simd128")]
        #[wasm_bindgen_test::wasm_bindgen_test]
        fn test_parse() {
            crate::tests::test_parse_ok(|s| unsafe { parse(s.as_bytes()) });
            crate::tests::test_parse_err(|s| unsafe { parse(s.as_bytes()) });
        }

        #[cfg(test)]
        #[cfg(target_feature = "simd128")]
        #[wasm_bindgen_test::wasm_bindgen_test]
        fn test_format() {
            crate::tests::test_format_simple(|src, upper| unsafe {
                format_simple(src, upper) //
            });
            crate::tests::test_format_hypenated(|src, upper| unsafe {
                format_hyphenated(src, upper) //
            });
        }
    }
}
