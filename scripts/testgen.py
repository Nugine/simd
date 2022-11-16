#!/usr/bin/python3
from itertools import product
import argparse

CRATES = [
    "vsimd",
    "hex-simd",
    "base64-simd",
    "uuid-simd",
    "base32-simd",
    "unicode-simd",
]

FEATURES = [
    ["", "unstable"],
    ["", "detect"],
    ["", "alloc", "std"],
]

RUSTFLAGS = {
    "x86": [
        "-Zsanitizer=address -C target-feature=+avx2",
        "-Zsanitizer=address -C target-feature=+sse4.1",
        "-Zsanitizer=address",
        "-C target-feature=+avx2",
        "-C target-feature=+sse4.1",
        "",
    ],
    "arm": [
        "-C target-feature=+neon",
        "",
    ],
    "wasm": [
        "-C target-feature=+simd128",
        "",
    ],
    "miri": [""],
}

TARGETS = {
    "x86": [
        "x86_64-unknown-linux-gnu",
        "i686-unknown-linux-gnu",
    ],
    "arm": [
        "aarch64-unknown-linux-gnu",
        "armv7-unknown-linux-gnueabihf",
    ],
    "wasm": [None],
    "miri": [None],
}

TARGET_REMAP = {
    "x86_64-unknown-linux-gnu": "x86",
    "i686-unknown-linux-gnu": "x86",
    "aarch64-unknown-linux-gnu": "arm",
    "armv7-unknown-linux-gnueabihf": "arm",
    "wasm32-unknown-unknown": "wasm",
}

TEST_MODES = ["x86", "arm", "wasm", "miri"]


def gen(mode, target, rustflag):
    for feat in product(*FEATURES):
        feat = ",".join(s for s in feat if len(s) > 0)
        if len(feat) > 0:
            feat = "--features " + feat

        if mode == "x86" or mode == "arm":
            use_cross = mode == "arm" or target == "i686-unknown-linux-gnu"
            prog = "cross" if use_cross else "cargo"
            lib = "--lib" if mode == "x86" else ""
            print(f'RUSTFLAGS="{rustflag}" {prog} test --target {target} {lib} --no-default-features {feat} $@')
        elif mode == "wasm":
            print(f'RUSTFLAGS="{rustflag}" wasm-pack test --node -- --no-default-features {feat} $@')
        elif mode == "miri":
            print(f'RUSTFLAGS="{rustflag}" cargo miri test --no-default-features {feat} $@')
        else:
            pass


if __name__ == "__main__":
    opt = argparse.ArgumentParser()
    opt.add_argument("--mode", type=str, choices=TEST_MODES)
    opt.add_argument("--crate", type=str, choices=CRATES)
    opt.add_argument("--target", type=str)
    args = opt.parse_args()

    modes = TEST_MODES
    targets = TARGETS
    if args.mode is not None:
        modes = [args.mode]
        if args.target is not None:
            assert args.target in TARGETS[args.mode]
            targets = {args.mode: [args.target]}
    else:
        if args.target is not None:
            modes = [TARGET_REMAP[args.target]]
            targets = {modes[0]: [args.target]}

    crates = CRATES
    if args.crate is not None:
        crates = [args.crate]

    print("#!/bin/bash -ex")

    for mode in modes:
        for target in targets[mode]:
            for rustflag in RUSTFLAGS[mode]:
                if target == "i686-unknown-linux-gnu" and "sanitizer" in rustflag:
                    continue

                for crate in crates:
                    print(f"pushd crates/{crate}")
                    gen(mode, target, rustflag)
                    print("popd")
