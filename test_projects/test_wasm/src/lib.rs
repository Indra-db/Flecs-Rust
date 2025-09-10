// Import functions from wasm32_musl_libc
use core::ffi::c_char;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm32_musl_libc::*;

#[cfg(not(target_arch = "wasm32"))]
use libc::*;

// Import the `console.log` function from the browser console
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Custom struct for wasm_bindgen testing
#[wasm_bindgen]
pub struct Person {
    name: String,
    age: u32,
    email: String,
}

#[wasm_bindgen]
impl Person {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, age: u32, email: String) -> Person {
        Person { name, age, email }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn age(&self) -> u32 {
        self.age
    }

    #[wasm_bindgen(getter)]
    pub fn email(&self) -> String {
        self.email.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[wasm_bindgen(setter)]
    pub fn set_age(&mut self, age: u32) {
        self.age = age;
    }

    #[wasm_bindgen(setter)]
    pub fn set_email(&mut self, email: String) {
        self.email = email;
    }

    // Method to print the person's details
    #[wasm_bindgen]
    pub fn print_details(&self) {
        console_log!(
            "Person Details:\n  Name: {}\n  Age: {}\n  Email: {}",
            self.name,
            self.age,
            self.email
        );
    }

    // Method to get formatted string representation
    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        format!(
            "Person(name: '{}', age: {}, email: '{}')",
            self.name, self.age, self.email
        )
    }

    // Static method to create a sample person
    #[wasm_bindgen]
    pub fn create_sample() -> Person {
        Person::new(
            "John Doe".to_string(),
            30,
            "john.doe@example.com".to_string(),
        )
    }
}

// wasm_bindgen function to test the Person struct
#[wasm_bindgen]
pub fn test_person_struct() -> String {
    let person = Person::create_sample();

    console_log!("=== Testing Person Struct ===");
    person.print_details();

    let person_str = person.to_string();
    console_log!("String representation: {}", person_str);

    // Create another person and modify it
    let mut person2 = Person::new(
        "Jane Smith".to_string(),
        25,
        "jane.smith@example.com".to_string(),
    );

    console_log!("=== Before modification ===");
    person2.print_details();

    person2.set_age(26);
    person2.set_email("jane.smith.updated@example.com".to_string());

    console_log!("=== After modification ===");
    person2.print_details();

    format!(
        "Test completed! Created 2 person objects: '{}' and '{}'",
        person.to_string(),
        person2.to_string()
    )
}

// Simple wasm_bindgen function for greeting
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    console_log!("Greeting function called with name: {}", name);
    format!(
        "Hello, {}! This message is from Rust via wasm_bindgen.",
        name
    )
}

// Function that calls C's strlen
extern "C" fn calls_strlen(s: *const c_char) -> usize {
    unsafe { strlen(s) as usize }
}

// Proper Rust function that calls calls_strlen - exported directly for WASM
#[unsafe(no_mangle)]
pub extern "C" fn get_string_length(s: *const c_char) -> usize {
    calls_strlen(s)
}

// Test function that creates a test string and calls get_string_length
#[wasm_bindgen]
pub fn test_string_length() -> usize {
    let test_str = b"Hello, WASM!\0";
    get_string_length(test_str.as_ptr() as *const c_char)
}

// Test malloc/free functionality
#[wasm_bindgen]
pub fn test_malloc_free() -> i32 {
    unsafe {
        // Allocate 100 bytes
        let ptr = malloc(100);

        // Check if allocation succeeded
        if ptr.is_null() {
            return -1; // Failed to allocate
        }

        // Cast to u8 pointer for byte operations
        let byte_ptr = ptr as *mut u8;

        // Write some data to the allocated memory
        *byte_ptr = 42;
        *(byte_ptr.add(99)) = 84; // Write to the last byte

        // Read back the data to verify it works
        let first_byte = *byte_ptr;
        let last_byte = *(byte_ptr.add(99));

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
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn test_malloc_string_copy() -> usize {
    unsafe {
        let source_str = b"Hello from malloc!\0";
        let str_len = strlen(source_str.as_ptr() as *const c_char) as usize;

        // Allocate memory for the string + null terminator
        let allocated_ptr = malloc(str_len + 1);

        if allocated_ptr.is_null() {
            return 0; // Failed to allocate
        }

        let allocated_str = allocated_ptr as *mut c_char;

        // Copy the string byte by byte
        for i in 0..=str_len {
            *allocated_str.add(i) = *(source_str.as_ptr() as *const c_char).add(i);
        }

        // Get the length of the copied string
        let copied_len = strlen(allocated_str) as usize;

        // Free the allocated memory
        free(allocated_ptr);

        // Return the length of the copied string
        copied_len
    }
}

// Hello Triangle WGPU Implementation

#[wasm_bindgen]
pub struct TriangleRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (u32, u32),
    render_pipeline: wgpu::RenderPipeline,
}

#[wasm_bindgen]
impl TriangleRenderer {
    #[wasm_bindgen(constructor)]
    pub async fn new(
        canvas: web_sys::HtmlCanvasElement,
    ) -> Result<TriangleRenderer, wasm_bindgen::JsValue> {
        console_log!("Creating TriangleRenderer...");

        let size = (canvas.width(), canvas.height());

        console_log!("Canvas size: {}x{}", size.0, size.1);

        // The instance is a handle to our GPU - allow both WebGL and WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU, // Allow both for compatibility
            ..Default::default()
        });

        console_log!("Created wgpu instance");

        // For WASM - use simplified surface creation
        let surface = {
            let surface_target = wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: raw_window_handle::WebDisplayHandle::new().into(),
                raw_window_handle: {
                    let canvas_ptr = canvas.as_ref() as *const web_sys::HtmlCanvasElement;
                    raw_window_handle::WebCanvasWindowHandle::new(
                        std::ptr::NonNull::new(canvas_ptr as *mut std::ffi::c_void).unwrap(),
                    )
                    .into()
                },
            };

            unsafe { instance.create_surface_unsafe(surface_target) }.map_err(|e| {
                wasm_bindgen::JsValue::from_str(&format!("Failed to create surface: {:?}", e))
            })?
        };

        console_log!("Created surface from canvas");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| {
                wasm_bindgen::JsValue::from_str(&format!("Failed to find adapter: {:?}", e))
            })?;

        console_log!("Got adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await
            .map_err(|e| {
                wasm_bindgen::JsValue::from_str(&format!("Failed to create device: {:?}", e))
            })?;

        console_log!("Created device and queue");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        console_log!("Surface format: {:?}", surface_format);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        console_log!("Configured surface");

        // Load the shaders from the included WGSL
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("triangle_shader.wgsl").into()),
        });

        console_log!("Created shader module");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        console_log!("Created render pipeline");

        Ok(TriangleRenderer {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
        })
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.size = (new_width, new_height);
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
            console_log!("Resized to {}x{}", new_width, new_height);
        }
    }

    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), wasm_bindgen::JsValue> {
        let output = self.surface.get_current_texture().map_err(|e| {
            wasm_bindgen::JsValue::from_str(&format!("Failed to get surface texture: {:?}", e))
        })?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Triangle Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Triangle Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_size(&self) -> Vec<u32> {
        vec![self.size.0, self.size.1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_creation() {
        let person = Person::new("Test User".to_string(), 25, "test@example.com".to_string());

        assert_eq!(person.name(), "Test User");
        assert_eq!(person.age(), 25);
        assert_eq!(person.email(), "test@example.com");
    }

    #[test]
    fn test_person_modification() {
        let mut person = Person::new(
            "Original Name".to_string(),
            30,
            "original@example.com".to_string(),
        );

        // Test setters
        person.set_name("Updated Name".to_string());
        person.set_age(31);
        person.set_email("updated@example.com".to_string());

        assert_eq!(person.name(), "Updated Name");
        assert_eq!(person.age(), 31);
        assert_eq!(person.email(), "updated@example.com");
    }

    #[test]
    fn test_person_to_string() {
        let person = Person::new("John".to_string(), 25, "john@test.com".to_string());

        let person_str = person.to_string();
        assert!(person_str.contains("John"));
        assert!(person_str.contains("25"));
        assert!(person_str.contains("john@test.com"));
        assert!(person_str.starts_with("Person("));
    }

    #[test]
    fn test_person_create_sample() {
        let sample = Person::create_sample();
        assert_eq!(sample.name(), "John Doe");
        assert_eq!(sample.age(), 30);
        assert_eq!(sample.email(), "john.doe@example.com");
    }

    #[test]
    fn test_greet_function() {
        let greeting = greet("Rust Test");
        assert!(greeting.contains("Hello, Rust Test!"));
        assert!(greeting.contains("wasm_bindgen"));
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_string_length() {
        // Test the wasm32_musl_libc strlen functionality directly
        let test_str = b"Hello, WASM!\0";
        let length = get_string_length(test_str.as_ptr() as *const c_char);
        // "Hello, WASM!" is 12 characters
        assert_eq!(length, 12);
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_malloc_free_functionality() {
        // Test the wasm32_musl_libc malloc/free functionality directly
        unsafe {
            // Allocate 100 bytes
            let ptr = malloc(100);

            // Check if allocation succeeded
            assert!(!ptr.is_null(), "malloc should succeed");

            // Cast to u8 pointer for byte operations
            let byte_ptr = ptr as *mut u8;

            // Write some data to the allocated memory
            *byte_ptr = 42;
            *(byte_ptr.add(99)) = 84; // Write to the last byte

            // Read back the data to verify it works
            let first_byte = *byte_ptr;
            let last_byte = *(byte_ptr.add(99));

            // Free the memory
            free(ptr);

            // Verify the values match what we wrote
            assert_eq!(first_byte, 42);
            assert_eq!(last_byte, 84);
        }
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_malloc_string_copy_functionality() {
        // Test the wasm32_musl_libc malloc with string copying directly
        unsafe {
            let source_str = b"Hello from malloc!\0";
            let str_len = strlen(source_str.as_ptr() as *const c_char) as usize;

            // Allocate memory for the string + null terminator
            let allocated_ptr = malloc(str_len + 1);

            assert!(!allocated_ptr.is_null(), "malloc should succeed");

            let allocated_str = allocated_ptr as *mut c_char;

            // Copy the string byte by byte
            for i in 0..=str_len {
                *allocated_str.add(i) = *(source_str.as_ptr() as *const c_char).add(i);
            }

            // Get the length of the copied string
            let copied_len = strlen(allocated_str) as usize;

            // Free the allocated memory
            free(allocated_ptr);

            // "Hello from malloc!" is 18 characters
            assert_eq!(copied_len, 18);
        }
    }

    #[test]
    fn test_person_struct_function() {
        // This tests the wasm_bindgen exported function behavior
        // Note: In native tests, wasm_bindgen functions may behave differently
        #[cfg(target_arch = "wasm32")]
        {
            let result = test_person_struct();
            assert!(result.contains("Test completed!"));
            assert!(result.contains("Created 2 person objects"));
            assert!(result.contains("John Doe"));
            assert!(result.contains("Jane Smith"));
        }
    }

    // Test memory safety and edge cases
    #[test]
    fn test_person_empty_strings() {
        let person = Person::new(String::new(), 0, String::new());

        assert_eq!(person.name(), "");
        assert_eq!(person.age(), 0);
        assert_eq!(person.email(), "");
    }

    #[test]
    fn test_person_unicode_strings() {
        let person = Person::new("José María".to_string(), 35, "josé@maría.com".to_string());

        assert_eq!(person.name(), "José María");
        assert_eq!(person.age(), 35);
        assert_eq!(person.email(), "josé@maría.com");
    }

    #[test]
    fn test_person_large_age() {
        let mut person = Person::new(
            "Old Person".to_string(),
            u32::MAX,
            "old@example.com".to_string(),
        );

        assert_eq!(person.age(), u32::MAX);

        person.set_age(0);
        assert_eq!(person.age(), 0);
    }

    #[test]
    fn test_greet_empty_name() {
        let greeting = greet("");
        assert!(greeting.contains("Hello, !"));
        assert!(greeting.contains("wasm_bindgen"));
    }

    #[test]
    fn test_greet_special_characters() {
        let greeting = greet("Test & <script>alert('xss')</script>");
        assert!(greeting.contains("Test & <script>alert('xss')</script>"));
        assert!(greeting.contains("wasm_bindgen"));
    }

    // Integration tests
    #[test]
    fn test_multiple_persons_independence() {
        let mut person1 = Person::new("Person 1".to_string(), 20, "p1@test.com".to_string());

        let person2 = Person::new("Person 2".to_string(), 30, "p2@test.com".to_string());

        // Modify person1
        person1.set_name("Modified Person 1".to_string());
        person1.set_age(21);

        // person2 should remain unchanged
        assert_eq!(person1.name(), "Modified Person 1");
        assert_eq!(person1.age(), 21);
        assert_eq!(person2.name(), "Person 2");
        assert_eq!(person2.age(), 30);
    }

    #[test]
    fn test_person_method_chaining_simulation() {
        let mut person = Person::create_sample();

        // Simulate method chaining by calling multiple setters
        person.set_name("Chained Name".to_string());
        person.set_age(99);
        person.set_email("chained@example.com".to_string());

        assert_eq!(person.name(), "Chained Name");
        assert_eq!(person.age(), 99);
        assert_eq!(person.email(), "chained@example.com");
    }

    // Tests for memory allocator functions
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_c_strlen_integration() {
        // This tests the integration with wasm32_musl_libc
        let test_str = b"Integration test string\0";
        let length = get_string_length(test_str.as_ptr() as *const core::ffi::c_char);
        assert_eq!(length, 23); // "Integration test string" is 23 characters
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_malloc_zero_bytes() {
        // Test edge case of zero byte allocation
        unsafe {
            let ptr = malloc(0);
            // malloc(0) behavior is implementation-defined, but should not crash
            if !ptr.is_null() {
                free(ptr);
            }
            // Test passes if we don't crash
        }
    }
}
