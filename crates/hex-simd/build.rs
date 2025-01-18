fn main() {
    println!("cargo:rustc-check-cfg=cfg(vsimd_dump_symbols)")
}
