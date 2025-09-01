const fs = require('fs');
const path = require('path');

// Enable detailed error reporting
process.on('unhandledRejection', (reason, promise) => {
    console.log('Unhandled Rejection at:', promise, 'reason:', reason);
});

async function runWasm() {
    try {
        // Load the WASM file directly without wasm-bindgen bindings
        const wasmBuffer = fs.readFileSync('../../target/wasm32-unknown-unknown/debug/flecs_test_wasm.wasm');
        
        // We need to create a reference that will be set after instantiation
        let wasmInstance = null;
        
        // Add console logging functions for WASM debugging
        const consoleImports = {
            console_log: (ptr, len) => {
                const view = new Uint8Array(wasmInstance.exports.memory.buffer, ptr, len);
                const str = new TextDecoder().decode(view);
                console.log('[WASM LOG]:', str);
            },
            console_error: (ptr, len) => {
                const view = new Uint8Array(wasmInstance.exports.memory.buffer, ptr, len);
                const str = new TextDecoder().decode(view);
                console.error('[WASM ERROR]:', str);
            },
            debug_trace: (value) => {
                console.log('[WASM TRACE]:', value);
            }
        };
        
        // Provide imports that the WASM module requires
        const imports = {
            env: {
                backtrace: () => 0,
                backtrace_symbols: () => 0,
                ...consoleImports,
            },
            wasi_snapshot_preview1: {
                environ_get: () => 0,
                environ_sizes_get: () => 0,
                fd_close: () => 0,
                fd_fdstat_get: () => 0,
                fd_prestat_get: () => 0,
                fd_prestat_dir_name: () => 0,
                fd_read: () => 0,
                fd_seek: () => 0,
                fd_write: () => 0,
                clock_time_get: (clock_id, precision, time_ptr) => {
                    // Return a simple timestamp in nanoseconds
                    // For WASM, we'll use performance.now() * 1000000 (convert ms to ns)
                    const timeNs = BigInt(Math.floor(performance.now() * 1000000));
                    const memory = new DataView(wasmInstance.exports.memory.buffer);
                    memory.setBigUint64(time_ptr, timeNs, true); // little-endian
                    return 0;
                },
                proc_exit: (code) => {
                    console.log('[WASM] Process exit with code:', code);
                    return 0;
                },
                random_get: () => 0,
            }
        };
        
        // Instantiate the WASM module with all required imports
        const wasmModule = await WebAssembly.instantiate(wasmBuffer, imports);
        wasmInstance = wasmModule.instance; // Set the reference
        
        console.log('\nWASM module loaded successfully!');
        //console.log('Available exports:', Object.keys(wasmInstance.exports));
    
        // Test the new world management functions
        console.log('\n--- Testing world management functions ---');
        try {
            // Create a new world
            console.log('Creating world...');
            const worldPtr = wasmInstance.exports.create_world();
            
            // Get initial position
            const initialPos = wasmInstance.exports.get_pos_x(worldPtr);
            console.log(`Initial position x: ${initialPos}`);
            
            // Progress the world twice and print positions
            console.log('Progressing world first time...');
            wasmInstance.exports.progress_world_ptr(worldPtr);
            const pos1 = wasmInstance.exports.get_pos_x(worldPtr);
            console.log(`Position x after 1st progress: ${pos1}`);
            
            console.log('Progressing world second time...');
            wasmInstance.exports.progress_world_ptr(worldPtr);
            const pos2 = wasmInstance.exports.get_pos_x(worldPtr);
            console.log(`Position x after 2nd progress: ${pos2}`);
            
            // Clean up
            console.log('Destroying world...');
            wasmInstance.exports.destroy_world(worldPtr);
            console.log('World destroyed successfully\n');
            
        } catch (error) {
            console.error('Error testing world functions:', error);
            console.error('Error name:', error.name);
            console.error('Error message:', error.message);
            console.error('Error stack:', error.stack);
            
            // Try to extract more WASM-specific information
            if (error instanceof WebAssembly.RuntimeError) {
                console.error('This is a WebAssembly RuntimeError');
                console.error('Error toString:', error.toString());
            }
        }
    } catch (error) {
        console.error('Error loading WASM module:', error);
        console.error('Make sure to run `cargo make build-wasm` first to generate the WASM file');
        process.exit(1);
    }
}

// Run the WASM module
runWasm();
