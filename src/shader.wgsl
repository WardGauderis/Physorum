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

struct Config {
    num_agents:             u32,

	move_speed:             f32,
	turn_speed:             f32,
	sensor_angle:           f32,
	sensor_offset: f32,
	sensor_width:           i32,

	diffuse_rate:           f32,
	decay_rate:             f32,

    time:                   f32,
	delta_time:             f32,
}
var<push_constant> config: Config;

fn modulo(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return ((a % b) + b) % b;
}

@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    let pos = vec2<i32>(in.xy);
    let dimensions = textureDimensions(texture);

	let original = textureLoad(texture, pos, 0);

	var diffuse = vec4(0.0);
	for (var offset_x = -1; offset_x <= 1; offset_x++) {
		for (var offset_y = -1; offset_y <= 1; offset_y++) {
//		    let pos = clamp(in.xy + vec2(f32(offset_x), f32(offset_y)), vec2(0.0), vec2(dimensions.x - 1.0, dimensions.y - 1.0));
			diffuse = diffuse + textureLoad(texture, modulo(pos + vec2(offset_x, offset_y), dimensions), 0);
		}
	}
	diffuse = diffuse / 9.0;

	let weight = clamp(config.diffuse_rate * config.delta_time, 0.0, 1.0);
	diffuse = mix(original, diffuse, weight);

	diffuse = max(vec4(0.0), diffuse - config.decay_rate * config.delta_time);

    return diffuse;
}
