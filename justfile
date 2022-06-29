doc:
    cargo doc --open --no-deps --all-features

x86-bench *ARGS:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    export RUSTFLAGS="-C target-feature=+avx2 -C target-feature=+sse4.1"
    cargo criterion {{ARGS}}

js-bench:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    node -v
    node ./scripts/base64.js
    deno -V
    deno run ./scripts/base64.js

x86-test *ARGS:
    #!/bin/bash -ex
    cd {{invocation_directory()}}
    export RUSTFLAGS="-C target-feature=+avx2 -C target-feature=+sse4.1 -Zsanitizer=address"
    cargo test --lib {{ARGS}}
    export RUSTFLAGS=""
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

miri *ARGS:
    #!/bin/bash
    cd {{invocation_directory()}}
    cargo miri test -- --nocapture --test-threads=1 {{ARGS}}

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
    vers='0.6.1'
    for pkg in `ls crates`
    do
        echo $pkg $vers
        pushd crates/$pkg > /dev/null
        cargo set-version $vers
        popd > /dev/null
    done

fmt:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    cargo fmt
    # cargo sort -w > /dev/null
