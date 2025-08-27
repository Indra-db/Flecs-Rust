const fs = require('fs');
const path = require('path');
const Module = require('module');

// Override Module resolution to handle 'env' module
const originalResolveFilename = Module._resolveFilename;
Module._resolveFilename = function(request, parent, isMain) {
    if (request === 'env') {
        // Return a dummy module path for 'env'
        return path.join(__dirname, 'pkg', 'env.js');
    }
    return originalResolveFilename.call(this, request, parent, isMain);
};

// Create a mock 'env' module to satisfy the import
const envPath = path.join(__dirname, 'pkg', 'env.js');
if (!fs.existsSync(envPath)) {
    fs.writeFileSync(envPath, 'module.exports = {};');
}

async function runWasm() {
    try {
        // Use the wasm-bindgen generated bindings
        const wasmModule = require('./pkg/flecs_test_wasm.js');
        
        console.log('WASM module loaded successfully via wasm-bindgen!');
        console.log('Available exports:', Object.keys(wasmModule));
        
        // Test the hello function first
        console.log('\n--- Testing hello() function ---');
        try {
            const helloResult = wasmModule.hello();
            console.log(`hello() returned: ${helloResult}`);
        } catch (error) {
            console.error('Error calling hello():', error);
        }

        // Test the example_pos_x function
        console.log('\n--- Testing example_pos_x() function ---');
        try {
            const posXResult = wasmModule.example_pos_x();
            console.log(`example_pos_x() returned: ${posXResult}`);
        } catch (error) {
            console.error('Error calling example_pos_x():', error);
        }

    } catch (error) {
        console.error('Error loading WASM module:', error);
        console.error('Make sure to run `cargo make build-wasm` first to generate the bindings');
        process.exit(1);
    }
}

// Run the WASM module
runWasm();
