function x86_test() {
    cargo test                                                              $@
    cargo test --features 'unstable'                                        $@

    cargo test --no-default-features --features ''                          $@
    cargo test --no-default-features --features 'alloc'                     $@
    cargo test --no-default-features --features 'std'                       $@
    
    cargo test --no-default-features --features 'detect'                    $@
    cargo test --no-default-features --features 'alloc,detect'              $@
    cargo test --no-default-features --features 'std,detect'                $@
    
    cargo test --no-default-features --features 'unstable'                  $@
    cargo test --no-default-features --features 'alloc,unstable'            $@
    cargo test --no-default-features --features 'std,unstable'              $@

    cargo test --no-default-features --features 'detect,unstable'           $@
    cargo test --no-default-features --features 'alloc,detect,unstable'     $@
    cargo test --no-default-features --features 'std,detect,unstable'       $@
}

function arm_test() {
    cross test --target armv7-unknown-linux-gnueabihf                                                               $@
    cross test --target armv7-unknown-linux-gnueabihf   --features 'unstable'                                       $@

    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features ''                         $@
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'alloc'                    $@
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'std'                      $@
    
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'detect'                   $@
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'alloc,detect'             $@
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'std,detect'               $@
    
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'unstable'                 $@
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'alloc,unstable'           $@
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'std,unstable'             $@

    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'detect,unstable'          $@
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'alloc,detect,unstable'    $@
    cross test --target armv7-unknown-linux-gnueabihf   --no-default-features --features 'std,detect,unstable'      $@

    cross test --target aarch64-unknown-linux-gnu                                                                   $@
    cross test --target aarch64-unknown-linux-gnu       --features 'unstable'                                       $@

    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features ''                         $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'alloc'                    $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'std'                      $@
    
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'detect'                   $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'alloc,detect'             $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'std,detect'               $@
    
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'unstable'                 $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'alloc,unstable'           $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'std,unstable'             $@
    
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'detect,unstable'          $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'alloc,detect,unstable'    $@
    cross test --target aarch64-unknown-linux-gnu       --no-default-features --features 'std,detect,unstable'      $@
}

function wasm_test() {
    wasm-pack test --node                                                              $@
    wasm-pack test --node --features 'unstable'                                        $@

    wasm-pack test --node --no-default-features --features ''                          $@
    wasm-pack test --node --no-default-features --features 'alloc'                     $@
    wasm-pack test --node --no-default-features --features 'std'                       $@
    
    wasm-pack test --node --no-default-features --features 'detect'                    $@
    wasm-pack test --node --no-default-features --features 'alloc,detect'              $@
    wasm-pack test --node --no-default-features --features 'std,detect'                $@
    
    wasm-pack test --node --no-default-features --features 'unstable'                  $@
    wasm-pack test --node --no-default-features --features 'alloc,unstable'            $@
    wasm-pack test --node --no-default-features --features 'std,unstable'              $@

    wasm-pack test --node --no-default-features --features 'detect,unstable'           $@
    wasm-pack test --node --no-default-features --features 'alloc,detect,unstable'     $@
    wasm-pack test --node --no-default-features --features 'std,detect,unstable'       $@
}
