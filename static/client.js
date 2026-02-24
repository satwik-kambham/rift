const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${protocol}//${window.location.host}/ws`;

const socket = new WebSocket(wsUrl);

socket.addEventListener('open', () => {
    console.log('Connected to WebSocket');
    socket.send('Hello from client!');
});

socket.addEventListener('message', (event) => {
    console.log('Received:', event.data);
});

socket.addEventListener('close', () => {
    console.log('Disconnected from WebSocket');
});
