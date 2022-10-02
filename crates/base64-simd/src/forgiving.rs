use crate::ascii::remove_ascii_whitespace_inplace;

const fn discard_table(mask: u8) -> [u8; 256] {
    let charset = vsimd::base64::STANDARD_CHARSET;
    let mut table = [0; 256];

    let mut i = 0;
    loop {
        table[i as usize] = i;
        if i == 255 {
            break;
        }
        i += 1;
    }

    let mut i = 0;
    while i < 64 {
        table[charset[i] as usize] = charset[i & mask as usize];
        i += 1;
    }
    table
}

#[inline(always)]
fn discard4(ch: &mut u8) {
    const TABLE: &[u8; 256] = &discard_table(0xf0);
    unsafe { *ch = *TABLE.get_unchecked(*ch as usize) }
}

#[inline(always)]
fn discard2(ch: &mut u8) {
    const TABLE: &[u8; 256] = &discard_table(0xfc);
    unsafe { *ch = *TABLE.get_unchecked(*ch as usize) }
}

pub fn normalize(buf: &mut [u8]) -> &mut [u8] {
    let buf = remove_ascii_whitespace_inplace(buf);

    if buf.is_empty() {
        return buf;
    }

    unsafe {
        let len = buf.len();
        match len % 4 {
            0 => {
                let x1 = *buf.get_unchecked(len - 1);
                let x2 = *buf.get_unchecked(len - 2);
                if x1 == b'=' {
                    if x2 == b'=' {
                        let last3 = buf.get_unchecked_mut(len - 3);
                        discard4(last3);
                        buf.get_unchecked_mut(..len - 2)
                    } else {
                        let last2 = buf.get_unchecked_mut(len - 2);
                        discard2(last2);
                        buf.get_unchecked_mut(..len - 1)
                    }
                } else {
                    buf
                }
            }
            1 => buf,
            2 => {
                let last1 = buf.get_unchecked_mut(len - 1);
                discard4(last1);
                buf
            }
            3 => {
                let last1 = buf.get_unchecked_mut(len - 1);
                discard2(last1);
                buf
            }
            _ => core::hint::unreachable_unchecked(),
        }
    }
}
