// The two things a frame of the output is made of, each drawn with one
// triangle that covers the whole target: the source frame, turned, cropped
// and placed in its rectangle, and over it the overlay.
//
// No colour is converted. The frame and the overlay hold sRGB-encoded values
// and so does the target.

// Where a point of the video's rectangle on the target lies in the frame's
// texture:
//   frame_uv = (m.x * u + m.z * v + t.x,  m.y * u + m.w * v + t.y)
// where (u, v) = (target_uv - dst.xy) * dst.zw is the point measured in the
// rectangle, 0,0 at its top-left and 1,1 at its bottom-right. `dst.zw` is
// one over the rectangle's share of the target: on the whole target it is
// exactly 1 and (u, v) is the target's own point. video_pass.rs works the
// numbers out from the crop, the rotation and the rectangle (D-133).
struct Placement {
    m: vec4<f32>,
    t: vec4<f32>,
    dst: vec4<f32>,
}

@group(0) @binding(0) var frame: texture_2d<f32>;
@group(0) @binding(1) var smooth_sampler: sampler;
@group(0) @binding(2) var<uniform> placement: Placement;
@group(0) @binding(3) var overlay: texture_2d<f32>;

struct Corner {
    @builtin(position) position: vec4<f32>,
    // 0,0 is the top-left corner of the target and 1,1 its bottom-right.
    @location(0) uv: vec2<f32>,
}

// One triangle with corners at (-1,-1), (3,-1) and (-1,3): the target is the
// part of it between -1 and 1.
@vertex
fn whole_target(@builtin(vertex_index) index: u32) -> Corner {
    let x = f32(i32(index & 1u) * 4 - 1);
    let y = f32(i32(index >> 1u) * 4 - 1);
    var corner: Corner;
    corner.position = vec4<f32>(x, y, 0.0, 1.0);
    corner.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
    return corner;
}

// Outside the rectangle the target is black. The frame is sampled at every
// point, so that the sample stays in uniform control flow, and black is
// chosen after.
@fragment
fn video(corner: Corner) -> @location(0) vec4<f32> {
    let local = (corner.uv - placement.dst.xy) * placement.dst.zw;
    let uv = vec2<f32>(
        placement.m.x * local.x + placement.m.z * local.y + placement.t.x,
        placement.m.y * local.x + placement.m.w * local.y + placement.t.y,
    );
    let sampled = textureSample(frame, smooth_sampler, uv).rgb;
    let inside = all(local >= vec2<f32>(0.0)) && all(local <= vec2<f32>(1.0));
    return vec4<f32>(select(vec3<f32>(0.0), sampled, inside), 1.0);
}

// The overlay texture holds straight alpha. It is premultiplied here, and the
// pipeline blends it as premultiplied: colour + (1 - alpha) x what is there.
@fragment
fn over(corner: Corner) -> @location(0) vec4<f32> {
    let pixel = textureSample(overlay, smooth_sampler, corner.uv);
    return vec4<f32>(pixel.rgb * pixel.a, pixel.a);
}
