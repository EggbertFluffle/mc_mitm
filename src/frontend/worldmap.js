import { fetchServerInfo } from "./api";

const cursorCoords = document.getElementById('cursor-coords');
const playerCoords = document.getElementById('player-coords');

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

    stepLadder: [1, 4, 16, 64, 128, 512, 1024, 4096, 8192]
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
