use vsimd::ascii::lookup_ascii_whitespace;

#[inline(always)]
#[must_use]
pub fn find_non_ascii_whitespace(data: &[u8]) -> usize {
    vsimd::ascii::multiversion::find_non_ascii_whitespace::auto(data)
}

#[inline(always)]
#[must_use]
pub unsafe fn remove_ascii_whitespace_fallback(mut src: *const u8, len: usize, mut dst: *mut u8) -> usize {
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
        assert_eq!(ans, &*expected, "case = {case:?}");
    }
}
