const now = (() => {
    if ("Deno" in globalThis) {
        return () => (performance.now() * 1e6);
    }
    if ("Bun" in globalThis) {
        return Bun.nanoseconds;
    }
    return () => Number(process.hrtime.bigint());
})();

function bench(name, iter, input, f) {
    const t1 = now();
    for (let i = 0; i < iter; ++i) {
        f(input);
    }
    const t2 = now();

    const dt = (t2 - t1) / 1e9;
    const freq = iter / dt;
    const time = (t2 - t1) / iter;

    const msg = [
        `${name.padEnd(16)}|`,
        `len = ${input.length.toString().padStart(8)}  |`,
        `iter = ${iter.toString().padStart(7)}  |`,
        `dt = ${dt.toFixed(3).toString().padStart(5)}s  |`,
        `freq = ${freq.toFixed(3).toString().padStart(12)}/s  |`,
        `time = ${time.toFixed(0).toString().padStart(10)}ns/op`
    ];

    console.log(msg.join("  "));
}

const TEST_CASES = [
    {
        data: "helloworld".repeat(1e5),
        iter: 100,
    },
    {
        data: "helloworld".repeat(1e4),
        iter: 1000,
    },
    {
        data: "helloworld".repeat(1e2),
        iter: 1e5,
    },
    {
        data: "helloworld".repeat(10),
        iter: 1e6,
    },
    {
        data: "abcdefghijklmnopqrstuvwx",
        iter: 1e5,
    },
    {
        data: "123",
        iter: 1e6
    }
]

const FUNCTIONS = [
    {
        name: "encode+decode",
        call(input) { return atob(btoa(input)); }
    },
    {
        name: "encode",
        call(input) { return btoa(input); }
    },
    {
        name: "decode",
        call(input) { return atob(input); }
    }
]

for (const t of TEST_CASES) {
    for (const f of FUNCTIONS) {
        bench(f.name, t.iter, t.data, f.call);
    }
    console.log("    ")
}
