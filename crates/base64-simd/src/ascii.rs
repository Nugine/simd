use vsimd::ascii::lookup_ascii_whitespace;

/// TODO
#[inline(always)]
#[must_use]
pub fn find_non_ascii_whitespace(data: &[u8]) -> usize {
    vsimd::ascii::multiversion::find_non_ascii_whitespace::auto(data)
}

#[inline(always)]
#[must_use]
unsafe fn remove_ascii_whitespace_fallback(mut src: *const u8, len: usize, mut dst: *mut u8) -> usize {
    let dst_base = dst;

    let end = src.add(len);
    while src < end {
        let x = src.read();
        if lookup_ascii_whitespace(x) == 0 {
            dst.write(x);
            dst = dst.add(1);
        }
        src = src.add(1);
    }

    dst.offset_from(dst_base) as usize
}

/// TODO
#[inline(always)]
#[must_use]
pub fn remove_ascii_whitespace_inplace(data: &mut [u8]) -> &mut [u8] {
    let pos = find_non_ascii_whitespace(data);
    debug_assert!(pos <= data.len());

    if pos == data.len() {
        return data;
    }

    unsafe {
        let len = data.len() - pos;
        let dst = data.as_mut_ptr().add(pos);
        let src = dst;

        let rem = remove_ascii_whitespace_fallback(src, len, dst);
        debug_assert!(rem <= len);

        data.get_unchecked_mut(..(pos + rem))
    }
}

// /// TODO
// #[inline(always)]
// #[must_use]
// pub fn remove_ascii_whitespace_copied<'s, 'd>(src: &'s [u8], mut dst: OutRef<'d, [u8]>) -> &'d mut [u8] {
//     assert!(src.len() <= dst.len());

//     let pos = find_non_ascii_whitespace(src);
//     debug_assert!(pos <= src.len());

//     unsafe {
//         let len = src.len();
//         let src = src.as_ptr();
//         let dst = dst.as_mut_ptr();

//         core::ptr::copy_nonoverlapping(src, dst, pos);

//         if pos == len {
//             return slice_mut(dst, pos);
//         }

//         let rem = remove_ascii_whitespace_fallback(src.add(pos), len - pos, dst.add(pos));
//         debug_assert!(rem <= len - pos);

//         slice_mut(dst, pos + rem)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_ascii_whitespace() {
        let cases = [
            "abcd",
            "ab\tcd",
            "ab\ncd",
            "ab\x0Ccd",
            "ab\rcd",
            "ab cd",
            "ab\t\n\x0C\r cd",
            "ab\t\n\x0C\r =\t\n\x0C\r =\t\n\x0C\r ",
        ];
        for case in cases {
            let mut buf = case.to_owned().into_bytes();
            let expected = {
                let mut v = buf.clone();
                v.retain(|c| !c.is_ascii_whitespace());
                v
            };
            let ans = remove_ascii_whitespace_inplace(&mut buf);
            assert_eq!(ans, &*expected, "case = {:?}", case);
        }
    }
}
