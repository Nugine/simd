use crate::{Error, Kind};

vsimd::dispatch_v2!(
    name        = {check},
    signature   = {pub(crate) unsafe fn(src: *const u8, len: usize, kind: Kind) -> Result<(), Error>},
    fallback    = {crate::check::check_fallback},
    simd        = {crate::check::check_simd},
    targets     = {"avx2", "ssse3", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);

vsimd::dispatch_v2!(
    name        = {decode},
    signature   = {pub(crate) unsafe fn(src: *const u8, len: usize, dst: *mut u8, kind: Kind) -> Result<(), Error>},
    fallback    = {crate::decode::decode_fallback},
    simd        = {crate::decode::decode_simd},
    targets     = {"avx2", "sse4.1", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);

vsimd::dispatch_v2!(
    name        = {encode},
    signature   = {pub(crate) unsafe fn(src: *const u8, len: usize, dst: *mut u8, kind: Kind, padding: bool) -> ()},
    fallback    = {crate::encode::encode_fallback},
    simd        = {crate::encode::encode_simd},
    targets     = {"avx2", "sse4.1", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);
