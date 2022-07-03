// Extracted from <https://github.com/denoland/deno/blob/main/cli/bench/deno_common.js>

function bench(name, n, f) {
    const t1 = performance.now();
    for (let i = 0; i < n; ++i) {
        f(i);
    }
    const t2 = performance.now();

    const dt = (t2 - t1) / 1e3;
    const freq = n / dt;
    const time = (t2 - t1) / n;

    const msg = [
        `${name}:     \t`,
        `n = ${n},          \t`,
        `dt = ${dt.toFixed(3)}s, \t`,
        `freq = ${freq.toFixed(3)}/s, \t`,
    ];

    if (time >= 1) {
        msg.push(`time = ${time.toFixed(3)}ms/op`);
    } else {
        msg.push(`time = ${(time * 1e6).toFixed(0)}ns/op`);
    }

    console.log(msg.join(""));
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
