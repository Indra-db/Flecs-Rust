use wasm_bindgen::prelude::*;
use web_sys;
use wgpu;

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

#[wasm_bindgen]
pub struct TriangleRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (u32, u32),
    render_pipeline: wgpu::RenderPipeline,
    position_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[wasm_bindgen]
impl TriangleRenderer {
    #[wasm_bindgen(constructor)]
    pub async fn new(
        canvas: web_sys::HtmlCanvasElement,
    ) -> Result<TriangleRenderer, wasm_bindgen::JsValue> {
        console_log!("Creating ECS TriangleRenderer...");

        let size = (canvas.width(), canvas.height());
        console_log!("Canvas size: {}x{}", size.0, size.1);

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        console_log!("Created wgpu instance");

        // Create surface from canvas
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

        // Request adapter
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

        // Request device and queue
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

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

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

        // Create position buffer for ECS Position component
        let position_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Position Buffer"),
            size: 8, // 2 f32s (x, y)
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout for position uniform
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Position Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Position Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: position_buffer.as_entire_binding(),
            }],
        });

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ECS Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("triangle_shader.wgsl").into()),
        });

        console_log!("Created shader module");

        // Create render pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ECS Triangle Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ECS Triangle Render Pipeline"),
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
            position_buffer,
            bind_group,
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
    pub fn update_position(&mut self, x: f32, y: f32) {
        // Convert ECS position to normalized device coordinates
        // Map from ECS coordinates to NDC (-1.0 to 1.0)
        let position_data = [x * 0.01, y * 0.01]; // Scale down the position

        self.queue.write_buffer(
            &self.position_buffer,
            0,
            bytemuck::cast_slice(&position_data),
        );
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
                label: Some("ECS Triangle Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ECS Triangle Render Pass"),
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
            render_pass.set_bind_group(0, &self.bind_group, &[]);
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
