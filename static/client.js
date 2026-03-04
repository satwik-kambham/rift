const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${protocol}//${window.location.host}/ws`;

const PING_ENABLED = true;
const PING_INTERVAL_MS = 5000;

const socket = new WebSocket(wsUrl);
let connectionStatus = 'disconnected';
let editorEl = null;
let wheelDeltaAccumulator = 0;
let touchLastY = null;
let pingStart = null;
let pingInterval = null;
let lastLatency = null;
let mediaRecorder = null;
let audioChunks = [];
let startRecordBtn = null;
let stopRecordBtn = null;

const WHEEL_STEP_PX = 40;
const WHEEL_LINE_PX = 40;
const MAX_ACTIONS_PER_WHEEL_EVENT = 8;

function getClassesFromAttributes(attrs) {
    const classes = ['segment'];

    if (attrs & 0x10) {
        classes.push('highlight-none');
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

function canSendActions() {
    return connectionStatus === 'initialized' && socket.readyState === WebSocket.OPEN;
}

function normalizeWheelDeltaY(event) {
    if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) {
        return event.deltaY * WHEEL_LINE_PX;
    }
    if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) {
        return event.deltaY * window.innerHeight;
    }
    return event.deltaY;
}

function onWheelScroll(event) {
    if (event.ctrlKey || event.metaKey) {
        return;
    }

    if (!canSendActions()) {
        return;
    }

    event.preventDefault();

    wheelDeltaAccumulator += normalizeWheelDeltaY(event);

    let emitted = 0;
    while (
        Math.abs(wheelDeltaAccumulator) >= WHEEL_STEP_PX &&
        emitted < MAX_ACTIONS_PER_WHEEL_EVENT
    ) {
        if (wheelDeltaAccumulator > 0) {
            runAction('scroll-down');
            wheelDeltaAccumulator -= WHEEL_STEP_PX;
        } else {
            runAction('scroll-up');
            wheelDeltaAccumulator += WHEEL_STEP_PX;
        }
        emitted += 1;
    }
}

function onTouchStart(event) {
    if (event.touches.length !== 1) {
        touchLastY = null;
        return;
    }
    touchLastY = event.touches[0].clientY;
}

function onTouchMove(event) {
    if (event.touches.length !== 1 || touchLastY === null) {
        touchLastY = null;
        return;
    }

    const currentY = event.touches[0].clientY;
    const deltaY = touchLastY - currentY;
    touchLastY = currentY;

    if (!canSendActions()) {
        return;
    }

    event.preventDefault();
    wheelDeltaAccumulator += deltaY;

    let emitted = 0;
    while (
        Math.abs(wheelDeltaAccumulator) >= WHEEL_STEP_PX &&
        emitted < MAX_ACTIONS_PER_WHEEL_EVENT
    ) {
        if (wheelDeltaAccumulator > 0) {
            runAction('scroll-down');
            wheelDeltaAccumulator -= WHEEL_STEP_PX;
        } else {
            runAction('scroll-up');
            wheelDeltaAccumulator += WHEEL_STEP_PX;
        }
        emitted += 1;
    }
}

function onTouchEnd() {
    touchLastY = null;
    wheelDeltaAccumulator = 0;
}

document.addEventListener('DOMContentLoaded', () => {
    editorEl = document.getElementById('editor');
    if (!editorEl) {
        console.error('Could not find editor element');
    }

    const statusEl = document.createElement('div');
    statusEl.id = 'connection-status';
    statusEl.style.cssText = 'position: fixed; top: 10px; right: 10px; padding: 4px 8px; background: #333; color: #fff; font-family: monospace; font-size: 12px; border-radius: 4px; z-index: 1000;';
    statusEl.textContent = 'Disconnected';
    document.body.appendChild(statusEl);

    startRecordBtn = document.createElement('button');
    startRecordBtn.id = 'start-record';
    startRecordBtn.textContent = 'Start Recording';
    startRecordBtn.style.cssText = 'position: fixed; top: 10px; left: 10px; padding: 4px 8px; background: #4CAF50; color: white; border: none; border-radius: 4px; font-family: monospace; font-size: 12px; cursor: pointer; z-index: 1000;';
    document.body.appendChild(startRecordBtn);

    stopRecordBtn = document.createElement('button');
    stopRecordBtn.id = 'stop-record';
    stopRecordBtn.textContent = 'Stop Recording';
    stopRecordBtn.style.cssText = 'position: fixed; top: 10px; left: 120px; padding: 4px 8px; background: #f44336; color: white; border: none; border-radius: 4px; font-family: monospace; font-size: 12px; cursor: pointer; z-index: 1000; display: none;';
    stopRecordBtn.style.display = 'none';
    document.body.appendChild(stopRecordBtn);

    startRecordBtn.addEventListener('click', startRecording);
    stopRecordBtn.addEventListener('click', stopRecording);

    window.addEventListener('wheel', onWheelScroll, { passive: false });
    window.addEventListener('touchstart', onTouchStart, { passive: true });
    window.addEventListener('touchmove', onTouchMove, { passive: false });
    window.addEventListener('touchend', onTouchEnd, { passive: true });
    window.addEventListener('touchcancel', onTouchEnd, { passive: true });
});

socket.addEventListener('open', () => {
    connectionStatus = 'connected';
    socket.send(JSON.stringify({ method: 'connected' }));
    startPing();
    updateStatusDisplay(null);
});

function runAction(actionName) {
    if (!canSendActions()) {
        return;
    }
    socket.send(JSON.stringify({ method: 'run_action', data: actionName }));
}

async function startRecording() {
    try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        mediaRecorder = new MediaRecorder(stream);
        audioChunks = [];

        mediaRecorder.ondataavailable = (event) => {
            if (event.data.size > 0) {
                audioChunks.push(event.data);
            }
        };

        mediaRecorder.onstop = async () => {
            const audioBlob = new Blob(audioChunks, { type: 'audio/webm' });
            const arrayBuffer = await audioBlob.arrayBuffer();

            if (socket.readyState === WebSocket.OPEN) {
                socket.send(arrayBuffer);
                console.log('Audio sent to server');
            } else {
                console.error('WebSocket not connected');
            }

            stream.getTracks().forEach(track => track.stop());
        };

        mediaRecorder.start();
        startRecordBtn.style.display = 'none';
        stopRecordBtn.style.display = 'block';
        console.log('Recording started');
    } catch (err) {
        console.error('Error accessing microphone:', err);
        alert('Could not access microphone. Please grant permission.');
    }
}

function stopRecording() {
    if (mediaRecorder && mediaRecorder.state !== 'inactive') {
        mediaRecorder.stop();
        startRecordBtn.style.display = 'block';
        stopRecordBtn.style.display = 'none';
        console.log('Recording stopped');
    }
}

function startPing() {
    if (!PING_ENABLED) return;
    pingInterval = setInterval(() => {
        if (socket.readyState === WebSocket.OPEN) {
            pingStart = Date.now();
            socket.send(JSON.stringify({ method: 'ping', data: { time: pingStart } }));
        }
    }, PING_INTERVAL_MS);
}

function stopPing() {
    if (pingInterval) {
        clearInterval(pingInterval);
        pingInterval = null;
    }
    pingStart = null;
    lastLatency = null;
}

function updateStatusDisplay(latency) {
    const statusEl = document.getElementById('connection-status');
    if (statusEl) {
        if (latency !== null) {
            statusEl.textContent = `Connected (${latency}ms)`;
        } else if (connectionStatus === 'connected' || connectionStatus === 'initialized') {
            statusEl.textContent = 'Connected';
        } else {
            statusEl.textContent = 'Disconnected';
        }
    }
}

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
            updateStatusDisplay(null);
        } else if (msg.method === 'render') {
            const bufferView = msg.data;
            updateEditor(bufferView);
        } else if (msg.method === 'pong') {
            if (pingStart) {
                lastLatency = Date.now() - pingStart;
                updateStatusDisplay(lastLatency);
                pingStart = null;
            }
        } else if (msg.method === 'transcription') {
            const transcriptionEl = document.getElementById('transcription');
            if (transcriptionEl) {
                transcriptionEl.textContent = msg.data;
            } else {
                const newTranscriptionEl = document.createElement('div');
                newTranscriptionEl.id = 'transcription';
                newTranscriptionEl.style.cssText = 'position: fixed; bottom: 10px; left: 10px; right: 10px; padding: 8px; background: rgba(0, 0, 0, 0.8); color: white; font-family: monospace; font-size: 14px; border-radius: 4px; z-index: 1000;';
                newTranscriptionEl.textContent = msg.data;
                document.body.appendChild(newTranscriptionEl);
                setTimeout(() => {
                    newTranscriptionEl.remove();
                }, 5000);
            }
        }
    } catch (e) {
        console.error('Failed to parse message:', e);
    }
});

socket.addEventListener('close', () => {
    console.log('Disconnected from WebSocket');
    connectionStatus = 'disconnected';
    wheelDeltaAccumulator = 0;
    stopPing();
    updateStatusDisplay(null);
});
