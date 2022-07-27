@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

struct Agent {
    position: vec2<f32>,
    angle: f32,
};
@group(0) @binding(1)
var<storage, read_write> agents: array<Agent>;

struct Config {
    num_agents:             u32,

	move_speed:             f32,
	turn_speed:             f32,
	sensor_angle:           f32,
	sensor_offset_distance: f32,
	sensor_width:           i32,

	diffuse_rate:           f32,
	decay_rate:             f32,

    time:                   f32,
	delta_time:             f32,
}
var<push_constant> config: Config;

fn hash(state: u32) -> u32
{
    var s = state ^ 2747636419u;
    s = s * 2654435769u;
    s = s ^ (s >> 16u);
    s = s * 2654435769u;
    s = s ^ (s >> 16u);
    s = s * 2654435769u;
    return s;
}

fn scale(state: u32) -> f32 {
    return f32(state) / 4294967295.0;
}

fn sense(position: vec2<f32>, angle: f32, sensor_angle: f32, config: Config, dimensions: vec2<f32>) -> f32 {
    let sensor_angle = angle + sensor_angle;
    let sensor_direction = vec2<f32>(cos(sensor_angle), sin(sensor_angle));
    let sensor_center = position + sensor_direction * config.sensor_offset_distance;

    var sum = 0.0;
    for (var offset_x = - i32(config.sensor_width); offset_x <= config.sensor_width; offset_x++) {
        for (var offset_y = - i32(config.sensor_width); offset_y <= config.sensor_width; offset_y++) {
		    let pos = clamp(sensor_center + vec2(f32(offset_x), f32(offset_y)), vec2(0.0), vec2(dimensions.x - 1.0, dimensions.y - 1.0));
		    //let pos = sensor_center + vec2(f32(offset_x), f32(offset_y));
			sum = sum + textureLoad(texture, vec2<i32>(pos)).r;
        }
    }

    return sum;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dimensions = vec2<f32>(textureDimensions(texture));

    if (id.x >= config.num_agents) {
      return;
    }

    var position = agents[id.x].position;
    var angle = agents[id.x].angle;

    let random = hash(u32(position.y * dimensions.x + position.x) + hash(id.x + u32(config.time * 100000.0)));

	let f = sense(position, angle, 0.0, config, dimensions);
	let fl = sense(position, angle, config.sensor_angle, config, dimensions);
	let fr = sense(position, angle, -config.sensor_angle, config, dimensions);

	let steer_strength = scale(random);

    if (f > fl && f > fr) {}
	else if (f < fl && f < fr) {
		angle = angle + (steer_strength - 0.5) * 2.0 * config.turn_speed * config.delta_time;
	}
	else if (fr > fl) {
		angle = angle - steer_strength * config.turn_speed * config.delta_time;
	}
	else if (fl > fr) {
		angle = angle + steer_strength * config.turn_speed * config.delta_time;
	}

    let direction = vec2(cos(angle), sin(angle));
    position = position + direction * config.delta_time * config.move_speed;

    if (position.x < 0.0 || position.x >= dimensions.x || position.y < 0.0 || position.y >= dimensions.y) {
		let random = hash(random);
		position = clamp(position, vec2(0.0, 0.0), vec2(dimensions.x - 1.0, dimensions.y - 1.0));
		angle = scale(random) * 2.0 * 3.14159;
	} else {
	    textureStore(texture, vec2<i32>(position) , vec4(1.0, 1.0, 1.0, 1.0));
	}

    agents[id.x].position = position;
	agents[id.x].angle = angle;
}
