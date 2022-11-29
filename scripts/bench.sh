#!/bin/bash -ex

mkdir -p target/simd-benches
COMMIT_HASH=$(git rev-parse --short HEAD)

case $DISPATCH in
    static)
        export RUSTFLAGS="-C target-cpu=native"
        FEATURES=""
        ;;
    static-unstable)
        export RUSTFLAGS="-C target-cpu=native"
        FEATURES="unstable"
        ;;
    static-experimental)
        export RUSTFLAGS="-C target-cpu=native"
        FEATURES="unstable,parallel"
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
        echo "Unknown dispatch: $DISPATCH"
        exit 1
        ;;
esac

NAME=target/simd-benches/$COMMIT_HASH-$DISPATCH

export CARGO_TERM_QUIET=true

time cargo criterion -p simd-benches --history-id "$COMMIT_HASH" --message-format json --features "$FEATURES" "$@" > "$NAME.jsonl"

python3 ./scripts/analyze.py "./target/simd-benches/$COMMIT_HASH-$DISPATCH.jsonl" > "$NAME.md"

if which bat; then
    bat --paging=never "$NAME.md"
else
    cat "$NAME.md"
fi
