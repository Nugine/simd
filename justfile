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
    mkdir -p target/simd-benches
    COMMIT_HASH=`git rev-parse --short HEAD`

    case "{{dispatch}}" in
        static)
            export RUSTFLAGS="-C target-cpu=native"
            FEATURES=""
            ;;
        dynamic)
            export RUSTFLAGS=""
            FEATURES="detect"
            ;;
        fallback)
            export RUSTFLAGS=""
            FEATURES=""
            ;;
        *)
            echo "Unknown dispatch: {{dispatch}}"
            exit 1
            ;;
    esac

    NAME=target/simd-benches/$COMMIT_HASH-{{dispatch}}

    time cargo criterion -p simd-benches --history-id $COMMIT_HASH --message-format json --features "$FEATURES" {{ARGS}} > $NAME.jsonl
    just analyze $COMMIT_HASH {{dispatch}} > $NAME.md
    bat --paging=never $NAME.md

bench-all:
    just bench static
    just bench dynamic
    just bench fallback

analyze commit dispatch:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    cargo run -q -p simd-analyze -- target/simd-benches/{{commit}}-{{dispatch}}.jsonl

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
    cargo build --profile bench --bin simd-benches -p simd-benches --target wasm32-wasi
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

test-all:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    ./scripts/testgen.py | bash -ex
    cargo miri test --workspace --exclude simd-benches --exclude simd-analyze
