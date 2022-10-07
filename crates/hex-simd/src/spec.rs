#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
pub mod x86 {
    use crate::fallback;
    use crate::Error;

    use vsimd::ascii::AsciiCase;
    use vsimd::isa::InstructionSet;
    use vsimd::isa::SSE2;
    use vsimd::tools::slice;
    use vsimd::tools::slice_parts;
    use vsimd::SIMD128;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn sse2_encode(src: &[u8], mut dst: *mut u8, case: AsciiCase) {
        let (mut src, mut len) = slice_parts(src);

        let offset = match case {
            AsciiCase::Lower => _mm_set1_epi8(0x27),
            AsciiCase::Upper => _mm_set1_epi8(0x07),
        };

        let encode_bytes16 = |x: __m128i, o: __m128i| {
            let t = _mm_add_epi8(x, _mm_set1_epi8(0x30));
            let cmp = _mm_cmplt_epi8(_mm_set1_epi8(0x39), t);
            let offset = _mm_and_si128(cmp, o);
            _mm_add_epi8(t, offset)
        };

        let end = src.add(len / 16 * 16);
        while src < end {
            let x = _mm_loadu_si128(src.cast());
            src = src.add(16);

            let m = _mm_set1_epi8(0x0f);
            let hi = _mm_and_si128(_mm_srli_epi16::<4>(x), m);
            let lo = _mm_and_si128(x, m);

            let hi = encode_bytes16(hi, offset);
            let lo = encode_bytes16(lo, offset);

            let y1 = _mm_unpacklo_epi8(hi, lo);
            let y2 = _mm_unpackhi_epi8(hi, lo);

            _mm_storeu_si128(dst.cast(), y1);
            dst = dst.add(16);

            _mm_storeu_si128(dst.cast(), y2);
            dst = dst.add(16);
        }
        len %= 16;

        if len > 0 {
            let charset = fallback::charset(case).as_ptr();
            fallback::encode_short(src, len, dst, charset);
        }
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn sse2_decode(mut src: *const u8, mut len: usize, mut dst: *mut u8) -> Result<(), Error> {
        let end = src.add(len / 16 * 16);
        while src < end {
            let x = _mm_loadu_si128(src.cast());

            let (nibbles, flag) = decode_nibbles(x);
            ensure!(_mm_movemask_epi8(flag) == 0);

            let ans = merge_bits(nibbles);
            dst.cast::<u64>().write_unaligned(ans);

            src = src.add(16);
            dst = dst.add(8);
        }
        len %= 16;

        if len > 0 {
            fallback::decode_short(src, len, dst)?;
        }

        Ok(())
    }

    #[inline(always)]
    unsafe fn _mm_set1_epu8(x: u8) -> __m128i {
        _mm_set1_epi8(x as i8)
    }

    #[inline(always)]
    unsafe fn decode_nibbles(x: __m128i) -> (__m128i, __m128i) {
        // http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html
        // Algorithm 3

        let t1 = _mm_add_epi8(x, _mm_set1_epu8(0xff - b'9'));
        let t2 = _mm_subs_epu8(t1, _mm_set1_epi8(6));
        let t3 = _mm_sub_epi8(t2, _mm_set1_epu8(0xf0));
        let t4 = _mm_and_si128(x, _mm_set1_epu8(0xdf));
        let t5 = _mm_sub_epi8(t4, _mm_set1_epi8(0x41));
        let t6 = _mm_adds_epu8(t5, _mm_set1_epi8(10));
        let t7 = _mm_min_epu8(t3, t6);
        let t8 = _mm_adds_epu8(t7, _mm_set1_epi8(127 - 15));
        (t7, t8)
    }

    #[inline(always)]
    unsafe fn merge_bits(x: __m128i) -> u64 {
        let lo = _mm_srli_epi16::<8>(x);
        let hi = _mm_slli_epi16::<4>(x);
        let t1 = _mm_or_si128(lo, hi);
        let t2 = _mm_and_si128(t1, _mm_set1_epi16(0xff));
        let t3 = _mm_packus_epi16(t2, _mm_setzero_si128());
        let [ans, _]: [u64; 2] = core::mem::transmute(t3);
        ans
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn sse2_check(data: &[u8]) -> Result<(), Error> {
        let s = SSE2::new();
        let (mut src, mut len) = slice_parts(data);

        let end = src.add(len / 16 * 16);
        while src < end {
            let x = s.v128_load_unaligned(src);
            ensure!(vsimd::hex::check_xn(s, x));
            src = src.add(16);
        }
        len %= 16;

        if len == 0 {
            return Ok(());
        }

        fallback::check_short(slice(src, len))
    }
}
