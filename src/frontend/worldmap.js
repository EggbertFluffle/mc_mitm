import { fetchServerInfo, fetchGameState } from "./api.js";

const cursorCoords = document.getElementById('cursor-coords');
const playerCoords = document.getElementById('player-coords');

function loadMarkerImage(file) {
    const img = new Image();
    img.src = `images/${file}`;
    return img;
}

// Human-readable mob type -> image used for its map marker.
const markerImages = {
    player: loadMarkerImage('steve.jpg'),
    pig: loadMarkerImage('pig.jpg'),
    sheep: loadMarkerImage('sheep.jpg'),
    cow: loadMarkerImage('cow.jpg')
};

const mobTypes = Object.keys(markerImages).filter((type) => type !== 'player');

// World positions of everything drawn on top of the grid.
const markers = {
    // The player always exists and starts at the origin.
    player: { x: 0, y: 0 },
    // Each mob: { type: 'pig' | 'sheep' | 'cow', x, y }
    mobs: []
};

// Marker sizes are in screen pixels so they stay visible at every zoom level.
const PLAYER_MARKER_SIZE = 32;
const MOB_MARKER_SIZE = 26;

const colors = {
    border: "#6b4a3e",
    gridStroke: "#6b4a3e",
    crosshair: "red",
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
    pollInterval: 1
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

    function draw() {
        ctx.clearRect(0, 0, element.width, element.height);

        const ox = Math.round(element.width / 2 + offX);
        const oy = Math.round(element.height / 2 + offY);

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
            ctx.fillText(String(k * step), ox - 4, py + offset);
        }

        // Origin Marker
        ctx.fillStyle = colors.border;
        ctx.beginPath();
        ctx.arc(ox, oy, 3, 0, Math.PI * 2);
        ctx.fill();

        // Player and mob markers. World coords -> screen:
        // screen x grows right, world y grows up.
        function drawMarker(type, coords, size) {
            const img = markerImages[type];
            if (!img.complete || img.naturalWidth === 0)
                return;

            const px = ox + coords.x * scale;
            const py = oy - coords.y * scale;

            // Skip markers outside the viewport (plus a marker of margin).
            if (px < -size || px > element.width + size ||
                py < -size || py > element.height + size)
                return;

            ctx.drawImage(img, px - size / 2, py - size / 2, size, size);
            ctx.strokeStyle = colors.border;
            ctx.lineWidth = 1;
            ctx.strokeRect(px - size / 2, py - size / 2, size, size);
        }

        for (const mob of markers.mobs)
            drawMarker(mob.type, mob, MOB_MARKER_SIZE);

        // Draw the player last so it sits on top of any overlapping mobs.
        drawMarker('player', markers.player, PLAYER_MARKER_SIZE);
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
            const y = Math.round((element.height / 2 + offY - py) / scale);
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
        draw: draw
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

// Move the player marker to world coords (x, y).
function setPlayerPosition(x, y) {
    markers.player.x = x;
    markers.player.y = y;
    playerCoords.textContent = `Player Coords: (${x}, ${y})`;
    canvas.draw();
}

// Populate the map with a random assortment of mobs in random positions
// near the origin, replacing any previously spawned mobs.
function populateMobs(count = 12, maxRadius = 32) {
    markers.mobs = [];

    for (let i = 0; i < count; i++) {
        const type = mobTypes[Math.floor(Math.random() * mobTypes.length)];
        const angle = Math.random() * Math.PI * 2;
        const dist = Math.floor(Math.random() * maxRadius);

        markers.mobs.push({
            type,
            x: Math.round(Math.cos(angle) * dist),
            y: Math.round(Math.sin(angle) * dist)
        });
    }

    canvas.draw();
    return markers.mobs;
}

setPlayerPosition(markers.player.x, markers.player.y);

// Poll the game-state API on a fixed interval and move the player marker.
// A failed poll keeps the last known position until the next one succeeds.
async function pollGameState() {
    try {
        const state = await fetchGameState();
        setPlayerPosition(
            Math.round(state.player.x),
            Math.round(state.player.z)
        );
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
