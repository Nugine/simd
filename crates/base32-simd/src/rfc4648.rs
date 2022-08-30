#![allow(dead_code)]

use crate::Error;

/// TODO
#[derive(Debug)]
pub struct Rfc4648Base32 {
    kind: Kind,
    padding: bool,
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Base32,
    Base32Hex,
}

/// TODO
pub const BASE32: Rfc4648Base32 = Rfc4648Base32 {
    kind: Kind::Base32,
    padding: true,
};

/// TODO
pub const BASE32HEX: Rfc4648Base32 = Rfc4648Base32 {
    kind: Kind::Base32Hex,
    padding: true,
};

impl Rfc4648Base32 {
    /// TODO
    #[inline(always)]
    pub const fn padding(mut self, padding: bool) -> Self {
        self.padding = padding;
        self
    }
}

impl Rfc4648Base32 {
    /// TODO
    #[inline]
    pub fn encoded_length(&self, n: usize) -> usize {
        assert!(n <= usize::MAX / 2);
        encoded_length_unchecked(n, self.padding)
    }

    /// TODO
    #[inline]
    pub fn decoded_length(&self, data: &[u8]) -> Result<usize, Error> {
        let (_, m) = decoded_length(data, self.padding)?;
        Ok(m)
    }

    /// TODO
    #[inline]
    pub const fn estimated_decoded_length(&self, n: usize) -> usize {
        if n % 8 == 0 {
            n / 8 * 5
        } else {
            (n / 8 + 1) * 5
        }
    }
}

// ------------------------------------------------------------------------------------------------

const BASE32_CHARSET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

const BASE32HEX_CHARSET: &[u8; 32] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

const fn decoding_table(charset: &[u8; 32]) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < 32 {
        let x = charset[i];
        table[x as usize] = x;
        i += 1;
    }
    table
}

const BASE32_TABLE: &[u8; 256] = &decoding_table(BASE32_CHARSET);

const BASE32HEX_TABLE: &[u8; 256] = &decoding_table(BASE32HEX_CHARSET);

impl Rfc4648Base32 {
    fn copy(&self) -> Self {
        Self {
            kind: self.kind,
            padding: self.padding,
        }
    }

    #[inline(always)]
    fn charset(&self) -> &'static [u8; 32] {
        match self.kind {
            Kind::Base32 => BASE32_CHARSET,
            Kind::Base32Hex => BASE32HEX_CHARSET,
        }
    }

    #[inline(always)]
    fn decoding_table(&self) -> &'static [u8; 256] {
        match self.kind {
            Kind::Base32 => BASE32_TABLE,
            Kind::Base32Hex => BASE32HEX_TABLE,
        }
    }
}

fn encoded_length_unchecked(len: usize, padding: bool) -> usize {
    let l = len / 5 * 8;
    if len % 5 == 0 {
        return l;
    }
    if padding {
        return l + 8;
    }
    const EXTRA: [u8; 5] = [0, 2, 4, 5, 7];
    l + EXTRA[len % 5] as usize
}

fn decoded_length(data: &[u8], padding: bool) -> Result<(usize, usize), Error> {
    if data.is_empty() {
        return Ok((0, 0));
    }

    let len = data.len();
    let n = if padding {
        ensure!(len % 8 == 0);
        let last = unsafe { data.get_unchecked(len - 6..) };
        let count = last.iter().copied().filter(|&x| x == b'=').count();
        len - count
    } else {
        data.len()
    };

    const EXTRA: [u8; 8] = [0, 0xff, 1, 0xff, 2, 3, 0xff, 4];
    let extra = EXTRA[n % 8];
    ensure!(extra != 0xff);
    let m = n / 8 * 5 + extra as usize;
    Ok((n, m))
}
