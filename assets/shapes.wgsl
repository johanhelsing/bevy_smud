// most of these are ported versions of the ones on inigo quilez website, https://iquilezles.org

fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

// https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm
// Has huge issues with instability when close to a circle or very eccentric
fn sd_ellipse(p: vec2<f32>, a: f32, b: f32) -> f32 {
    var p = abs(p);
    var ab = vec2<f32>(a, b);
    if (p.x > p.y)
    {
        p = p.yx;
        ab = ab.yx;
    }
    let l = ab.y * ab.y - ab.x * ab.x;
    let m = ab.x * p.x / l;
    let m2 = m * m;
    let n = ab.y * p.y / l;
    let n2 = n * n;
    let c = (m2 + n2 - 1.0) / 3.0;
    let c3 = c * c * c;
    let q = c3 + m2 * n2 * 2.0;
    let d = c3 + m2 * n2;
    let g = m + m * n2;
    var co: f32;
    if (d < 0.0)
    {
        let h = acos(q / c3) / 3.0;
        let s = cos(h);
        let t = sin(h) * sqrt(3.0);
        let rx = sqrt(-c * (s + t + 2.0) + m2);
        let ry = sqrt(-c * (s - t + 2.0) + m2);
        co = (ry + sign(l) * rx + abs(g) / (rx * ry) - m) / 2.0;
    }
    else
    {
        let h = 2.0 * m * n * sqrt(d);
        let s = sign(q + h) * pow(abs(q + h), 1.0 / 3.0);
        let u = sign(q - h) * pow(abs(q - h), 1.0 / 3.0);
        let rx = -s - u - c * 4.0 + 2.0 * m2;
        let ry = (s - u) * sqrt(3.0);
        let rm = sqrt(rx * rx + ry * ry);
        co = (ry / sqrt(rm - rx) + 2.0 * g / rm - m) / 2.0;
    }
    let r = ab * vec2<f32>(co, sqrt(1.0 - co * co));
    return length(r - p) * sign(p.y - r.y);
}

fn sd_vesica(p: vec2<f32>, r: f32, d: f32) -> f32 {
    let p = abs(p);
    let b = sqrt(r * r - d * d);
    return select(
        length(p - vec2<f32>(-d, 0.0)) - r,
        length(p - vec2<f32>(0.0, b)),
        (p.y - b) * d > p.x * b
    );
}

fn sd_moon(p: vec2<f32>, d: f32, ra: f32, rb: f32) -> f32
{
    var p = p;
    p.y = abs(p.y);
    let a = (ra * ra - rb * rb + d * d) / (2. * d);
    let b = sqrt(max(ra*ra - a*a, 0.));

    if (d * (p.x * b - p.y * a) > d * d * max(b - p.y, 0.)) {
        return length(p-vec2<f32>(a,b));
    }

    return max(
        length(p)-ra,
        -(length(p - vec2<f32>(d, 0.)) - rb)
    );
}

fn sd_renormalize_uv(uv: vec2<f32>) -> vec2<f32> {
    return uv * 2. - vec2<f32>(1., 1.);
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothStep(aaf, -aaf, distance);
}

// I think this one looks better than the fwidth version, but perhaps it's more expensive?
fn sd_fill_alpha_dpd(distance: f32) -> f32 {
    let aaf = length(vec2<f32>(dpdx(distance), dpdy(distance))) * 0.71;
    return smoothStep(aaf, -aaf, distance);
}

// Dirt cheap, but ugly
fn sd_fill_alpha_nearest(distance: f32) -> f32 { return step(-distance, 0.);
}

fn sd_union(distance_1: f32, distance_2: f32) -> f32 {
    return min(distance_1, distance_2);
}

fn sd_subtract(distance_1: f32, distance_2: f32) -> f32 {
    return max(-distance_1, distance_2);
}

fn sd_intersect(distance_1: f32, distance_2: f32) -> f32 {
    return max(distance_1, distance_2);
}

// rotations

fn sin_cos(a: f32) -> vec2<f32> {
    return vec2<f32>(sin(a), cos(a));
}

// Rotation given sin cos vector
fn sd_rotate(p: vec2<f32>, sc: vec2<f32>) -> vec2<f32> {
    let s = sc.x;
    let c = sc.y;
    return vec2<f32>(
        p.x * c - p.y * s,
        p.x * s + p.y * c,
    );
}

fn sd_rotate_rad(p: vec2<f32>, a: f32) -> vec2<f32> {
    return sd_rotate(p, sin_cos(a));
}

fn sd_rotate_45(p: vec2<f32>) -> vec2<f32> {
    let c = 0.70710678118; // cos(pi / 4) == sin(pi / 4);
    let xc = p.x * c;
    let yc = p.y * c;
    return vec2<f32>(xc - yc, xc + yc);
}

fn sd_smooth_subtract(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return mix(d2, -d1, h) + k * h * (1.0 - h);
}

fn sd_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn sd_smooth_intersect(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) + k * h * (1.0 - h);
}