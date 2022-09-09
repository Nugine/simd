const now = (() => {
    if ("Deno" in globalThis) {
        return () => (performance.now() * 1e6);
    }
    if ("Bun" in globalThis) {
        return Bun.nanoseconds;
    }
    return () => Number(process.hrtime.bigint());
})();

function bench(name, n, f) {
    const t1 = now();
    for (let i = 0; i < n; ++i) {
        f(i);
    }
    const t2 = now();

    const dt = (t2 - t1) / 1e9;
    const freq = n / dt;
    const time = (t2 - t1) / n;

    const msg = [
        `${name.padEnd(12)}|`,
        `n = ${n.toString().padStart(7)},`,
        `dt = ${dt.toFixed(3).toString().padStart(5)}s,`,
        `freq = ${freq.toFixed(3).toString().padStart(12)}/s,`,
        `time = ${time.toFixed(0).toString().padStart(10)}ns/op`
    ];

    console.log(msg.join("    "));
}

const LONG = "helloworld".repeat(1e5);
const SHORT = "123";

function b64Long() {
    const input = LONG;
    bench("b64Long", 100, () => {
        atob(btoa(input));
    });
}

function b64LongE() {
    const input = LONG;
    bench("b64LongE", 100, () => {
        btoa(input);
    });
}

function b64LongD() {
    const input = btoa(LONG);
    bench("b64LongD", 100, () => {
        atob(input);
    });
}

function b64Short() {
    const input = SHORT;
    bench("b64Short", 1e6, () => {
        atob(btoa(input));
    });
}

function b64ShortE() {
    const input = SHORT;
    bench("b64ShortE", 1e6, () => {
        btoa(input);
    });
}

function b64ShortD() {
    const input = btoa(SHORT);
    bench("b64ShortD", 1e6, () => {
        atob(input);
    });
}

b64Long();
b64LongE();
b64LongD();
b64Short();
b64ShortE();
b64ShortD();
