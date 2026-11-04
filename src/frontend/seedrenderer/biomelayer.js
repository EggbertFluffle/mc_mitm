import { seedToUint32Pair } from "./seed.js";

// Draws the Minecraft biome map (Java 26.3) behind the coordinate grid.
//
// The map is rendered in a Web Worker as one image covering the visible area
// plus a margin. That image is positioned in world coordinates, so it keeps
// lining up while the user pans or zooms; a fresh one is requested only when
// the view leaves it or the zoom needs a different sampling resolution.

// Horizontal resolutions cubiomes supports: blocks per sampled cell.
const SAMPLE_SCALES = [1, 4, 16, 64, 256];

// Pick the coarsest resolution whose cells stay at or below this many pixels,
// which keeps each tile to roughly 100k cells (a few hundred ms to generate).
const MAX_CELL_PIXELS = 12;

// Extra area generated around the view, as a fraction of the view size.
const MARGIN = 0.5;

// Hard cap on cells per tile; the margin is dropped if a tile would exceed it.
const MAX_CELLS = 160000;

// The plane labels a block by its centre point, so a block's square spans
// +-0.5 around its coordinate.
const HALF_BLOCK = 0.5;

// view = { width, height, ox, oy, scale }
//   ox, oy: pixel position of the origin; scale: pixels per block.
// Plane y points up while Minecraft's +Z points south, so plane y = -Z and
// Z grows downward on screen. Tiles come back with north at the top, so the
// image needs no flipping.
export function createBiomeLayer(seed, onUpdate) {
    const worker = new Worker(
        new URL("./biomeworker.js", import.meta.url),
        { type: "module" }
    );
    const [lo, hi] = seedToUint32Pair(seed);
    worker.postMessage({
        type: "init",
        wasmUrl: new URL("./biomemap.wasm", import.meta.url).href,
        lo: lo,
        hi: hi
    });

    // The tile currently drawn: { canvas, x, z, w, h, s } with x, z, w, h in cells.
    let current = null;
    let busy = false;
    let failed = false;
    let lastView = null;
    let requestId = 0;

    function sampleScaleFor(pixelsPerBlock) {
        let chosen = SAMPLE_SCALES[0];
        for (const s of SAMPLE_SCALES)
            if (s * pixelsPerBlock <= MAX_CELL_PIXELS)
                chosen = s;
        return chosen;
    }

    // Visible rectangle in block coordinates.
    function visibleBlocks(view) {
        return {
            x0: (0 - view.ox) / view.scale + HALF_BLOCK,
            x1: (view.width - view.ox) / view.scale + HALF_BLOCK,
            z0: (0 - view.oy) / view.scale + HALF_BLOCK,
            z1: (view.height - view.oy) / view.scale + HALF_BLOCK
        };
    }

    function covers(tile, vis) {
        return vis.x0 >= tile.x * tile.s
            && vis.x1 <= (tile.x + tile.w) * tile.s
            && vis.z0 >= tile.z * tile.s
            && vis.z1 <= (tile.z + tile.h) * tile.s;
    }

    function cellRange(vis, s, margin) {
        const mx = (vis.x1 - vis.x0) * margin;
        const mz = (vis.z1 - vis.z0) * margin;
        const x = Math.floor((vis.x0 - mx) / s);
        const z = Math.floor((vis.z0 - mz) / s);
        return {
            x: x,
            z: z,
            w: Math.ceil((vis.x1 + mx) / s) - x,
            h: Math.ceil((vis.z1 + mz) / s) - z
        };
    }

    worker.onmessage = (event) => {
        const message = event.data;

        if (message.type === "error") {
            busy = false;
            failed = true;
            console.error("Biome map worker:", message.message);
            return;
        }

        if (message.type !== "tile")
            return;

        const canvas = document.createElement("canvas");
        canvas.width = message.w;
        canvas.height = message.h;
        canvas.getContext("2d").putImageData(
            new ImageData(new Uint8ClampedArray(message.buffer), message.w, message.h),
            0, 0
        );

        current = { canvas, x: message.x, z: message.z, w: message.w, h: message.h, s: message.scale };
        busy = false;
        onUpdate();

        // The view may have moved while this tile was being generated.
        update(lastView);
    };

    worker.onerror = (event) => {
        failed = true;
        console.error("Biome map worker failed:", event.message);
    };

    // Requests a new tile if the current one doesn't suit the view.
    function update(view) {
        lastView = view;
        if (failed || busy || !view)
            return;

        const s = sampleScaleFor(view.scale);
        const vis = visibleBlocks(view);
        if (current && current.s === s && covers(current, vis))
            return;

        let range = cellRange(vis, s, MARGIN);
        if (range.w * range.h > MAX_CELLS)
            range = cellRange(vis, s, 0);

        busy = true;
        worker.postMessage({
            type: "tile",
            id: ++requestId,
            x: range.x,
            z: range.z,
            w: range.w,
            h: range.h,
            scale: s
        });
    }

    function draw(ctx, view) {
        if (!current)
            return;

        // Blocks are squares, not a smoothed gradient.
        ctx.imageSmoothingEnabled = false;

        const cellPx = current.s * view.scale;
        const left = view.ox + (current.x * current.s - HALF_BLOCK) * view.scale;
        const top = view.oy + (current.z * current.s - HALF_BLOCK) * view.scale;

        // Only draw the part of the image that is on screen, so a huge
        // zoomed-in destination rectangle never reaches the canvas.
        const sx0 = Math.max(0, Math.floor((0 - left) / cellPx));
        const sx1 = Math.min(current.w, Math.ceil((view.width - left) / cellPx));
        const sy0 = Math.max(0, Math.floor((0 - top) / cellPx));
        const sy1 = Math.min(current.h, Math.ceil((view.height - top) / cellPx));
        if (sx1 <= sx0 || sy1 <= sy0)
            return;

        ctx.drawImage(
            current.canvas,
            sx0, sy0, sx1 - sx0, sy1 - sy0,
            left + sx0 * cellPx, top + sy0 * cellPx,
            (sx1 - sx0) * cellPx, (sy1 - sy0) * cellPx
        );
    }

    return { update, draw };
}
