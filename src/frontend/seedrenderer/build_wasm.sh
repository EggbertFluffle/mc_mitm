#!/usr/bin/env bash
# Rebuilds biomemap.wasm from the Conflux-Union cubiomes fork.
# Requires: git, python3, and `pip install ziglang` (zig's bundled clang targets wasm,
# so no Emscripten install is needed).
set -euo pipefail

[ -d cubiomes ] || git clone --depth 1 https://github.com/Conflux-Union/cubiomes.git cubiomes
C=cubiomes

python3 -m ziglang cc -target wasm32-wasi -O2 -mexec-model=reactor \
    -ffunction-sections -fdata-sections -Wl,--gc-sections \
    -Wl,--export=bm_init -Wl,--export=bm_biome_at -Wl,--export=bm_tile \
    -I"$C" -o biomemap.wasm biomemap.c \
    "$C"/generator.c "$C"/biomenoise.c "$C"/biomes.c "$C"/layers.c \
    "$C"/noise.c "$C"/util.c "$C"/xradv.c "$C"/terrainnoise.c
