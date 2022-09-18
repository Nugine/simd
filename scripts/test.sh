function x86_test() {
    cargo test --no-default-features --features 'std,unstable' $@
    cargo test --features 'unstable' $@
}

function arm_test() {
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'std,unstable' $@
    cross test --target armv7-unknown-linux-gnueabihf   --features 'unstable' $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'std,unstable' $@
    cross test --target aarch64-unknown-linux-gnu       --features 'unstable' $@
}

function wasm_test() {
    wasm-pack test --node $@
}
