use vsimd::alsw::AlswLut;
use vsimd::vector::{V128, V256};

struct Base32Alsw;

impl Base32Alsw {
    #[inline]
    const fn decode(c: u8) -> u8 {
        match c {
            b'A'..=b'Z' => c - b'A',
            b'2'..=b'7' => c - b'2' + 26,
            _ => 0xff,
        }
    }

    #[inline]
    const fn check_hash(i: u8) -> u8 {
        match i {
            0x0 => 1,
            0x1 => 1,
            0x2..=0x7 => 6,
            0x8..=0xA => 1,
            0xB..=0xF => 7,
            _ => unreachable!(),
        }
    }

    #[inline]
    const fn decode_hash(i: u8) -> u8 {
        Self::check_hash(i)
    }
}

vsimd::impl_alsw!(Base32Alsw);

struct Base32HexAlsw;

impl Base32HexAlsw {
    #[inline]
    const fn decode(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'V' => c - b'A' + 10,
            _ => 0xff,
        }
    }

    #[inline]
    const fn check_hash(i: u8) -> u8 {
        match i {
            0 => 1,
            1..=6 => 1,
            7..=9 => 7,
            0xA..=0xF => 2,
            _ => unreachable!(),
        }
    }

    #[inline]
    const fn decode_hash(i: u8) -> u8 {
        Self::check_hash(i)
    }
}

vsimd::impl_alsw!(Base32HexAlsw);

pub const BASE32_ALSW_CHECK_X2: AlswLut<V256> = Base32Alsw::check_lut().x2();
pub const BASE32_ALSW_DECODE_X2: AlswLut<V256> = Base32Alsw::decode_lut().x2();

pub const BASE32HEX_ALSW_CHECK_X2: AlswLut<V256> = Base32HexAlsw::check_lut().x2();
pub const BASE32HEX_ALSW_DECODE_X2: AlswLut<V256> = Base32HexAlsw::decode_lut().x2();

#[cfg(test)]
mod algorithm {
    use super::*;

    #[cfg_attr(
        any(miri, not(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))),
        ignore
    )]
    #[test]
    fn base32_alsw() {
        Base32Alsw::test_check();
        Base32Alsw::test_decode();
    }

    #[cfg_attr(
        any(miri, not(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))),
        ignore
    )]
    #[test]
    fn base32hex_alsw() {
        Base32HexAlsw::test_check();
        Base32HexAlsw::test_decode();
    }
}
