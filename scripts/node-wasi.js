"use strict";
const fs = require("fs");
const { WASI } = require("wasi");

const wasi = new WASI();
const importObject = { wasi_snapshot_preview1: wasi.wasiImport };

(async () => {
    const wasmPath = process.argv[2]
    const wasm = await WebAssembly.compile(
        fs.readFileSync(wasmPath)
    );
    const instance = await WebAssembly.instantiate(wasm, importObject);

    wasi.start(instance);
})();