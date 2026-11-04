// Web Worker that runs cubiomes (compiled to WebAssembly) off the main thread.
//
// Messages in:
//   { type: 'init', wasmUrl, lo, hi }                 set up the world seed
//   { type: 'tile', id, x, z, w, h, scale }           render a w*h grid of cells
// Messages out:
//   { type: 'ready' }
//   { type: 'tile', id, x, z, w, h, scale, buffer }   RGBA pixels, row 0 = north
//   { type: 'error', id, message }

let wasm = null;
let resolveReady;
const ready = new Promise((resolve) => { resolveReady = resolve; });

// The module only imports a few WASI functions (from libc stdio that the biome
// code never actually calls), so minimal stubs are enough.
const imports = {
    wasi_snapshot_preview1: {
        fd_close: () => 0,
        fd_fdstat_get: () => 0,
        fd_seek: () => 70, // ESPIPE
        fd_write: () => 0,
        proc_exit: (code) => { throw new Error('wasm exited with code ' + code); }
    }
};

async function init(message) {
    const response = await fetch(message.wasmUrl);
    const bytes = await response.arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes, imports);

    wasm = instance.exports;
    wasm._initialize();
    wasm.bm_init(message.lo, message.hi);

    resolveReady();
    self.postMessage({ type: 'ready' });
}

async function tile(message) {
    await ready;

    const { id, x, z, w, h, scale } = message;
    const ptr = wasm.bm_tile(x, z, w, h, scale);
    if (!ptr) {
        self.postMessage({ type: 'error', id, message: 'bm_tile failed' });
        return;
    }

    // Copy out of WASM memory (it can grow and detach the old buffer).
    const pixels = new Uint8ClampedArray(wasm.memory.buffer, ptr, w * h * 4).slice();
    self.postMessage(
        { type: 'tile', id, x, z, w, h, scale, buffer: pixels.buffer },
        [pixels.buffer]
    );
}

self.onmessage = (event) => {
    const message = event.data;
    const task = message.type === 'init' ? init(message) : tile(message);
    task.catch((error) => {
        self.postMessage({ type: 'error', id: message.id, message: String(error) });
    });
};
