const fs = require('fs');

async function testWasm() {
    try {
        // Read the WASM file
        const wasmBuffer = fs.readFileSync('./test_wasm.wasm');
        
        // Provide WASI imports that the libc.a requires
        const wasiImports = {
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
        
        // Instantiate the WASM module with WASI imports
        const wasmModule = await WebAssembly.instantiate(wasmBuffer, wasiImports);
        
        const { test_string_length, get_string_length, test_malloc_free, test_malloc_string_copy } = wasmModule.instance.exports;
        
        // Test the built-in test function
        const testResult = test_string_length();
        
        console.log('=== WASM String Length Test (using libc.a strlen) ===');
        console.log(`Test string length result: ${testResult}`);
        console.log(`Expected: 12 (length of "Hello, WASM!")`);
        console.log(`Status: ${testResult === 12 ? '✅ SUCCESS' : '❌ FAILED'}`);
        
        // Test malloc/free functionality
        console.log('\n=== WASM Malloc/Free Test ===');
        const mallocResult = test_malloc_free();
        console.log(`Malloc/Free test result: ${mallocResult}`);
        console.log(`Expected: 1 (success)`);
        console.log(`Status: ${mallocResult === 1 ? '✅ SUCCESS' : '❌ FAILED'}`);
        
        // Test malloc with string copying
        console.log('\n=== WASM Malloc String Copy Test ===');
        const stringCopyResult = test_malloc_string_copy();
        console.log(`String copy test result: ${stringCopyResult}`);
        console.log(`Expected: 18 (length of "Hello from malloc!")`);
        console.log(`Status: ${stringCopyResult === 18 ? '✅ SUCCESS' : '❌ FAILED'}`);
        
        // Test with custom strings would require memory management
        console.log('\n=== WASM Module Info ===');
        console.log('Exported functions:', Object.keys(wasmModule.instance.exports));
        
    } catch (error) {
        console.error('Error loading WASM:', error);
    }
}

testWasm();
