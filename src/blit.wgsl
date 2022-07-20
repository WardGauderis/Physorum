@vertex
fn vs_main(
    @builtin(vertex_index) in: u32,
) -> @builtin(position) vec4<f32> {
    let x = -1.0 + f32((in & 1u) << 2u);
    let y = -1.0 + f32((in & 2u) << 1u);
    return vec4(x, y, 0.0, 1.0);
}

@group(0) @binding(0)
var texture: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(texture, vec2<i32>(in.xy), 0);
}