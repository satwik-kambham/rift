const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${protocol}//${window.location.host}/ws`;

const socket = new WebSocket(wsUrl);
let connectionStatus = 'disconnected';
let editorEl = null;

function getClassesFromAttributes(attrs) {
    const classes = ['segment'];

    if (attrs & 0x10) {
        // HIGHLIGHT_NONE
    }
    if (attrs & 0x20) {
        classes.push('highlight-white');
    }
    if (attrs & 0x40) {
        classes.push('highlight-red');
    }
    if (attrs & 0x80) {
        classes.push('highlight-orange');
    }
    if (attrs & 0x100) {
        classes.push('highlight-blue');
    }
    if (attrs & 0x200) {
        classes.push('highlight-green');
    }
    if (attrs & 0x400) {
        classes.push('highlight-purple');
    }
    if (attrs & 0x800) {
        classes.push('highlight-yellow');
    }
    if (attrs & 0x1000) {
        classes.push('highlight-gray');
    }
    if (attrs & 0x2000) {
        classes.push('highlight-turquoise');
    }

    if (attrs & 0x1) {
        classes.push('gutter');
    }
    if (attrs & 0x2) {
        classes.push('underline');
    }
    if (attrs & 0x4) {
        classes.push('selection');
    }
    if (attrs & 0x8) {
        classes.push('cursor');
    }

    if (attrs & 0x4000) {
        classes.push('diag-hint');
    }
    if (attrs & 0x8000) {
        classes.push('diag-info');
    }
    if (attrs & 0x10000) {
        classes.push('diag-warning');
    }
    if (attrs & 0x20000) {
        classes.push('diag-error');
    }

    return classes;
}

function updateEditor(bufferView) {
    if (!editorEl) {
        console.error('Editor element not initialized');
        return;
    }

    editorEl.innerHTML = '';

    for (const line of bufferView) {
        const lineEl = document.createElement('div');
        lineEl.className = 'line';

        for (const [text, attrs] of line) {
            const segmentEl = document.createElement('span');
            segmentEl.className = getClassesFromAttributes(attrs).join(' ');
            segmentEl.textContent = text;
            lineEl.appendChild(segmentEl);
        }

        editorEl.appendChild(lineEl);
    }
}

function roughGrid(fontSizePx) {
    const w = editorEl.clientWidth;
    const h = editorEl.clientHeight;

    const charWidth = fontSizePx * 0.6;
    const lineHeight = fontSizePx * 1.5;

    return {
        viewport_rows: Math.max(1, Math.floor(h / lineHeight) - 3),
        viewport_columns: Math.max(1, Math.floor(w / charWidth) - 3),
    };
}

document.addEventListener('DOMContentLoaded', () => {
    editorEl = document.getElementById('editor');
    if (!editorEl) {
        console.error('Could not find editor element');
    }
});

socket.addEventListener('open', () => {
    connectionStatus = 'connected';
    socket.send(JSON.stringify({ method: 'connected' }));
});

socket.addEventListener('message', (event) => {
    try {
        const msg = JSON.parse(event.data);
        if (msg.method === 'initialize') {
            const editorFontSize = msg.data.editor_font_size;

            if (editorEl) {
                editorEl.style.fontSize = editorFontSize + 'px';
            }

            const grid = roughGrid(editorFontSize);
            socket.send(JSON.stringify({
                method: 'initialized',
                data: grid,
            }));
            connectionStatus = 'initialized';
        } else if (msg.method === 'render') {
            const bufferView = msg.data;
            updateEditor(bufferView);
        }
    } catch (e) {
        console.error('Failed to parse message:', e);
    }
});

socket.addEventListener('close', () => {
    console.log('Disconnected from WebSocket');
    connectionStatus = 'disconnected';
});
