#![allow(dead_code)]

use crate::Error;

/// TODO
#[derive(Debug)]
pub struct CrockfordBase32 {
    case_insensitive: bool,
}

/// TODO
pub const CROCKFORD_BASE32: CrockfordBase32 = CrockfordBase32 {
    case_insensitive: false,
};

impl CrockfordBase32 {
    /// TODO
    #[inline]
    pub const fn case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    /// TODO
    #[inline]
    pub fn encoded_length(&self, n: usize) -> usize {
        assert!(n <= usize::MAX / 2);
        encoded_length_unchecked(n)
    }

    /// TODO
    #[inline]
    pub fn decoded_length(&self, n: usize) -> Result<usize, Error> {
        decoded_length(n)
    }
}

// ---------------------------------------------------------------------

const CHARSET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

const fn decoding_table(case_insensitive: bool) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < 32 {
        let x = CHARSET[i];
        table[x as usize] = x;
        if case_insensitive {
            table[x.to_ascii_lowercase() as usize] = x;
        }
        i += 1;
    }
    table
}

const INSENSITIVE_TABLE: &[u8; 256] = &decoding_table(true);

const SENSITIVE_TABLE: &[u8; 256] = &decoding_table(false);

fn encoded_length_unchecked(len: usize) -> usize {
    const EXTRA: [u8; 5] = [0, 2, 4, 5, 7];
    len / 5 * 8 + EXTRA[len % 5] as usize
}

fn decoded_length(len: usize) -> Result<usize, Error> {
    const EXTRA: [u8; 8] = [0, 0xff, 1, 0xff, 2, 3, 0xff, 4];
    let extra = EXTRA[len % 8];
    ensure!(extra != 0xff);
    Ok(len / 8 * 5 + extra as usize)
}
