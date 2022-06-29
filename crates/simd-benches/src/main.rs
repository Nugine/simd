fn bench_base64(n: usize, src: &str) -> u128 {
    let encoded = base64_simd::Base64::STANDARD
        .encode_to_boxed_str(src.as_bytes())
        .into_boxed_bytes();

    let mut time = 0;
    for _ in 0..n {
        let mut encoded = encoded.clone();
        let t0 = std::time::Instant::now();
        let dst = base64_simd::Base64::forgiving_decode_inplace(&mut encoded).unwrap();
        let t1 = std::time::Instant::now();
        assert_eq!(src.as_bytes(), dst);
        time += (t1 - t0).as_nanos();
    }
    time
}

fn main() {
    println!("simd-benches quick mode");

    {
        println!("base64-simd forgiving_decode_inplace");

        {
            let src = "helloworld".repeat(100_000);
            let n = 100;
            let time = bench_base64(n, &src);
            println!("n = {n:<8} long  = {}ms", (time / n as u128) as f64 / 1e6);
        }

        {
            let src = "123";
            let n = 1_000_000;
            let time = bench_base64(n, src);
            println!("n = {n:<8} short = {}ns", (time / n as u128));
        }
    }
}
