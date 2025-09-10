/**
 * Flecs ECS + WebAssembly Demo
 * 
 * This demo shows how to use Flecs ECS with wasm-bindgen and wasm32-unknown-unknown.
 * It demonstrates basic ECS operations: creating worlds, entities, components, and systems.
 */

// Global state
let worldState = null;
let progressCount = 0;
let autoProgressInterval = null;
let isAutoProgressing = false;
let triangleRenderer = null;

// DOM elements
const elements = {
    createBtn: null,
    progressBtn: null,
    positionBtn: null,
    destroyBtn: null,
    autoProgressBtn: null,
    triangleCanvas: null,
    wasmStatus: null,
    worldStatus: null,
    positionDisplay: null,
    progressCount: null,
    logContainer: null
};

/**
 * Initialize the application when the page loads
 */
document.addEventListener('DOMContentLoaded', async () => {
    // Get DOM element references
    elements.createBtn = document.getElementById('createBtn');
    elements.progressBtn = document.getElementById('progressBtn');
    elements.positionBtn = document.getElementById('positionBtn');
    elements.destroyBtn = document.getElementById('destroyBtn');
    elements.autoProgressBtn = document.getElementById('autoProgressBtn');
    elements.triangleCanvas = document.getElementById('triangleCanvas');
    elements.wasmStatus = document.getElementById('wasmStatus');
    elements.worldStatus = document.getElementById('worldStatus');
    elements.positionDisplay = document.getElementById('positionDisplay');
    elements.progressCount = document.getElementById('progressCount');
    elements.logContainer = document.getElementById('logContainer');

    // Load the WASM module
    await loadWasmModule();
});

/**
 * Load and initialize the WebAssembly module
 */
async function loadWasmModule() {
    try {
        log('Loading WASM module...', 'info');
        
        // Import the wasm-bindgen generated module
        const { default: init, WorldState, Position, TriangleRenderer } = await import('./pkg/flecs_test_wasm.js');
        
        // Initialize the WASM module
        await init();
        
        // Store classes globally for use in other functions
        window.WorldState = WorldState;
        window.Position = Position;
        window.TriangleRenderer = TriangleRenderer;
        
        log('✅ WASM module loaded successfully!', 'success');
        elements.wasmStatus.textContent = 'Ready';
        elements.createBtn.disabled = false;
        
    } catch (error) {
        log(`❌ Failed to load WASM module: ${error.message}`, 'error');
        elements.wasmStatus.textContent = 'Failed';
    }
}

/**
 * Create a new Flecs world with a simple ECS setup
 */
async function createWorld() {
    try {
        log('Creating new Flecs world...', 'info');
        
        // Create a new world instance
        worldState = new window.WorldState();
        progressCount = 0;
        
        // Initialize renderer automatically
        await initRenderer();
        
        // Update UI state
        updateUI();
        
        // Get initial position
        const position = worldState.get_position();
        log(`✅ World created! Initial position: (${position.x}, ${position.y})`, 'success');
        
        // Start auto-rendering if renderer is ready
        if (triangleRenderer) {
            startAutoRender();
        }
        
    } catch (error) {
        log(`❌ Error creating world: ${error.message}`, 'error');
    }
}

/**
 * Progress the world simulation by one frame
 */
function progressWorld() {
    if (!worldState) {
        log('❌ No world exists', 'error');
        return;
    }

    try {
        // Progress the simulation
        worldState.progress();
        progressCount++;
        
        // Get updated position
        const position = worldState.get_position();
        
        updateUI();
        
    } catch (error) {
        log(`❌ Error progressing world: ${error.message}`, 'error');
    }
}

/**
 * Get the current position of the entity
 */
function getPosition() {
    if (!worldState) {
        log('❌ No world exists', 'error');
        return;
    }

    try {
        const position = worldState.get_position();
        log(`📍 Current position: (${position.x}, ${position.y})`, 'info');
        updateUI();
        
    } catch (error) {
        log(`❌ Error getting position: ${error.message}`, 'error');
    }
}

/**
 * Destroy the current world and clean up resources
 */
function destroyWorld() {
    if (!worldState) {
        log('❌ No world exists', 'error');
        return;
    }

    try {
        log('Destroying world...', 'info');
        
        // Stop auto progress if running
        if (isAutoProgressing) {
            toggleAutoProgress();
        }
        
        // Clean up renderer
        if (triangleRenderer) {
            triangleRenderer.free();
            triangleRenderer = null;
            log('🎨 Renderer cleaned up', 'info');
        }
        
        // Free the world resources
        worldState.free();
        worldState = null;
        progressCount = 0;
        
        log('✅ World destroyed successfully!', 'success');
        updateUI();
        
    } catch (error) {
        log(`❌ Error destroying world: ${error.message}`, 'error');
    }
}

/**
 * Toggle automatic world progression
 */
function toggleAutoProgress() {
    if (!worldState) {
        log('❌ No world exists', 'error');
        return;
    }

    if (isAutoProgressing) {
        // Stop auto progress
        clearInterval(autoProgressInterval);
        autoProgressInterval = null;
        isAutoProgressing = false;
        elements.autoProgressBtn.textContent = 'Auto Progress';
        elements.progressBtn.disabled = false;
        log('⏸️ Auto progress stopped', 'info');
    } else {
        // Start auto progress
        autoProgressInterval = setInterval(() => {
            progressWorld();
        }, 50); // Progress every 50ms
        isAutoProgressing = true;
        elements.autoProgressBtn.textContent = 'Stop Auto';
        elements.progressBtn.disabled = true;
        log('▶️ Auto progress started (50ms interval)', 'success');
    }
    
    updateUI();
}

/**
 * Update the UI to reflect current state
 */
function updateUI() {
    const hasWorld = worldState !== null;
    
    // Update button states
    elements.createBtn.disabled = hasWorld;
    elements.progressBtn.disabled = !hasWorld || isAutoProgressing;
    elements.positionBtn.disabled = !hasWorld;
    elements.destroyBtn.disabled = !hasWorld;
    elements.autoProgressBtn.disabled = !hasWorld;
    
    // Update status display
    elements.worldStatus.textContent = hasWorld ? 'Active' : 'None';
    elements.progressCount.textContent = progressCount;
    
    // Update position display
    if (hasWorld) {
        try {
            const position = worldState.get_position();
            elements.positionDisplay.textContent = `(${position.x}, ${position.y})`;
        } catch (error) {
            elements.positionDisplay.textContent = 'Error';
        }
    } else {
        elements.positionDisplay.textContent = '--';
    }
}

/**
 * Add a log entry to the activity log
 * @param {string} message - The message to log
 * @param {string} type - The type of message ('info', 'success', 'error', 'warning')
 */
function log(message, type = 'info') {
    const timestamp = new Date().toLocaleTimeString();
    const logEntry = document.createElement('div');
    logEntry.className = `log-entry log-${type}`;
    
    logEntry.innerHTML = `
        <span class="log-timestamp">[${timestamp}]</span>
        <span class="log-message">${message}</span>
    `;
    
    elements.logContainer.appendChild(logEntry);
    elements.logContainer.scrollTop = elements.logContainer.scrollHeight;
    
    // Keep only the last 50 log entries
    while (elements.logContainer.children.length > 50) {
        elements.logContainer.removeChild(elements.logContainer.firstChild);
    }
}

/**
 * Initialize the triangle renderer
 */
async function initRenderer() {
    try {
        log('Initializing triangle renderer...', 'info');
        
        if (!window.TriangleRenderer) {
            log('❌ TriangleRenderer not available', 'error');
            return;
        }
        
        triangleRenderer = await new window.TriangleRenderer(elements.triangleCanvas);
        
        log('✅ Triangle renderer initialized!', 'success');
        elements.renderBtn.disabled = false;
        
        // Start auto-rendering if we have a world
        if (worldState) {
            startAutoRender();
        }
        
    } catch (error) {
        log(`❌ Failed to initialize renderer: ${error.message}`, 'error');
    }
}

/**
 * Render a single frame of the triangle
 */
function renderTriangle() {
    if (!triangleRenderer) {
        log('❌ Renderer not initialized', 'error');
        return;
    }
    
    try {
        // Update triangle position from ECS if world exists
        if (worldState) {
            const position = worldState.get_position();
            triangleRenderer.update_position(position.x, position.y);
        }
        
        triangleRenderer.render();
        
    } catch (error) {
        log(`❌ Error rendering triangle: ${error.message}`, 'error');
    }
}

/**
 * Start automatic rendering loop
 */
function startAutoRender() {
    if (triangleRenderer && worldState) {
        // Render every frame
        const renderLoop = () => {
            if (triangleRenderer && worldState) {
                renderTriangle();
                requestAnimationFrame(renderLoop);
            }
        };
        requestAnimationFrame(renderLoop);
        log('🎬 Auto-rendering started!', 'info');
    }
}
