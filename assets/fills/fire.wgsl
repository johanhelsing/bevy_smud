#define_import_path smud::fire_fill

#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

// Noise functions copied from noisy_bevy

fn permute_4_(x: vec4<f32>) -> vec4<f32> {
    return ((x * 34. + 1.) * x) % vec4<f32>(289.);
}

fn taylor_inv_sqrt_4_(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn simplex_noise_3d(v: vec3<f32>) -> f32 {
    let C = vec2(1. / 6., 1. / 3.);
    let D = vec4(0., 0.5, 1., 2.);

    // first corner
    var i = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);

    // other corners
    let g = step(x0.yzx, x0.xyz);
    let l = 1. - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    // x0 = x0 - 0. + 0. * C
    let x1 = x0 - i1 + 1. * C.xxx;
    let x2 = x0 - i2 + 2. * C.xxx;
    let x3 = x0 - 1. + 3. * C.xxx;

    // permutations
    i = i % vec3(289.);
    let p = permute_4_(permute_4_(permute_4_(
        i.z + vec4(0., i1.z, i2.z, 1.)) +
        i.y + vec4(0., i1.y, i2.y, 1.)) +
        i.x + vec4(0., i1.x, i2.x, 1.)
    );

    // gradients (NxN points uniformly over a square, mapped onto an octahedron)
    let n_ = 1. / 7.; // N=7
    let ns = n_ * D.wyz - D.xzx;

    let j = p - 49. * floor(p * ns.z * ns.z); // mod(p, N*N)

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7. * x_); // mod(j, N)

    let x = x_ * ns.x + ns.yyyy;
    let y = y_ * ns.x + ns.yyyy;
    let h = 1. - abs(x) - abs(y);

    let b0 = vec4(x.xy, y.xy);
    let b1 = vec4(x.zw, y.zw);

    let s0 = floor(b0) * 2. + 1.;
    let s1 = floor(b1) * 2. + 1.;
    let sh = -step(h, vec4(0.));

    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3(a0.xy, h.x);
    var p1 = vec3(a0.zw, h.y);
    var p2 = vec3(a1.xy, h.z);
    var p3 = vec3(a1.zw, h.w);

    // normalize gradients
    let norm = taylor_inv_sqrt_4_(vec4(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 = p0 * norm.x;
    p1 = p1 * norm.y;
    p2 = p2 * norm.z;
    p3 = p3 * norm.w;

    // mix final noise value
    var m = 0.5 - vec4(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
    m = max(m, vec4(0.));
    m *= m;
    return 105. * dot(m * m, vec4(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}

/// Fractional brownian motion (fbm) based on 3d simplex noise
fn fbm_simplex_3d(pos: vec3<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    var sum = 0.;
    var amplitude = 1.;
    var frequency = 1.;

    for (var i = 0; i < octaves; i+= 1) {
        sum += simplex_noise_3d(pos * frequency) * amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    return sum;
}

fn fill(input: smud::FillInput) -> vec4<f32> {
    let t = globals.time;
    let p = input.pos;
    var d = input.distance;
    let noise_scale = 0.03;
    let scroll_speed = 0.4;

    // Domain warping: use noise to offset the position before sampling main noise
    let warp_noise = fbm_simplex_3d(vec3<f32>(p.x * 0.01, p.y * 0.01 - t * 0.2, t * 0.08), 3, 2., 0.5);
    let warped_pos = p + vec2<f32>(warp_noise * 10.0, warp_noise * 5.0);

    let noise = fbm_simplex_3d(vec3<f32>(warped_pos.x * noise_scale, warped_pos.y * noise_scale - t * scroll_speed, t * 0.05), 7, 2., 0.5) * 0.3;

    // Increase distortion towards the top of the flame
    let flame_distortion_t = saturate(p.y * 0.002 + 0.5);
    d -= (mix(0, noise, flame_distortion_t) + 0.2) * 30.0;

    // Alpha with cubic falloff
    let d2 = 1. - (d * 0.13);
    let alpha = clamp(d2 * d2 * d2, 0., 1.) * input.color.a;
    let aaf = 0.7 / fwidth(d);

    // Color palette: white-hot core -> yellow -> orange -> red -> dark
    // Temperature gradient based on height and distance from core
    let height_temp = -p.y * 0.0017 + noise * 0.15;
    let core_temp = clamp(-d * 0.008, 0., 1.);

    // Time-based flickering for dynamic intensity
    let flicker_scale = 0.003; // very low scale for large area flicker
    var flicker = simplex_noise_3d(vec3<f32>(p.x * flicker_scale, p.y * flicker_scale, t * 2.0)) * 0.1 + 0.95; // Range: 0.85 to 1.05

    // As d approaches 0 (the edge), fade to dark/smoke regardless of other factors
    let edge_fade = smoothstep(10.0, -10.0, d); // 1 when deep inside, 0 at edge
    let temp = saturate(height_temp + core_temp + 0.3) * edge_fade * flicker;

    // Fire color ramp
    var c: vec3<f32>;
    if (temp > 0.8) {
        // White hot core
        c = mix(vec3<f32>(1.0, 0.9, 0.5), vec3<f32>(1.0, 1.0, 1.0), (temp - 0.8) * 5.0);
    } else if (temp > 0.5) {
        // Yellow to orange
        c = mix(vec3<f32>(1.0, 0.5, 0.0), vec3<f32>(1.0, 0.9, 0.5), (temp - 0.5) * 3.33);
    } else if (temp > 0.2) {
        // Red to orange
        c = mix(vec3<f32>(0.8, 0.1, 0.0), vec3<f32>(1.0, 0.5, 0.0), (temp - 0.2) * 3.33);
    } else {
        // Dark red to black
        c = mix(vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.8, 0.1, 0.0), temp * 5.0);
    }

    let shadow_color = 0.2 * c;
    let c_falloff = mix(c, shadow_color, clamp(d * aaf, 0., 1.));

    return vec4<f32>(c_falloff, alpha);
}