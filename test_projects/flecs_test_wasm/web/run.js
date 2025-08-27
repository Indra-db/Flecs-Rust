const fs = require('fs');
const path = require('path');

async function runWasm() {
    try {
        // Load the WASM file directly without wasm-bindgen bindings
        const wasmBuffer = fs.readFileSync('../../target/wasm32-unknown-unknown/debug/flecs_test_wasm.wasm');
        
        // Provide imports that the WASM module requires
        const imports = {
            env: {
                backtrace: () => 0,
                backtrace_symbols: () => 0,
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
                proc_exit: () => 0,
                random_get: () => 0,
            }
        };
        
        // Instantiate the WASM module with all required imports
        const wasmModule = await WebAssembly.instantiate(wasmBuffer, imports);
        
        console.log('WASM module loaded successfully (with libc.a)!');
        console.log('Available exports:', Object.keys(wasmModule.instance.exports));
        
        // Test the example_pos_x function
        console.log('\n--- Testing example_pos_x() function ---');
        try {
            const posXResult = wasmModule.instance.exports.example_pos_x();
            console.log(`example_pos_x() returned: ${posXResult}`);
        } catch (error) {
            console.error('Error calling example_pos_x():', error);
        }

    } catch (error) {
        console.error('Error loading WASM module:', error);
        console.error('Make sure to run `cargo make build-wasm` first to generate the WASM file');
        process.exit(1);
    }
}

// Run the WASM module
runWasm();
