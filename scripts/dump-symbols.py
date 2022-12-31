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
        "is_ascii": ["avx2", "sse2", "neon", "simd128"],
        "is_utf32le": ["avx2", "sse4.1", "neon", "simd128"],
        "utf32_swap_endianness": ["avx2", "ssse3", "neon", "simd128"],
        "utf16_swap_endianness": ["avx2", "ssse3", "neon", "simd128"],
    },
    "uuid-simd": {
        "parse_simple": ["avx2", "ssse3", "sse2", "neon", "simd128"],
        "parse_hyphenated": ["avx2", "sse4.1", "neon", "simd128"],
        "format_simple": ["avx2", "ssse3", "sse2", "neon", "simd128"],
        "format_hyphenated": ["avx2", "sse4.1", "neon", "simd128"],
    },
    "base32-simd": {
        "check": ["avx2", "ssse3", "neon", "simd128"],
        "encode": ["avx2", "sse4.1", "neon", "simd128"],
        "decode": ["avx2", "sse4.1", "neon", "simd128"],
    },
}

TARGETS = {
    "avx2": ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"],
    "sse4.1": ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"],
    "ssse3": ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"],
    "sse2": ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"],
    "neon": ["aarch64-unknown-linux-gnu", "armv7-unknown-linux-gnueabihf"],
    "simd128": ["wasm32-unknown-unknown"],
}


def space_join(l):
    return " ".join(l)


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

                    rustflags = ["--cfg vsimd_dump_symbols"]
                    if target == "wasm32-unknown-unknown":
                        rustflags.append("-C target-feature=+simd128")

                    if args.mode == "asm":
                        extra_flags = "--wasm" if target == "wasm32-unknown-unknown" else ""
                        print(
                            f'RUSTFLAGS="{space_join(rustflags)}" '
                            f"cargo asm -p {pkg} --lib --simplify --features unstable --target {target} {extra_flags} -- {symbol} "
                            f"| awk NF"
                            f"> target/symbols/{target}/{symbol}.asm"
                        )
                    elif args.mode == "llvm-ir":
                        rustflags.append("-Cdebuginfo=0")
                        print(
                            f'RUSTFLAGS="{space_join(rustflags)}" '
                            f"cargo asm -p {pkg} --lib --llvm --features unstable --target {target} -- {symbol} "
                            f"> target/symbols/{target}/{symbol}.ll"
                        )
