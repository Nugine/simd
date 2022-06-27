fn main() {
    let src = "helloworld".repeat(100_000).into_bytes();
    let encoded = base64_simd::Base64::STANDARD
        .encode_to_boxed_str(&*src)
        .into_boxed_bytes();

    let n = 100;
    let mut time = 0;
    for _ in 0..n {
        let mut encoded = encoded.clone();
        let t0 = std::time::Instant::now();
        let dst = base64_simd::Base64::forgiving_decode_inplace(&mut encoded).unwrap();
        let t1 = std::time::Instant::now();
        assert_eq!(src, dst);
        time += (t1 - t0).as_nanos();
    }
    println!("time = {}ms", (time / n) as f64 / 1e6);
}
