pub const fn lookup(lut: &[u8; 16], x: u8) -> u8 {
    if x < 0x80 {
        lut[(x & 0x0f) as usize]
    } else {
        0
    }
}

pub const fn avgr(a: u8, b: u8) -> u8 {
    ((a as u16 + b as u16 + 1) >> 1) as u8
}

#[cfg(test)]
pub fn print_fn_table(is_primary: impl Fn(u8) -> bool, f: impl Fn(u8) -> u8) {
    print!("     0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F");
    for c in 0..=255u8 {
        let val = f(c);

        if c & 0x0f == 0 {
            println!();
            print!("{:x} | ", c >> 4);
        }

        if is_primary(c) {
            print!("\x1b[1;31m{:0>2X}\x1b[0m  ", val);
        } else if val >= 0x80 {
            print!("\x1b[1;36m{:0>2X}\x1b[0m  ", val);
        } else {
            print!("\x1b[1;32m{:0>2X}\x1b[0m  ", val);
        }
    }
    println!();
    println!();
}

pub const fn alsw_hash(hash: &[u8; 16], c: u8) -> u8 {
    avgr(0xE0 | (c >> 3), lookup(hash, c))
}

#[cfg(test)]
pub const fn alsw_check(hash: &[u8; 16], offset: &[u8; 16], c: u8) -> u8 {
    let h = alsw_hash(hash, c);
    let o = lookup(offset, h);
    (c as i8).saturating_add(o as i8) as u8
}

#[cfg(test)]
pub const fn alsw_decode(hash: &[u8; 16], offset: &[u8; 16], c: u8) -> u8 {
    let h = alsw_hash(hash, c);
    let o = lookup(offset, h);
    c.wrapping_add(o)
}

macro_rules! alsw_gen_check_offset {
    ($is_primary:ident, $gen_check_hash:ident) => {{
        const ARRAY: [u8; 16] = {
            let hash = &$crate::u8x16!($gen_check_hash);
            let mut arr = [0x80; 16];
            let mut c: u8 = 255;
            loop {
                if $is_primary(c) {
                    let h = $crate::algorithm::alsw_hash(hash, c);
                    arr[(h & 0x0f) as usize] = 0u8.wrapping_sub(c);
                }
                if c == 0 {
                    break;
                }
                c -= 1;
            }
            arr
        };
        ARRAY
    }};
}
