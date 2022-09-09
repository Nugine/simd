pub fn lookup(lut: &[u8; 16], x: u8) -> u8 {
    if x < 0x80 {
        lut[(x & 0x0f) as usize]
    } else {
        0
    }
}

pub fn avgr(a: u8, b: u8) -> u8 {
    ((a as u16 + b as u16 + 1) >> 1) as u8
}

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
