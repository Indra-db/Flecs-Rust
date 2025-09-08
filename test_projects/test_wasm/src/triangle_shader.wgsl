@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Create a simple triangle with explicit coordinates
    var pos = array<vec2<f32>, 3>(
        vec2<f32>( 0.0,  0.5),  // Top vertex
        vec2<f32>(-0.5, -0.5),  // Bottom left
        vec2<f32>( 0.5, -0.5)   // Bottom right
    );
    return vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // Return bright red color
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
