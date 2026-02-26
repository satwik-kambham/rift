const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${protocol}//${window.location.host}/ws`;

const socket = new WebSocket(wsUrl);
let connectionStatus = 'disconnected';

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
