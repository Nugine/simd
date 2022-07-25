use std::time::Instant;

fn bench_base64(n: usize, src: &str) -> u128 {
    let encoded = base64_simd::Base64::STANDARD
        .encode_to_boxed_str(src.as_bytes())
        .into_boxed_bytes();

    let mut bufs = vec![encoded; n];

    let t0 = Instant::now();
    for buf in &mut bufs {
        let _ = base64_simd::Base64::forgiving_decode_inplace(buf).unwrap();
    }
    let t1 = Instant::now();
    (t1 - t0).as_nanos()
}

fn bench_ascii(n: usize, data: &[u8]) -> u128 {
    let t0 = Instant::now();
    for _ in 0..n {
        assert!(unicode_simd::is_ascii_ct(data));
    }
    let t1 = Instant::now();
    (t1 - t0).as_nanos()
}

fn main() {
    println!("simd-benches quick mode\n");

    {
        println!("base64-simd forgiving_decode_inplace");

        {
            let src = "helloworld".repeat(100_000);
            let n = 100;
            let time = bench_base64(n, &src);
            let time_per_op = (time / n as u128) as f64 / 1e6;
            println!("long  | n = {n:<8} time = {}ms", time_per_op);
        }

        {
            let src = "123";
            let n = 1_000_000;
            let time = bench_base64(n, src);
            let time_per_op = time / n as u128;
            println!("short | n = {n:<8} time = {}ns", time_per_op);
        }
        println!();
    }

    {
        println!("is_ascii_ct");
        let data = "helloworld".repeat(100_000);
        let n = 10000;
        let time = bench_ascii(n, data.as_bytes());
        let time_per_op = (time / n as u128) as f64 / 1e6;
        let throughput = {
            let total = n as u128 * data.len() as u128;
            let time_sec = time as f64 / 1e9;
            let gib = u128::pow(1024, 3);
            total as f64 / gib as f64 / time_sec
        };
        println!(
            "long  | n = {n:<8} time = {}ms, throughout = {:.6}GiB/s",
            time_per_op, throughput
        );
        println!();
    }
}
