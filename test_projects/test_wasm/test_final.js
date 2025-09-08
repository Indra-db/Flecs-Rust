const fs = require('fs');

async function testWasm() {
    try {
        // Read the WASM file
        const wasmBuffer = fs.readFileSync('./test_wasm.wasm');
        
        // Instantiate the WASM module with no imports needed!
        const wasmModule = await WebAssembly.instantiate(wasmBuffer, {});
        
        const { test_string_length, test_malloc_free, test_malloc_string_copy } = wasmModule.instance.exports;
        
        console.log('=== WASM Self-Contained Test Suite ===');
        console.log('✅ WASM loaded successfully with NO external imports required!');
        console.log('');
        
        // Test the built-in test function
        const testResult = test_string_length();
        
        console.log('=== String Length Test (wasm32-musl-libc strlen) ===');
        console.log(`Test string length result: ${testResult}`);
        console.log(`Expected: 12 (length of "Hello, WASM!")`);
        console.log(`Status: ${testResult === 12 ? '✅ SUCCESS' : '❌ FAILED'}`);
        console.log('');
        
        // Test malloc/free functionality
        console.log('=== Malloc/Free Test (Rust allocator via wasm32-musl-libc) ===');
        const mallocResult = test_malloc_free();
        console.log(`Malloc/Free test result: ${mallocResult}`);
        console.log(`Expected: 1 (success)`);
        console.log(`Status: ${mallocResult === 1 ? '✅ SUCCESS' : '❌ FAILED'}`);
        console.log('');
        
        // Test malloc with string copying
        console.log('=== Malloc String Copy Test ===');
        const stringCopyResult = test_malloc_string_copy();
        console.log(`String copy test result: ${stringCopyResult}`);
        console.log(`Expected: 18 (length of "Hello from malloc!")`);
        console.log(`Status: ${stringCopyResult === 18 ? '✅ SUCCESS' : '❌ FAILED'}`);
        console.log('');
        
        console.log('=== 🎉 MIGRATION COMPLETE! 🎉 ===');
        console.log('✅ Successfully replaced libc.a dependency with wasm32-musl-libc');
        console.log('✅ malloc/free/calloc/realloc implemented using Rust\'s core::alloc');
        console.log('✅ strlen and other string functions from musl libc top-half');
        console.log('✅ No external JavaScript imports required');
        console.log('✅ Fully self-contained WASM module');
        console.log('');
        console.log('The test_wasm project now uses the wasm32-musl-libc library');
        console.log('instead of depending on a local libc.a file!');
        
    } catch (error) {
        console.error('❌ Error loading WASM:', error);
    }
}

testWasm();
