use std::ops::Not;

use rand::Rng;

fn rand_bytes(n: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut bytes = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

#[cfg(miri)]
use std::io::Write as _;

macro_rules! dbgmsg {
    ($($fmt:tt)*) => {
        println!($($fmt)*);
        #[cfg(miri)]
        std::io::stdout().flush().unwrap();
    };
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn random() {
    for n in 0..256 {
        dbgmsg!("n = {n}");

        let mut src = rand_bytes(n);
        src.iter_mut().for_each(|x| *x >>= 1);

        assert!(unicode_simd::is_ascii(&src));

        if n > 0 {
            let pos = rand::thread_rng().gen_range(0..n);
            src[pos] = 0x80;
            assert!(unicode_simd::is_ascii(&src).not());
        }
    }
}
