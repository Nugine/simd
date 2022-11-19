use std::time::Instant;

#[inline]
fn time(f: impl FnOnce()) -> u128 {
    let t0 = Instant::now();
    f();
    let t1 = Instant::now();
    (t1 - t0).as_nanos()
}

#[inline(never)]
fn bench_base64(n: usize, src: &str) -> u128 {
    let encoded = base64_simd::STANDARD.encode_type::<Box<[u8]>>(src.as_bytes());

    let mut bufs = vec![encoded; n];

    time(|| {
        for buf in &mut bufs {
            let _ = base64_simd::forgiving_decode_inplace(buf).unwrap();
        }
    })
}

#[inline(never)]
fn bench_ascii(n: usize, data: &[u8]) -> u128 {
    time(|| {
        for _ in 0..n {
            assert!(unicode_simd::is_ascii_ct(data));
        }
    })
}

#[inline(never)]
fn bench_utf32(n: usize, data: &[u32]) -> u128 {
    time(|| {
        for _ in 0..n {
            assert!(unicode_simd::is_utf32le_ct(data));
        }
    })
}

/// GiB/s
fn throughput(bytes: u128, ns: u128) -> f64 {
    let time_sec = ns as f64 / 1e9;
    let gib = u128::pow(1024, 3);
    bytes as f64 / gib as f64 / time_sec
}

fn main() {
    println!("simd-benches quick mode\n");

    {
        println!("base64-simd forgiving_decode_inplace");

        {
            let data = "helloworld".repeat(100_000);
            let n = 100;
            let time = bench_base64(n, &data);
            let time_per_op = (time / n as u128) as f64;
            let throughput = throughput(n as u128 * data.len() as u128, time);
            println!("long  | n = {n:<8} time = {time_per_op:>8} ns, throughout = {throughput:.6} GiB/s");
        }

        {
            let data = "123";
            let n = 1_000_000;
            let time = bench_base64(n, data);
            let time_per_op = time / n as u128;
            let throughput = throughput(n as u128 * data.len() as u128, time);
            println!("short | n = {n:<8} time = {time_per_op:>8} ns, throughout = {throughput:.6} GiB/s");
        }
        println!();
    }

    {
        println!("unicode-simd is_ascii_ct");
        let data = "helloworld".repeat(100_000);
        let n = 10000;
        let time = bench_ascii(n, data.as_bytes());
        let time_per_op = time / n as u128;
        let throughput = throughput(n as u128 * data.len() as u128, time);
        println!(
            "long  | n = {n:<8} time = {:>8} ns, throughout = {:.6} GiB/s",
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
        let throughput = throughput(n as u128 * data.len() as u128 * 4, time);
        println!(
            "long  | n = {n:<8} time = {:>8} ns, throughput = {:.6} GiB/s",
            time_per_op, throughput
        );

        println!();
    }

    #[cfg(feature = "unstable")]
    {
        println!("vsimd::unstable::is_ascii");
        let data = "helloworld".repeat(100_000);
        let n = 1000;
        let time = time(|| {
            for _ in 0..n {
                assert!(vsimd::unstable::is_ascii(data.as_bytes()));
            }
        });
        let time_per_op = time / n as u128;
        let throughput = throughput(n as u128 * data.len() as u128, time);
        println!(
            "long  | n = {n:<8} time = {:>8} ns, throughout = {:.6} GiB/s",
            time_per_op, throughput
        );
        println!();
    }
}
