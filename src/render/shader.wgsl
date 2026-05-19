@group(0) @binding(0) var screen: texture_storage_2d<rgba16float, write>;

struct Config {
    pan: vec2<f32>,
    zoom: f32,
    _pad: u32,
    image_size: vec2<f32>,
    _pad2: vec2<f32>,
}

@group(1) @binding(0) var image_texture: texture_2d<f32>;
@group(1) @binding(1) var image_sampler: sampler;
@group(1) @binding(2) var<uniform> config: Config;

@compute @workgroup_size(8, 8, 1)
fn compute_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // each global_id comes from compute_pass.dispatch_workgroups() in Rust
    // each global_id.x and global_id.y should yield a pixel on the texture/surface

    let screen_dims = vec2<f32>(textureDimensions(screen));

    if global_id.x >= u32(screen_dims.x) || global_id.y >= u32(screen_dims.y) {
        return;
    }

    let screen_pos = vec2<f32>(global_id.xy);
    let screen_center = screen_dims * 0.5;
    let image_center = config.image_size * 0.5;

    // Determine target location to grab using uniform layout
    let image_pos = (screen_pos - screen_center - config.pan) / config.zoom + image_center;

    // Bounds check to show a neutral dark gray background if zoomed out
    if image_pos.x < 0.0 || image_pos.y < 0.0 || image_pos.x >= config.image_size.x || image_pos.y >= config.image_size.y {
        textureStore(screen, global_id.xy, vec4<f32>(0.1, 0.1, 0.1, 1.0));
        return;
    }

    let uv = image_pos / config.image_size;
    let color = textureSampleLevel(image_texture, image_sampler, uv, 0.0);

    textureStore(screen, global_id.xy, color);
}

@vertex
fn vs_main(@builtin(vertex_index) vert_index: u32) -> @builtin(position) vec4<f32> {
    let pos = array(
        vec2<f32>(-1.0, -1.0), // clip space range [-1, 1] so extending to 3 stretches the triangle to cover the clip space
        vec2<f32>(3.0, -1.0), // https://webgpufundamentals.org/webgpu/lessons/webgpu-large-triangle-to-cover-clip-space.html
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(pos[vert_index], 0.0, 1.0);
}

@group(0) @binding(0) var output_texture: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) frag_position: vec4<f32>) -> @location(0) vec4<f32> {
    let dims_u = textureDimensions(output_texture);
    let dims = vec2<i32>(dims_u);
    let pixel = clamp(vec2<i32>(frag_position.xy), vec2<i32>(0, 0), dims - vec2<i32>(1, 1));
    let color = textureLoad(output_texture, pixel, 0);

    // sRGB gamma conversion
    let srgb_color = select(
        color.rgb * 12.92,
        pow(color.rgb, vec3<f32>(1.0 / 2.4)) * 1.055 - 0.055,
        color.rgb > vec3<f32>(0.0031308)
    );
    return vec4<f32>(srgb_color, color.a);
}