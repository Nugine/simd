use std::time::Instant;

fn time(f: impl FnOnce()) -> u128 {
    let t0 = Instant::now();
    f();
    let t1 = Instant::now();
    (t1 - t0).as_nanos()
}

fn bench_base64(n: usize, src: &str) -> u128 {
    let encoded = base64_simd::STANDARD
        .encode_to_boxed_str(src.as_bytes())
        .into_boxed_bytes();

    let mut bufs = vec![encoded; n];

    time(|| {
        for buf in &mut bufs {
            let _ = base64_simd::Rfc4648Base64::forgiving_decode_inplace(buf).unwrap();
        }
    })
}

fn bench_ascii(n: usize, data: &[u8]) -> u128 {
    time(|| {
        for _ in 0..n {
            assert!(unicode_simd::is_ascii_ct(data));
        }
    })
}

fn bench_utf32(n: usize, data: &[u32]) -> u128 {
    time(|| {
        for _ in 0..n {
            assert!(unicode_simd::is_utf32le_ct(data));
        }
    })
}

fn main() {
    println!("simd-benches quick mode\n");

    {
        println!("base64-simd forgiving_decode_inplace");

        {
            let src = "helloworld".repeat(100_000);
            let n = 100;
            let time = bench_base64(n, &src);
            let time_per_op = (time / n as u128) as f64;
            println!("long  | n = {n:<8} time = {:>8}ns", time_per_op);
        }

        {
            let src = "123";
            let n = 1_000_000;
            let time = bench_base64(n, src);
            let time_per_op = time / n as u128;
            println!("short | n = {n:<8} time = {:>8}ns", time_per_op);
        }
        println!();
    }

    {
        println!("unicode-simd is_ascii_ct");
        let data = "helloworld".repeat(100_000);
        let n = 10000;
        let time = bench_ascii(n, data.as_bytes());
        let time_per_op = time / n as u128;
        let throughput = {
            let total = n as u128 * data.len() as u128;
            let time_sec = time as f64 / 1e9;
            let gib = u128::pow(1024, 3);
            total as f64 / gib as f64 / time_sec
        };
        println!(
            "long  | n = {n:<8} time = {:>8}ns, throughout = {:.6}GiB/s",
            time_per_op, throughput
        );
        println!();
    }

    {
        println!("unicode-simd is_utf32le_ct");
        let data: Vec<u32> = "helloworld".repeat(100_000).chars().map(|ch| ch as u32).collect();
        let n = 1000;
        let time = bench_utf32(n, &data);
        let time_per_op = time / n as u128;
        let throughput = {
            let total = n as u128 * data.len() as u128 * 4;
            let time_sec = time as f64 / 1e9;
            let gib = u128::pow(1024, 3);
            total as f64 / gib as f64 / time_sec
        };
        println!(
            "long  | n = {n:<8} time = {:>8}ns, throughput = {:.6}GiB/s",
            time_per_op, throughput
        );
    }
}
