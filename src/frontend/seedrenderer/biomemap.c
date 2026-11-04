// Thin WebAssembly-friendly wrapper around cubiomes (Conflux-Union fork).
// Targets Minecraft Java 26.3, Overworld only.
#include <stdint.h>
#include <stdlib.h>
#include "generator.h"
#include "util.h"
#include "biomes.h"

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#define EXPORT EMSCRIPTEN_KEEPALIVE
#else
#define EXPORT __attribute__((visibility("default")))
#endif

static Generator g;
static unsigned char colors[256][3];
static int ready = 0;

static int *biomeBuf = NULL;
static size_t biomeBufCap = 0;
static unsigned char *pixBuf = NULL;
static size_t pixBufCap = 0;

// Sets the world seed (given as two unsigned 32-bit halves) for 26.3.
EXPORT void bm_init(uint32_t seedLo, uint32_t seedHi)
{
    uint64_t seed = ((uint64_t) seedHi << 32) | seedLo;
    setupGenerator(&g, MC_26_3, 0);
    applySeed(&g, DIM_OVERWORLD, seed);
    initBiomeColors(colors);
    ready = 1;
}

// Biome id at a block position (surface level), or -1 on failure.
EXPORT int bm_biome_at(int x, int z)
{
    if (!ready)
        return -1;
    return getBiomeAt(&g, 1, x, 63, z);
}

// Renders a w*h grid of cells to RGBA. (x, z) is the north-west cell in
// units of `scale` blocks, scale must be one of 1, 4, 16, 64, 256.
// Row 0 is the northernmost row (smallest z). Returns a pointer into
// module memory that stays valid until the next bm_tile call, or NULL.
EXPORT unsigned char *bm_tile(int x, int z, int w, int h, int scale)
{
    if (!ready || w <= 0 || h <= 0)
        return NULL;

    Range r = { scale, x, z, w, h, scale == 1 ? 63 : 15, 1 };

    size_t need = getMinCacheSize(&g, r.scale, r.sx, r.sy, r.sz);
    if (need > biomeBufCap) {
        free(biomeBuf);
        biomeBuf = (int *) malloc(need * sizeof(int));
        biomeBufCap = biomeBuf ? need : 0;
        if (!biomeBuf)
            return NULL;
    }
    if (genBiomes(&g, biomeBuf, r) != 0)
        return NULL;

    size_t pix = (size_t) w * h * 4;
    if (pix > pixBufCap) {
        free(pixBuf);
        pixBuf = (unsigned char *) malloc(pix);
        pixBufCap = pixBuf ? pix : 0;
        if (!pixBuf)
            return NULL;
    }

    for (size_t i = 0; i < (size_t) w * h; i++) {
        int id = biomeBuf[i];
        const unsigned char *c = colors[(id >= 0 && id < 256) ? id : 0];
        pixBuf[i * 4 + 0] = c[0];
        pixBuf[i * 4 + 1] = c[1];
        pixBuf[i * 4 + 2] = c[2];
        pixBuf[i * 4 + 3] = 255;
    }
    return pixBuf;
}
