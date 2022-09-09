dev:
    just fmt
    time just test-all

doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo doc --open --no-deps --all-features

x86-bench *ARGS:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    cd crates/simd-benches

    COMMIT_HASH=`git rev-parse --short HEAD`
    
    export RUSTFLAGS="-C target-feature=+avx2 -C target-feature=+sse4.1"
    cargo build --benches --release
    
    declare -a benches
    benches[0]="uuid"
    benches[1]="hex"
    benches[2]="base64"
    
    for bench in "${benches[@]}"
    do
        cargo bench --bench "$bench" -- --save-baseline "$bench-$COMMIT_HASH" {{ARGS}}
    done

critcmp:
    #!/bin/bash -ex
    COMMIT_HASH=`git rev-parse --short HEAD`
    mkdir -p target/critcmp
    critcmp -g '.+?/.+?/.+?/(.+)'  --color always "uuid-$COMMIT_HASH"   -f uuid-simd-parse      > "target/critcmp/uuid-$COMMIT_HASH-parse.ansi"  
    critcmp -g '.+?/.+?/.+?/(.+)'  --color always "uuid-$COMMIT_HASH"   -f uuid-simd-format     > "target/critcmp/uuid-$COMMIT_HASH-format.ansi"  
    critcmp -g '.+?/.+?/.+?/(.+)'  --color always "hex-$COMMIT_HASH"    -f hex-simd-check       > "target/critcmp/hex-$COMMIT_HASH-check.ansi"
    critcmp -g '.+?/.+?/.+?/(.+)'  --color always "hex-$COMMIT_HASH"    -f hex-simd-encode      > "target/critcmp/hex-$COMMIT_HASH-encode.ansi"   
    critcmp -g '.+?/.+?/.+?/(.+)'  --color always "hex-$COMMIT_HASH"    -f hex-simd-decode      > "target/critcmp/hex-$COMMIT_HASH-decode.ansi"
    critcmp -g '.+?/.+?/.+?/(.+)'  --color always "base64-$COMMIT_HASH" -f base64-simd-encode   > "target/critcmp/base64-$COMMIT_HASH-encode.ansi"
    critcmp -g '.+?/.+?/.+?/(.+)'  --color always "base64-$COMMIT_HASH" -f base64-simd-decode   > "target/critcmp/base64-$COMMIT_HASH-decode-ansi"

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

arm-test pkg *ARGS:
    #!/bin/bash -ex
    function armtest(){
        cross test --target armv7-unknown-linux-gnueabihf \
            --no-default-features --features 'std,unstable' \
            -p {{pkg}} -- {{ARGS}}
        cross test --target aarch64-unknown-linux-gnu \
            --no-default-features --features 'std,unstable' \
            -p {{pkg}} -- {{ARGS}}
        cross test --target armv7-unknown-linux-gnueabihf \
            --features 'unstable' \
            -p {{pkg}} -- {{ARGS}}
        cross test --target aarch64-unknown-linux-gnu \
            --features 'unstable' \
            -p {{pkg}} -- {{ARGS}}
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

test member:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    pushd crates/{{member}}
    just x86-test
    just arm-test {{member}}
    just wasm-test
    popd

test-all:
    #!/bin/bash -ex
    cd {{justfile_directory()}}

    declare -a members
    members[0]="simd-abstraction"
    members[1]="uuid-simd"
    members[2]="hex-simd"
    members[3]="base64-simd"
    members[4]="unicode-simd"
    members[5]="base32-simd"
    members[6]="vsimd"

    for member in "${members[@]}"
    do
        just test $member
    done

sync-version:
    #!/bin/bash -e
    cd {{justfile_directory()}}
    cargo set-version -p simd-abstraction   '0.8.0'
    cargo set-version -p simd-benches       '0.8.0'
    cargo set-version -p uuid-simd          '0.8.0'
    cargo set-version -p hex-simd           '0.8.0'
    cargo set-version -p base64-simd        '0.8.0'
    cargo set-version -p unicode-simd       '0.0.1'
    cargo set-version -p base32-simd        '0.0.1'
    cargo set-version -p vsimd              '0.0.1'

fmt:
    #!/bin/bash -ex
    cd {{justfile_directory()}}
    cargo fmt
    # cargo sort -w > /dev/null
