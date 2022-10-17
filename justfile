dev:
    just fmt
    just test-all

doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo doc --open --no-deps --all-features

x86-bench *ARGS:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    export RUSTFLAGS="-C target-feature=+avx2 -C target-feature=+sse4.1"
    time cargo criterion {{ARGS}}

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
    deno run ./scripts/base64.js
    echo

    echo "bun" `bun --version`
    bun ./scripts/base64.js
    echo

wasi-bench:
    #!/bin/bash -e
    cd {{justfile_directory()}}

    export RUSTFLAGS="-C target-feature=+simd128"
    cargo build --release --bins -p simd-benches --target wasm32-wasi
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

x86-test *ARGS:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    function x86test(){
        cargo test --no-default-features --features 'std,unstable' $@ {{ARGS}}
        cargo test --features 'unstable' $@ {{ARGS}}
    }
    export RUSTFLAGS="-Zsanitizer=address -C target-feature=+avx2"
    x86test --lib
    export RUSTFLAGS="-Zsanitizer=address -C target-feature=+sse4.1"
    x86test --lib
    export RUSTFLAGS="-Zsanitizer=address"
    x86test --lib
    export RUSTFLAGS="-C target-feature=+avx2"
    x86test
    export RUSTFLAGS="-C target-feature=+sse4.1"
    x86test
    export RUSTFLAGS=""
    x86test

arm-test *ARGS:
    #!/bin/bash -ex
    function armtest(){
        cross test --target armv7-unknown-linux-gnueabihf \
            --no-default-features --features 'std,unstable' {{ARGS}}
        cross test --target aarch64-unknown-linux-gnu \
            --no-default-features --features 'std,unstable' {{ARGS}}
        cross test --target armv7-unknown-linux-gnueabihf \
            --features 'unstable' {{ARGS}}
        cross test --target aarch64-unknown-linux-gnu \
            --features 'unstable' {{ARGS}}
    }

    export RUSTFLAGS="-C target-feature=+neon"
    armtest
    export RUSTFLAGS=""
    armtest

wasm-test:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    export RUSTFLAGS="-C target-feature=+simd128"
    wasm-pack test --firefox --headless
    wasm-pack test --chrome --headless 

miri *ARGS:
    #!/bin/bash
    cd {{invocation_directory()}}
    time cargo miri test -- --nocapture --test-threads=1 {{ARGS}}

test-all:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    members=("simd-abstraction" "uuid-simd" "hex-simd" "base64-simd")
    just arm-test
    for member in "${members[@]}"
    do
        pushd crates/$member
        just x86-test
        just wasm-test
        popd
    done

sync-version:
    #!/bin/bash -e
    cd {{justfile_directory()}}
    cargo set-version -p simd-abstraction '0.7.1'
    cargo set-version -p uuid-simd '0.7.1'
    cargo set-version -p hex-simd '0.7.1'
    cargo set-version -p base64-simd '0.7.1'
    cargo set-version -p simd-benches '0.7.1'

fmt:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    cargo fmt
    # cargo sort -w > /dev/null
