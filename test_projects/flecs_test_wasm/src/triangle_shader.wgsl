// Vertex shader for a movable triangle
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

// Uniform buffer for position from ECS
@group(0) @binding(0)
var<uniform> position: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Create a simple triangle with explicit coordinates
    var pos = array<vec2<f32>, 3>(
        vec2<f32>( 0.0,  0.1),  // Top vertex
        vec2<f32>(-0.05, -0.1),  // Bottom left
        vec2<f32>( 0.05, -0.1)   // Bottom right
    );
    
    var out: VertexOutput;
    // Apply the position offset from ECS uniform buffer
    out.clip_position = vec4<f32>(pos[in_vertex_index] + position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Return bright green color for ECS triangle
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
