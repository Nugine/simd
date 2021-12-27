doc:
    cargo doc --open --no-deps --all-features

x86-bench *ARGS:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    export RUSTFLAGS="-C target-feature=+avx2 -C target-feature=+sse4.1"
    cargo criterion {{ARGS}}

x86-test *ARGS:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    export RUSTFLAGS="-C target-feature=+avx2 -C target-feature=+sse4.1"
    cargo test {{ARGS}}

arm-test *ARGS:
    #!/bin/bash -ex
    export RUSTFLAGS="-C target-feature=+neon"
    cross test --target armv7-unknown-linux-gnueabihf --features unstable {{ARGS}}
    cross test --target aarch64-unknown-linux-gnu --features unstable {{ARGS}}

wasm-test:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    export RUSTFLAGS="-C target-feature=+simd128"
    wasm-pack test --firefox --headless
    wasm-pack test --chrome --headless

miri:
    #!/bin/bash
    cargo miri test -- --nocapture --test-threads=1

test-all:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    members=`python3 -c 'list(map(print,__import__("toml").load(open("Cargo.toml"))["workspace"]["members"]))'`
    for member in $members
    do
        cd $member
        just wasm-test
        cd ..
    done
    just arm-test
    just x86-test
