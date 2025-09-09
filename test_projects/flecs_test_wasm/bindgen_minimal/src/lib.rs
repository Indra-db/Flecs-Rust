use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser's console API
//#[wasm_bindgen]
extern "C" {
    //#[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Export the exact same functions as our real Flecs implementation
//#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

//#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

//#[wasm_bindgen]
pub fn run_flecs_bindgen_test() {
    console_log!("Minimal stub - will be replaced with real Flecs implementation");
}
