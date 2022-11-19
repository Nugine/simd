#!/usr/bin/python3
import argparse

SYMBOLS = {
    "hex-simd": {
        "check": ["avx2", "sse2", "neon", "simd128"],
        "encode": ["avx2", "ssse3", "sse2", "neon", "simd128"],
        "decode": ["avx2", "ssse3", "sse2", "neon", "simd128"],
    },
    "base64-simd": {
        "encode": ["avx2", "ssse3", "neon", "simd128"],
        "decode": ["avx2", "ssse3", "neon", "simd128"],
        "check": ["avx2", "ssse3", "neon", "simd128"],
        "find_non_ascii_whitespace": ["avx2", "sse2", "neon", "simd128"],
    },
    "unicode-simd": {
        "is_ascii_ct": ["avx2", "sse2", "neon", "simd128"],
        "is_utf32le_ct": ["avx2", "sse4.1", "neon", "simd128"],
        "utf32_swap_endianness": ["avx2", "ssse3", "neon", "simd128"],
        "utf16_swap_endianness": ["avx2", "ssse3", "neon", "simd128"],
    },
    # ...
}

TARGETS = {
    "avx2": ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"],
    "sse4.1": ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"],
    "ssse3": ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"],
    "sse2": ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"],
    "neon": ["aarch64-unknown-linux-gnu", "armv7-unknown-linux-gnueabihf"],
    # "simd128": ["wasm32-unknown-unknown"],
    "simd128": [],  # TODO: https://github.com/pacak/cargo-show-asm/issues/91
}

if __name__ == "__main__":
    opt = argparse.ArgumentParser()
    opt.add_argument("--mode", type=str, choices=["asm", "llvm-ir"], required=True)
    args = opt.parse_args()

    print("#!/bin/bash -ex")

    for pkg, symbols in SYMBOLS.items():
        for name, features in symbols.items():
            for feature in features:
                for target in TARGETS[feature]:
                    print(f"mkdir -p target/symbols/{target}")
                    symbol = f"{pkg.replace('-', '_')}::multiversion::{name}::{feature.replace('.', '')}"
                    if args.mode == "asm":
                        print(
                            f"cargo asm -p {pkg} --simplify --features unstable --target {target} -- {symbol} "
                            f"| awk NF"
                            f"> target/symbols/{target}/{symbol}.asm"
                        )
                    elif args.mode == "llvm-ir":
                        print(
                            f"RUSTFLAGS=-Cdebuginfo=0 "
                            f"cargo asm -p {pkg} --llvm --features unstable --target {target} -- {symbol} "
                            f"> target/symbols/{target}/{symbol}.ll"
                        )
