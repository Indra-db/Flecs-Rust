// External declaration of C's strlen function
unsafe extern "C" {
    fn strlen(s: *const i8) -> usize;
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

// Function that calls C's strlen
extern "C" fn calls_strlen(s: *const i8) -> usize {
    unsafe { strlen(s) }
}

// Proper Rust function that calls calls_strlen - exported directly for WASM
#[unsafe(no_mangle)]
pub extern "C" fn get_string_length(s: *const i8) -> usize {
    calls_strlen(s)
}

// Test function that creates a test string and calls get_string_length
#[unsafe(no_mangle)]
pub extern "C" fn test_string_length() -> usize {
    let test_str = b"Hello, WASM!\0";
    get_string_length(test_str.as_ptr() as *const i8)
}

// Test malloc/free functionality
#[unsafe(no_mangle)]
pub extern "C" fn test_malloc_free() -> i32 {
    unsafe {
        // Allocate 100 bytes
        let ptr = malloc(100);

        // Check if allocation succeeded
        if ptr.is_null() {
            return -1; // Failed to allocate
        }

        // Write some data to the allocated memory
        *ptr = 42;
        *(ptr.add(99)) = 84; // Write to the last byte

        // Read back the data to verify it works
        let first_byte = *ptr;
        let last_byte = *(ptr.add(99));

        // Free the memory
        free(ptr);

        // Return success if the values match what we wrote
        if first_byte == 42 && last_byte == 84 {
            1 // Success
        } else {
            0 // Data corruption
        }
    }
}

// Test malloc with string copying
#[unsafe(no_mangle)]
pub extern "C" fn test_malloc_string_copy() -> usize {
    unsafe {
        let source_str = b"Hello from malloc!\0";
        let str_len = strlen(source_str.as_ptr() as *const i8);

        // Allocate memory for the string + null terminator
        let allocated_str = malloc(str_len + 1) as *mut i8;

        if allocated_str.is_null() {
            return 0; // Failed to allocate
        }

        // Copy the string byte by byte
        for i in 0..=str_len {
            *allocated_str.add(i) = *(source_str.as_ptr() as *const i8).add(i);
        }

        // Get the length of the copied string
        let copied_len = strlen(allocated_str);

        // Free the allocated memory
        free(allocated_str as *mut u8);

        // Return the length of the copied string
        copied_len
    }
}
