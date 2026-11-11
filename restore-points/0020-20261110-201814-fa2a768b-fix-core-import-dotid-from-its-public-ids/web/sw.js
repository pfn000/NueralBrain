// service-worker.js — Keeps the brain dreaming when tab is closed

let brainState = null;
let dreamInterval = null;

self.addEventListener('install', (event) => {
    self.skipWaiting();
    console.log('[BioNet SW] Installed — Dream engine ready');
});

self.addEventListener('activate', (event) => {
    console.log('[BioNet SW] Activated — Background processing enabled');
    startDreaming();
});

self.addEventListener('message', (event) => {
    const { type, data } = event.data;
    
    switch (type) {
        case 'SAVE_STATE':
            brainState = data;
            saveToDB(data);
            break;
            
        case 'WAKE_UP':
            stopDreaming();
            break;
            
        case 'SLEEP':
            startDreaming();
            break;
            
        case 'GET_STATE':
            event.ports[0].postMessage({ 
                type: 'STATE', 
                data: brainState 
            });
            break;
    }
});

function startDreaming() {
    if (dreamInterval) return;
    
    dreamInterval = setInterval(async () => {
        if (!brainState) {
            brainState = await loadFromDB();
        }
        
        if (brainState) {
            // Run dream tick
            // In production, this would call into a lightweight
            // JS implementation of the dream engine, or a 
            // separate WASM module
            processDreamTick(brainState);
            
            // Persist updated state
            await saveToDB(brainState);
        }
    }, 5000); // Dream tick every 5 seconds (slow processing)
    
    console.log('[BioNet SW] Dreaming started...');
}

function stopDreaming() {
    if (dreamInterval) {
        clearInterval(dreamInterval);
        dreamInterval = null;
    }
    console.log('[BioNet SW] Waking up...');
}

function processDreamTick(state) {
    // Lightweight dream processing
    // - Decay unused connection strengths
    // - Strengthen frequently-used pathways
    // - Consolidate high-value memories
    // - Reduce fatigue across all dots
    
    console.log('[BioNet SW] Dream tick processed');
}

async function saveToDB(data) {
    // Save to IndexedDB for persistence across sessions
    const db = await openDB();
    const tx = db.transaction('brainState', 'readwrite');
    tx.objectStore('brainState').put(data, 'current');
}

async function loadFromDB() {
    const db = await openDB();
    const tx = db.transaction('brainState', 'readonly');
    return tx.objectStore('brainState').get('current');
}

function openDB() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open('BioneticBrain', 1);
        request.onupgradeneeded = () => {
            request.result.createObjectStore('brainState');
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
    });
}
