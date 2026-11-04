import { fetchServerInfo, fetchGameState } from "./api.js";
import { createBiomeLayer } from "./seedrenderer/biomelayer.js";

const cursorCoords = document.getElementById('cursor-coords');
const playerCoords = document.getElementById('player-coords');

function loadMarkerImage(file) {
    const img = new Image();
    img.src = `images/${file}.png`;
    return img;
}

// Human-readable mob type -> image used for its map marker.
const markerImages = {
    player: loadMarkerImage('steve'),
    cow: loadMarkerImage('cow'),
    pig: loadMarkerImage('pig'),
    sheep: loadMarkerImage('sheep'),
    skeleton: loadMarkerImage('skeleton'),
    spider: loadMarkerImage('spider'),
    villager: loadMarkerImage('villager'),
    wolf: loadMarkerImage('wolf'),
    zombie: loadMarkerImage('zombie'),
};

const mobTypes = Object.keys(markerImages).filter((type) => type !== 'player');

// Reverse of entities.json: numeric entity_type -> marker type name.
const ENTITY_TYPES_BY_ID = await (async () => {
    const res = await fetch('./entities.json');
    if (!res.ok) throw new Error(`entities.json failed: ${res.status}`);
    const idsByName = await res.json();
    return Object.fromEntries(
        Object.entries(idsByName).map(([name, id]) => [String(id), name])
    );
})();

// World positions of everything drawn on top of the grid.
const markers = {
    // The player always exists and starts at the origin.
    // yaw 0 faces south (+z), 90 west (-x), increasing clockwise.
    player: { x: 0, y: 0, yaw: 0 },
    // Mobs keyed by their server-assigned entity id.
    // Each value: { type: 'pig' | 'sheep' | ..., x (east), y (z, south) }
    mobs: {}
};

// Marker sizes are in screen pixels so they stay visible at every zoom level.
const PLAYER_MARKER_SIZE = 32;
const MOB_MARKER_SIZE = 26;

const colors = {
    border: "#6b4a3e",
    gridStroke: "#6b4a3e",
    crosshair: "red",
    playerArrow: "#ff5555",
    labels: "white",
    axes: "#ddc7be"
};

const mapSettings = {
    padding: 200,
    minWidth: 400,
    maxWidth: 960,
    // How many pixels one unit of distance spans.
    maxScale: 160,
    // The map represents values up to this number.
    maxRange: 20000,
    // Grid cells collapse when they get smaller than this many pixels.
    minSpacing: 32,

    stepLadder: [1, 4, 16, 64, 128, 512, 1024, 4096, 8192],

    // How often to poll the game-state API, in seconds.
    pollInterval: 0.5
};

const canvasInstance = function(seed) {
    const element = document.getElementById('coord-map');
    const ctx = element.getContext('2d');

    let scale = 40;

    let offX = 0;
    let offY = 0;

    let dragging = false;
    let dragStartX = 0;
    let dragStartY = 0;
    let startOffX = 0;
    let startOffY = 0;

    const biomeLayer = createBiomeLayer(seed, () => draw());

    function draw() {
        ctx.clearRect(0, 0, element.width, element.height);

        const ox = Math.round(element.width / 2 + offX);
        const oy = Math.round(element.height / 2 + offY);

        const view = { width: element.width, height: element.height, ox: ox, oy: oy, scale: scale };
        biomeLayer.draw(ctx, view);
        biomeLayer.update(view);

        ctx.strokeStyle = colors.gridStroke;
        ctx.lineWidth = 1;

        const step = (() => {
            for (const s of mapSettings.stepLadder)
                if (s * scale >= mapSettings.minSpacing)
                    return s;
            return mapSettings.stepLadder[mapSettings.stepLadder.length - 1];
        })();
        const spacing = step * scale;

        // Draw vertical lines
        const iMin = Math.ceil(-ox / spacing);
        const iMax = Math.floor((element.width - ox) / spacing);

        for (let i = iMin; i <= iMax; i++) {
            if (i === 0)
                continue;

            const px = ox + i * spacing;
            ctx.beginPath();
            ctx.moveTo(px + 0.5, 0);
            ctx.lineTo(px + 0.5, element.height);
            ctx.stroke();
        }

        // Draw horizontal lines
        const kMin = Math.ceil((oy - element.height) / spacing);
        const kMax = Math.floor(oy / spacing);

        for (let k = kMin; k <= kMax; k++) {
            if (k === 0)
                continue;

            const py = oy - k * spacing;
            ctx.beginPath();
            ctx.moveTo(0, py + 0.5);
            ctx.lineTo(element.width, py + 0.5);
            ctx.stroke();
        }

        // Axes
        ctx.strokeStyle = colors.axes;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(ox, 0);
        ctx.lineTo(ox, element.height);
        ctx.moveTo(0, oy);
        ctx.lineTo(element.width, oy);
        ctx.stroke();

        // Axis tick labels
        ctx.fillStyle = colors.labels;
        ctx.font = '10px monospace';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';

        for (let i = iMin; i <= iMax; i++) {
            if (i === 0)
                continue;
            ctx.fillText(String(i * step), ox + i * spacing, oy + 4);
        }

        for (let k = kMin; k <= kMax; k++) {
            if (k === 0)
                continue;
            const py = oy - k * spacing;
            const offset = k > 0 ? -12 : 0;
            // z points down (south), so the line above the origin is negative z.
            ctx.fillText(String(-k * step), ox - 4, py + offset);
        }

        // Axis letters and compass labels (chunkbase standard:
        // -x west, +x east, -z north, +z south).
        ctx.font = 'bold 12px monospace';
        ctx.fillText('X', element.width - 14, oy - 18);
        ctx.fillText('Z', ox - 14, element.height - 18);

        ctx.font = '10px monospace';
        ctx.fillText('N', element.width / 2, 4);
        ctx.fillText('S', element.width / 2, element.height - 14);
        ctx.textBaseline = 'middle';
        ctx.textAlign = 'left';
        ctx.fillText('W', 4, element.height / 2);
        ctx.textAlign = 'right';
        ctx.fillText('E', element.width - 4, element.height / 2);
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';

        // Origin Marker
        ctx.fillStyle = colors.border;
        ctx.beginPath();
        ctx.arc(ox, oy, 3, 0, Math.PI * 2);
        ctx.fill();

        // Player and mob markers. World coords -> screen:
        // screen x grows right (east), world z grows down (south).
        function drawMarker(type, coords, size) {
            const img = markerImages[type];
            if (!img.complete || img.naturalWidth === 0)
                return;

            const px = ox + coords.x * scale;
            const py = oy + coords.y * scale;

            // Skip markers outside the viewport (plus a marker of margin).
            if (px < -size || px > element.width + size ||
                py < -size || py > element.height + size)
                return;

            ctx.drawImage(img, px - size / 2, py - size / 2, size, size);
            ctx.strokeStyle = colors.border;
            ctx.lineWidth = 1;
            ctx.strokeRect(px - size / 2, py - size / 2, size, size);
        }

        for (const mob of Object.values(markers.mobs))
            drawMarker(mob.type, mob, MOB_MARKER_SIZE);

        // Draw the player last so it sits on top of any overlapping mobs.
        drawMarker('player', markers.player, PLAYER_MARKER_SIZE);

        // Heading arrow: yaw 0 is south (+z, screen-down), 90 is west
        // (-x, screen-left), increasing clockwise. Rotating the screen-down
        // vector (0, 1) clockwise in canvas space reproduces that exactly.
        function drawPlayerYawArrow() {
            const px = ox + markers.player.x * scale;
            const py = oy + markers.player.y * scale;

            ctx.save();
            ctx.translate(px, py);
            ctx.rotate(markers.player.yaw * Math.PI / 180);

            // Arrow starts just past the marker edge, pointing screen-down
            // before rotation (i.e. yaw 0 -> south).
            const startY = PLAYER_MARKER_SIZE / 2 + 2;
            const tipY = startY + 16;

            ctx.strokeStyle = colors.playerArrow;
            ctx.fillStyle = colors.playerArrow;
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.moveTo(0, startY);
            ctx.lineTo(0, tipY);
            ctx.stroke();

            // Arrowhead: two short edges folding back from the tip.
            const headLength = 7;
            const spread = Math.PI / 6; // 30 degrees off the shaft
            ctx.beginPath();
            ctx.moveTo(0, tipY);
            ctx.lineTo(headLength * Math.sin(spread), tipY - headLength * Math.cos(spread));
            ctx.lineTo(-headLength * Math.sin(spread), tipY - headLength * Math.cos(spread));
            ctx.closePath();
            ctx.fill();

            ctx.restore();
        }

        drawPlayerYawArrow();
    }

    function resize() {
        element.width = Math.min(
            mapSettings.maxWidth,
            Math.max(mapSettings.minWidth, window.innerWidth - mapSettings.padding * 2)
        );
        element.height = Math.max(
            300, 
            window.innerHeight - element.offsetTop - mapSettings.padding
        );
        draw();
    }

    function zoomAt(px, py, factor) {
        const minScale = () => (element.width / 2) / mapSettings.maxRange;
        const newScale = Math.min(mapSettings.maxScale, Math.max(minScale(), scale * factor));
        const ratio = newScale / scale;

        // Already at zoom limit
        if (ratio === 1)
            return;

        offX = offX * ratio - (px - element.width / 2) * (ratio - 1);
        offY = offY * ratio - (py - element.height / 2) * (ratio - 1);
        scale = newScale;
        draw();
    }

    // Pan the map so world coords (x, z) sit at the canvas center.
    // Screen px = (width/2 + offX) + x*scale, so centering needs offX = -x*scale.
    function centerOn(x, z) {
        offX = -x * scale;
        offY = -z * scale;
        draw();
    }

    function endDrag(event) {
        if (!dragging)
            return;

        dragging = false;
        element.classList.remove('dragging');
        try {
            element.releasePointerCapture(event.pointerId);
        } catch {}
    }

    window.addEventListener('resize', resize);

    element.addEventListener('wheel', (event) => {
        event.preventDefault();
        const rect = element.getBoundingClientRect();
        const px = event.clientX - rect.left;
        const py = event.clientY - rect.top;

        // Negative deltaY zooms in
        const factor = Math.exp(-event.deltaY * 0.0015);
        zoomAt(px, py, factor);
    }, { passive: false });

    element.addEventListener('pointerdown', (event) => {
        dragging = true;
        element.setPointerCapture(event.pointerId);
        element.classList.add('dragging');
        dragStartX = event.clientX;
        dragStartY = event.clientY;
        startOffX = offX;
        startOffY = offY;
    });

    element.addEventListener('pointerup', endDrag);
    element.addEventListener('pointercancel', endDrag);

    element.addEventListener('pointermove', (event) => {
        if (dragging) {
            offX = startOffX + (event.clientX - dragStartX);
            offY = startOffY + (event.clientY - dragStartY);
        }

        const { px, py, x, y } = (() => {
            const rect = element.getBoundingClientRect();
            const px = event.clientX - rect.left;
            const py = event.clientY - rect.top;
            const x = Math.round((px - (element.width / 2 + offX)) / scale);
            const y = Math.round((py - (element.height / 2 + offY)) / scale);
            return { px, py, x, y };
        })();

        cursorCoords.textContent = `Mouse Coords: (${x}, ${y})`;
        draw();

        // Crosshair on the point under the cursor.
        if (crosshairEnabled) {
            ctx.strokeStyle = colors.crosshair;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(px + 0.5, 0);
            ctx.lineTo(px + 0.5, element.height);
            ctx.moveTo(0, py + 0.5);
            ctx.lineTo(element.width, py + 0.5);
            ctx.stroke();
        }
    });

    element.addEventListener('mouseleave', () => {
        cursorCoords.textContent = 'Mouse Coords: (0, 0)';
        draw();
    });

    return {
        resize: resize,
        draw: draw,
        centerOn: centerOn
    };
}

let crosshairEnabled = false;

const canvas = await (async () => {
    const info = await fetchServerInfo();
    return canvasInstance(info.seed);
})();

canvas.resize();

// Redraw once marker images finish loading (they load asynchronously).
for (const img of Object.values(markerImages))
    if (!img.complete)
        img.addEventListener('load', () => canvas.draw());

// Move the player marker to world coords (x, z).
// Move the player marker to world coords (x, z), optionally updating yaw.
function setPlayerPosition(x, z, yaw) {
    markers.player.x = x;
    markers.player.y = z;
    if (yaw !== undefined)
        markers.player.yaw = yaw;
    playerCoords.textContent = `Player Coords: (${x}, ${z})`;
    canvas.draw();
}

// Populate the map with a random assortment of mobs in random positions
// near the origin, replacing any previously spawned mobs.
function populateMobs(count = 12, maxRadius = 32) {
    markers.mobs = {};

    for (let i = 0; i < count; i++) {
        const type = mobTypes[Math.floor(Math.random() * mobTypes.length)];
        const angle = Math.random() * Math.PI * 2;
        const dist = Math.floor(Math.random() * maxRadius);

        markers.mobs[`fake:${i}`] = {
            type,
            x: Math.round(Math.cos(angle) * dist),
            y: Math.round(Math.sin(angle) * dist)
        };
    }

    canvas.draw();
    return markers.mobs;
}

setPlayerPosition(markers.player.x, markers.player.y);

// Sync mob markers against the latest entity snapshot: move existing mobs,
// add any newly seen entity ids, and drop mobs the server no longer reports.
function syncMobs(entities) {
    // Remove stored mobs whose id disappeared from the latest result.
    for (const id of Object.keys(markers.mobs))
        if (!(id in entities))
            delete markers.mobs[id];

    for (const [id, entity] of Object.entries(entities)) {
        const type = ENTITY_TYPES_BY_ID[entity.entity_type];

        // entity_type has no entry in entities.json (e.g. a new mob type).
        if (!type) {
            console.warn(`Unknown entity_type ${entity.entity_type} for entity ${id}`);
            continue;
        }

        markers.mobs[id] = {
            type,
            x: Math.round(entity.x),
            y: Math.round(entity.z)
        };
    }

    canvas.draw();
}

// Poll the game-state API on a fixed interval and move the player marker.
// A failed poll keeps the last known position until the next one succeeds.
async function pollGameState() {
    try {
        const state = await fetchGameState();
        setPlayerPosition(
            Math.round(state.player.x),
            Math.round(state.player.z),
            state.player.yaw
        );
        syncMobs(state.entities);

        // Keep the viewport locked onto the player while follow mode is on.
        if (followPlayerEnabled)
            canvas.centerOn(markers.player.x, markers.player.y);
    } catch (err) {
        console.error('Failed to poll game state:', err);
    }
}

setInterval(pollGameState, mapSettings.pollInterval * 1000);

export { setPlayerPosition, populateMobs };

const crosshairButton = document.getElementById('toggle-crosshair');
function updateCrosshair() {
    crosshairButton.setAttribute('aria-pressed', String(crosshairEnabled));
    crosshairButton.classList.toggle('active', crosshairEnabled);
    canvas.draw();
}

crosshairButton.addEventListener('click', () => {
    crosshairEnabled = !crosshairEnabled;
    updateCrosshair();
});

updateCrosshair();

const centerButton = document.getElementById('center-marker');
centerButton.addEventListener('click', () => {
    canvas.centerOn(markers.player.x, markers.player.y);
});

let followPlayerEnabled = false;
const followButton = document.getElementById('follow-player');
function updateFollow() {
    followButton.setAttribute('aria-pressed', String(followPlayerEnabled));
    followButton.classList.toggle('active', followPlayerEnabled);
}

followButton.addEventListener('click', () => {
    followPlayerEnabled = !followPlayerEnabled;
    // Snap to the player immediately instead of waiting for the next poll.
    if (followPlayerEnabled)
        canvas.centerOn(markers.player.x, markers.player.y);
    updateFollow();
});

updateFollow();
