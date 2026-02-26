const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${protocol}//${window.location.host}/ws`;

const socket = new WebSocket(wsUrl);
let connectionStatus = 'disconnected';
let editorEl = null;

// Calculate approximate grid size
function roughGrid(editorEl, fontSizePx) {
    const w = editorEl.clientWidth;
    const h = editorEl.clientHeight;

    const charWidth = fontSizePx * 0.6;
    const lineHeight = fontSizePx * 1.5;

    return {
        viewport_rows: Math.max(1, Math.floor(h / lineHeight)),
        viewport_columns: Math.max(1, Math.floor(w / charWidth)),
    };
}

socket.addEventListener('open', () => {
    console.log('Connected to WebSocket');
    connectionStatus = 'connected';
    socket.send(JSON.stringify({ method: 'connected' }));
});

socket.addEventListener('message', (event) => {
    console.log('Received:', event.data);
    try {
        const msg = JSON.parse(event.data);
        if (msg.method === 'initialize') {
            const editorFontSize = msg.data.editor_font_size;
            console.log('Editor font size:', editorFontSize);

            const grid = roughGrid(editorEl, editorFontSize);
            socket.send(JSON.stringify({
                method: 'initialized',
                data: grid,
            }));
            connectionStatus = 'initialized';
        }
    } catch (e) {
        console.error('Failed to parse message:', e);
    }
});

socket.addEventListener('close', () => {
    console.log('Disconnected from WebSocket');
    connectionStatus = 'disconnected';
});
