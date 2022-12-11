dev: fmt clippy
    /usr/bin/time -v -o target/time-test-all.txt just test-all
    cat target/time-test-all.txt

clippy: fmt
    cargo clippy --target x86_64-unknown-linux-gnu
    cargo clippy --target armv7-unknown-linux-gnueabihf
    cargo clippy --target aarch64-unknown-linux-gnu
    cargo clippy --target wasm32-unknown-unknown

doc pkg="vsimd":
    #!/bin/bash -e
    export RUSTDOCFLAGS="--cfg docsrs"
    cargo doc --no-deps --all-features
    cargo doc --open -p {{pkg}}

bench dispatch *ARGS:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    DISPATCH={{dispatch}} ./scripts/bench.sh {{ARGS}}

js-bench:
    #!/bin/bash -e
    cd {{justfile_directory()}}

    F=./scripts/base64.js
    echo "running $F"
    echo

    echo "node" `node -v`
    node ./scripts/base64.js
    echo

    deno -V
    deno run --allow-hrtime ./scripts/base64.js
    echo

    echo "bun" `bun --version`
    bun ./scripts/base64.js
    echo

wasi-bench:
    #!/bin/bash -e
    cd {{justfile_directory()}}

    export RUSTFLAGS="-C target-feature=+simd128"
    cargo build -p simd-benches --bin simd-benches --features unstable --profile bench --target wasm32-wasi
    F=./target/wasm32-wasi/release/simd-benches.wasm

    wasmer -V
    wasmer run --enable-all $F
    echo

    wasmtime -V
    wasmtime --wasm-features simd $F
    echo

    echo "node" `node -v`
    node --experimental-wasi-unstable-preview1 ./scripts/node-wasi.js $F
    echo

sync-version:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    cargo set-version -p uuid-simd          '0.8.0-dev'
    cargo set-version -p hex-simd           '0.8.0-dev'
    cargo set-version -p base64-simd        '0.8.0-dev'
    cargo set-version -p unicode-simd       '0.8.0-dev'
    cargo set-version -p base32-simd        '0.8.0-dev'
    cargo set-version -p vsimd              '0.8.0-dev'

fmt:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    cargo fmt
    # cargo sort -w > /dev/null

test crate:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/testgen.py --crate {{crate}} | bash -ex

x86-test crate:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/testgen.py --crate {{crate}} --mode x86 | bash -ex

arm-test crate:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/testgen.py --crate {{crate}} --mode arm | bash -ex

wasm-test crate:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/testgen.py --crate {{crate}} --mode wasm | bash -ex

miri-test crate:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    cargo miri test -p {{crate}}

mips-test crate:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/testgen.py --crate {{crate}} --mode mips | bash -ex

test-all:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/testgen.py | bash -ex
    cargo miri test --workspace --exclude simd-benches

dump-asm:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/dump-symbols.py --mode asm | bash -ex
    COMMIT_HASH=`git rev-parse --short HEAD`
    cd target/symbols
    F=$COMMIT_HASH-asm.txt
    tokei -f -s files -t assembly -c 150 > $F
    tokei -f -s lines -t assembly -c 150
    echo target/symbols/$F

dump-llvm-ir:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/dump-symbols.py --mode llvm-ir | bash -ex
    COMMIT_HASH=`git rev-parse --short HEAD`
    cd target/symbols
    F=$COMMIT_HASH-llvm-ir.txt
    tokei -f -s files -t LLVM -c 150 > $F
    tokei -f -s lines -t LLVM -c 150
    echo target/symbols/$F

bench-quick:
    RUSTFLAGS='-Ctarget-cpu=native' cargo run -p simd-benches --bin simd-benches --profile bench --features unstable
