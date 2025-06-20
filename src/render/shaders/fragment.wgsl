// Fragment shader for basic rendering

@fragment
fn fs_main(
    @location(0) tex_coords: vec2<f32>
) -> @location(0) vec4<f32> {
    // Simple background color
    return vec4<f32>(0.1, 0.1, 0.1, 0.9);
}
