#define_import_path smud
// Most of these are ported versions of the ones on Inigo Quilez website, https://iquilezles.org

fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var r_2 = r;
    // swizzle assigment isn't supported yet
    // r_2.xy = select(r_2.zw, r_2.xy, p.x > 0.);
    let tmp = select(r_2.zw, r_2.xy, p.x > 0.);
    r_2.x = tmp.x;
    r_2.y = tmp.y;
    r_2.x = select(r_2.y, r_2.x, p.y > 0.);
    let q = abs(p) - b + r_2.x;
    return min(
        max(q.x, q.y),
        0.
    ) + length(max(q, vec2<f32>(0.))) - r_2.x;
}

fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.))) + min(max(d.x, d.y), 0.);
}

fn sd_oriented_box(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, th: f32) -> f32 {
    let l = length(b - a);
    let d = (b - a) / l;
    var q = (p - (a + b) * 0.5);
    q = mat2x2<f32>(d.x, -d.y, d.y, d.x) * q;
    q = abs(q) - vec2<f32>(l, th) * 0.5;
    return length(max(q, vec2<f32>(0.))) + min(max(q.x, q.y), 0.);
}

fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba)/dot(ba, ba), 0., 1.);
    return length(pa - ba * h);
}

fn ndot(a: vec2<f32>, b: vec2<f32>) -> f32 {
    return a.x * b.x - a.y * b.y;
}

fn dot2_(a: vec2<f32>) -> f32 {
    return dot(a, a);
}

fn sd_rhombus(p: vec2<f32>, b: vec2<f32>)  -> f32 {
    let p_2 = abs(p);
    let h = clamp(ndot(b - 2. * p_2, b) / dot(b, b), -1., 1.);
    let d = length(p_2 - 0.5 * b * vec2<f32>(1. - h, 1. + h));
    return d * sign(p_2.x * b.y + p_2.y * b.x - b.x * b.y);
}

fn sd_trapezoid(p: vec2<f32>, r1: f32, r2: f32, he: f32) -> f32 {
    var p_2 = p;
    let k1 = vec2<f32>(r2, he);
    let k2 = vec2<f32>(r2 - r1, 2. * he);
    p_2.x = abs(p_2.x);
    let r = select(r2, r1, p_2.y < 0.);
    let ca = vec2<f32>(p_2.x - min(p_2.x, r), abs(p_2.y) - he);
    let cb = p_2 - k1 + k2 * clamp(dot(k1 - p_2, k2) / dot2_(k2), 0., 1.);
    let s = select(1., -1., cb.x < 0. && ca.y < 0.);
    return s * sqrt(min(dot2_(ca), dot2_(cb)));
}

fn sd_parallelogram(p: vec2<f32>, wi: f32, he: f32, sk: f32) -> f32 {
    let e = vec2<f32>(sk, he);
    var p_2 = select(p, -p, p.y < 0.);
    var w = p_2 - e;
    w.x = w.x - clamp(w.x, -wi, wi);
    var d = vec2<f32>(dot(w, w), -w.y);
    let s = p_2.x*e.y - p_2.y*e.x;
    p_2 = select(p_2, -p_2, s < 0.);
    var v = p_2 - vec2<f32>(wi, 0.);
    v = v - e * clamp(dot(v, e) / dot(e, e), -1., 1.);
    d = min(d, vec2<f32>(dot(v, v), wi * he - abs(s)));
    return sqrt(d.x)*sign(-d.y);
}

fn sd_equilateral_triangle(p: vec2<f32>, r: f32) -> f32 {
    var p_2 = p;
    let k = sqrt(3.);
    p_2.x = abs(p_2.x) - r;
    p_2.y = p_2.y + r / k;
    if (p_2.x + k * p_2.y > 0.) {
        p_2 = vec2<f32>(p_2.x - k * p_2.y, -k * p_2.x - p_2.y) / 2.;
    }
    p_2.x = p_2.x - clamp(p_2.x, -2. * r, 0.);
    return -length(p_2) * sign(p_2.y);
}

fn sd_triangle_isosceles(p: vec2<f32>, q: vec2<f32>) -> f32 {
    var p_2 = p;
    p_2.x = abs(p_2.x);
    let a = p_2 - q * clamp(dot(p_2, q)/dot(q, q), 0., 1.);
    let b = p_2 - q * vec2<f32>(clamp(p_2.x / q.x, 0., 1.), 1.);
    let s = -sign(q.y);
    let d = min(vec2<f32>(dot(a, a), s * (p_2.x * q.y - p_2.y * q.x)),
                  vec2<f32>(dot(b, b), s * (p_2.y - q.y)));
    return -sqrt(d.x) * sign(d.y);
}

fn sd_triangle(p: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    let e0 = p1 - p0;
    let e1 = p2 - p1;
    let e2 = p0 - p2;

    let v0 = p - p0;
    let v1 = p - p1;
    let v2 = p - p2;

    let pq0 = v0 - e0*clamp(dot(v0, e0)/dot(e0, e0), 0., 1.);
    let pq1 = v1 - e1*clamp(dot(v1, e1)/dot(e1, e1), 0., 1.);
    let pq2 = v2 - e2*clamp(dot(v2, e2)/dot(e2, e2), 0., 1.);

    let s = sign(e0.x*e2.y - e0.y*e2.x);
    let d = min(min(vec2<f32>(dot(pq0, pq0), s*(v0.x*e0.y-v0.y*e0.x)),
                     vec2<f32>(dot(pq1, pq1), s*(v1.x*e1.y-v1.y*e1.x))),
                     vec2<f32>(dot(pq2, pq2), s*(v2.x*e2.y-v2.y*e2.x)));
    return -sqrt(d.x)*sign(d.y);
}

fn sd_uneven_capsule(p: vec2<f32>, r1: f32, r2: f32, h: f32) -> f32 {
    var p_2 = p;
    p_2.x = abs(p_2.x);
    let b = (r1 - r2) / h;
    let a = sqrt(1. - b * b);
    let k = dot(p_2, vec2<f32>(-b, a));
    if (k < 0.) {
        return length(p_2) - r1;
    }
    if (k > a*h) {
        return length(p_2 - vec2<f32>(0., h)) - r2;
    }
    return dot(p_2, vec2<f32>(a, b)) - r1;
}

fn sd_pentagon(p: vec2<f32>, r: f32) -> f32 {
    let k = vec3<f32>(0.809016994, 0.587785252, 0.726542528);
    var p_2 = p;
    p_2.x = abs(p_2.x);
    p_2 = p_2 - 2. * min(dot(vec2<f32>(-k.x, k.y), p_2), 0.) * vec2<f32>(-k.x, k.y);
    p_2 = p_2 - 2. * min(dot(vec2<f32>(k.x, k.y), p_2), 0.) * vec2<f32>(k.x, k.y);
    p_2 = p_2 - vec2<f32>(clamp(p_2.x, -r * k.z, r * k.z), r);
    return length(p_2) * sign(p_2.y);
}

fn sd_hexagon(p_in: vec2<f32>, r: f32) -> f32 {
    let k = vec3<f32>(-0.866025404, 0.5, 0.577350269);
    var p = abs(p_in);
    p = p - 2. * min(dot(k.xy, p), 0.) * k.xy;
    p = p - vec2<f32>(clamp(p.x, -k.z * r, k.z * r), r);
    return length(p) * sign(p.y);
}

fn sd_octagon(p: vec2<f32>, r: f32) -> f32 {
    let k = vec3<f32>(-0.9238795325, 0.3826834323, 0.4142135623);
    var p_2 = abs(p);
    p_2 = p_2 - 2. * min(dot(vec2<f32>(k.x, k.y), p_2), 0.) * vec2<f32>(k.x, k.y);
    p_2 = p_2 - 2. * min(dot(vec2<f32>(-k.x, k.y), p_2), 0.) * vec2<f32>(-k.x, k.y);
    p_2 = p_2 - vec2<f32>(clamp(p_2.x, -k.z * r, k.z * r), r);
    return length(p_2) * sign(p_2.y);
}

fn sd_hexagram(p_in: vec2<f32>, r: f32) -> f32 {
    let k = vec4<f32>(-0.5, 0.8660254038, 0.5773502692, 1.7320508076);
    var p = abs(p_in);
    p = p - 2. * min(dot(k.xy, p), 0.) * k.xy;
    p = p - 2. * min(dot(k.yx, p), 0.) * k.yx;
    p = p - vec2<f32>(clamp(p.x, r * k.z, r * k.w), r);
    return length(p) * sign(p.y);
}

fn sd_star_5_(p_in: vec2<f32>, r: f32, rf: f32) -> f32 {
    let k1 = vec2<f32>(0.809016994375, -0.587785252292);
    let k2 = vec2<f32>(-k1.x, k1.y);
    var p = p_in;
    p.x = abs(p.x);
    p = p - 2. * max(dot(k1, p), 0.) * k1;
    p = p - 2. * max(dot(k2, p), 0.) * k2;
    p.x = abs(p.x);
    p.y = p.y - r;
    let ba = rf*vec2<f32>(-k1.y, k1.x) - vec2<f32>(0., 1.);
    let h = clamp(dot(p, ba) / dot(ba, ba), 0., r);
    return length(p - ba * h) * sign(p.y * ba.x - p.x * ba.y);
}

// wgsl made the cowardly choice of keeping an incorrectly named modf,
// function and doesn't currently have a glsl mod equivalent :(
// see: https://github.com/gpuweb/gpuweb/issues/3987
fn modulo(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

fn sd_star(p_in: vec2<f32>, r: f32, n: i32, m: f32) -> f32 {
    // next 4 lines can be precomputed for a given shape
    let an = 3.141593 / f32(n);
    let en = 3.141593 / m; // m is between 2 and n
    let acs = vec2<f32>(cos(an), sin(an));
    let ecs = vec2<f32>(cos(en), sin(en)); // ecs=vec2(0, 1) for regular polygon

    let bn = modulo(atan2(p_in.x, p_in.y), (2. * an)) - an;
    var p = length(p_in) * vec2<f32>(cos(bn), abs(sin(bn)));
    p = p - r * acs;
    p = p + ecs * clamp(-dot(p, ecs), 0., r * acs.y / ecs.y);
    return length(p) * sign(p.x);
}

fn sd_pie(p_in: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
    var p = p_in;
    p.x = abs(p.x);
    let l = length(p) - r;
    let m = length(p - c * clamp(dot(p, c), 0., r)); // c=sin/cos of aperture
    return max(l, m * sign(c.y * p.x - c.x * p.y));
}

fn sd_cut_disk(p_in: vec2<f32>, r: f32, h: f32) -> f32 {
    let w = sqrt(r * r - h * h); // constant for any given shape
    var p = p_in;
    p.x = abs(p.x);
    let s = max(
        (h - r) * p.x * p.x + w * w * (h + r - 2. * p.y),
        h * p.x - w * p.y
    );
    return select(
        select(length(p-vec2<f32>(w, h)), h - p.y, p.x< w),
        length(p)-r,
        s < 0.
    );
}

// sc is the sin/cos of the arc's aperture
fn sd_arc(p_in: vec2<f32>, sc: vec2<f32>, ra: f32, rb: f32) -> f32 {
    var p = p_in;
    p.x = abs(p.x);
    return select(
        abs(length(p)-ra),
        length(p - sc * ra),
        sc.y * p.x > sc.x * p.y
    ) - rb;
}

fn sd_arc_oriented(p_in: vec2<f32>, sc_orientation: vec2<f32>, sc_aperture: vec2<f32>, ra: f32, rb: f32) -> f32
{
    var p = p_in;
    p = p * mat2x2<f32>(sc_orientation.x, sc_orientation.y, -sc_orientation.y, sc_orientation.x);
    p.x = abs(p.x);
    let k = select(
        length(p),
        dot(p, sc_aperture),
        sc_aperture.y*p.x > sc_aperture.x*p.y
    );
    return sqrt(dot(p, p) + ra * ra - 2. * ra * k) - rb;
}

fn sd_horseshoe(p_in: vec2<f32>, c: vec2<f32>, r: f32, w: vec2<f32>) -> f32 {
    var p = p_in;
    p.x = abs(p.x);
    let l = length(p);
    p = mat2x2<f32>(-c.x, c.y, c.y, c.x) * p;
    p = vec2<f32>(
        select(l*sign(-c.x), p.x, p.y > 0. || p.x > 0.),
        select(l, p.y, p.x > 0.)
    );
    p = vec2<f32>(p.x, abs(p.y - r)) - w;
    return length(max(p, vec2<f32>(0.))) + min(0., max(p.x, p.y));
}

fn sd_rounded_cross(p_in: vec2<f32>, h: f32) -> f32 {
    let k = 0.5 * (h + 1. / h); // k should be const at modeling time
    var p = abs(p_in);
    return select(
        sqrt(min(
            dot2_(p - vec2<f32>(0., h)),
            dot2_(p - vec2<f32>(1., 0.))
        )),
        k - sqrt(dot2_(p - vec2<f32>(1., k))),
        p.x < 1. && p.y < p.x * (k - h) + h
    );
}

fn sd_egg(p_in: vec2<f32>, ra: f32, rb: f32) -> f32 {
    var p = p_in;
    let k = sqrt(3.);
    p.x = abs(p.x);
    let r = ra - rb;
    return select(
        select(
            length(vec2<f32>(p.x + r, p.y)) - 2. * r,
            length(vec2<f32>(p.x, p.y - k * r)),
            k * (p.x + r) < p.y
        ),
        length(vec2<f32>(p.x, p.y)) - r,
        p.y < 0.
    ) - rb;
}

fn sd_heart(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    p.x = abs(p.x);

    if (p.y + p.x > 1.) {
        return sqrt(dot2_(p - vec2<f32>(0.25, 0.75))) - sqrt(2.) / 4.;
    }

    return sqrt(min(
        dot2_(p - vec2<f32>(0., 1.)),
        dot2_(p - 0.5 * max(p.x + p.y, 0.))
    )) * sign(p.x - p.y);
}

fn sd_cross(p_in: vec2<f32>, b: vec2<f32>, r: f32)  -> f32 {
    var p = abs(p_in);
    p = select(p.xy, p.yx, p.y > p.x);
    let q = p - b;
    let k = max(q.y, q.x);
    let w = select(
        vec2<f32>(b.y - p.x, -k),
        q,
        k > 0.
    );
    return sign(k) * length(max(w, vec2<f32>(0.))) + r;
}

fn sd_rounded_x(p_in: vec2<f32>, w: f32, r: f32) -> f32 {
    let p = abs(p_in);
    return length(p - min(p.x + p.y, w) * 0.5) - r;
}

// fn sd_polygon(vec2[N] v, p: vec2<f32>) -> f32 {
//     var d = dot(p - v[0], p - v[0]);
//     var s = 1.;
//     for(int i = 0, j = N - 1; i < N; j = i, i++) {
//         let e = v[j] - v[i];
//         let w = p - v[i];
//         let b = w - e * clamp(dot(w, e) / dot(e, e), 0., 1.);
//         d = min(d, dot(b, b));
//         let c = vec3<bool>(
//             p.y >= v[i].y,
//             p.y < v[j].y,
//             e.x * w.y > e.y * w.x
//         );
//         if (all(c) || all(not(c))) {
//             s *= -1.;
//         }
//     }
//     return s * sqrt(d);
// }

// https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm
// Has huge issues with instability when close to a circle or very eccentric
fn sd_ellipse(p_in: vec2<f32>, a: f32, b: f32) -> f32 {
    var p = abs(p_in);
    var ab = vec2<f32>(a, b);
    if (p.x > p.y) {
        p = p.yx;
        ab = ab.yx;
    }
    let l = ab.y * ab.y - ab.x * ab.x;
    let m = ab.x * p.x / l;
    let m2 = m * m;
    let n = ab.y * p.y / l;
    let n2 = n * n;
    let c = (m2 + n2 - 1.) / 3.;
    let c3 = c * c * c;
    let q = c3 + m2 * n2 * 2.;
    let d = c3 + m2 * n2;
    let g = m + m * n2;
    var co: f32;
    if (d < 0.) {
        let h = acos(q / c3) / 3.;
        let s = cos(h);
        let t = sin(h) * sqrt(3.);
        let rx = sqrt(-c * (s + t + 2.) + m2);
        let ry = sqrt(-c * (s - t + 2.) + m2);
        co = (ry + sign(l) * rx + abs(g) / (rx * ry) - m) / 2.;
    } else {
        let h = 2. * m * n * sqrt(d);
        let s = sign(q + h) * pow(abs(q + h), 1. / 3.);
        let u = sign(q - h) * pow(abs(q - h), 1. / 3.);
        let rx = -s - u - c * 4. + 2. * m2;
        let ry = (s - u) * sqrt(3.);
        let rm = sqrt(rx * rx + ry * ry);
        co = (ry / sqrt(rm - rx) + 2. * g / rm - m) / 2.;
    }
    let r = ab * vec2<f32>(co, sqrt(1. - co * co));
    return length(r - p) * sign(p.y - r.y);
}

fn sd_parabola(p_in: vec2<f32>, k: f32) -> f32 {
    var pos = p_in;
    pos.x = abs(pos.x);
    let ik = 1. / k;
    let p = ik * (pos.y - 0.5 * ik) / 3.;
    let q = 0.25 * ik * ik * pos.x;
    let h = q * q - p * p * p;
    let r = sqrt(abs(h));
    let x = select(
        2. * cos(atan2(r, q) / 3.) * sqrt(p),
        pow(q + r, 1. / 3.) - pow(abs(q - r), 1. / 3.) * sign(r - q),
        h > 0.
    );
    return length(pos-vec2<f32>(x, k*x*x)) * sign(pos.x-x);
}

fn sd_parabola_segment(p_in: vec2<f32>, wi: f32, he: f32) -> f32 {
    var pos = p_in;
    pos.x = abs(pos.x);
    let ik = wi * wi / he;
    let p = ik * (he - pos.y - 0.5 * ik) / 3.;
    let q = pos.x * ik * ik * 0.25;
    let h = q * q - p * p * p;
    let r = sqrt(abs(h));
    var x = select(
        2. * cos(atan(r / q) / 3.) * sqrt(p),
        pow(q + r, 1. / 3.) - pow(abs(q - r), 1. / 3.) * sign(r - q),
        h > 0.
    );
    x = min(x, wi);
    return length(pos - vec2<f32>(x, he - x * x / ik)) *
        sign(ik * (pos.y - he) + pos.x * pos.x);
}

fn sd_bezier(pos: vec2<f32>, A: vec2<f32>, B: vec2<f32>, C: vec2<f32>) -> f32 {
    let a = B - A;
    let b = A - 2. * B + C;
    let c = a * 2.;
    let d = A - pos;
    let kk = 1. / dot(b, b);
    let kx = kk * dot(a, b);
    let ky = kk * (2. * dot(a, a) + dot(d, b)) / 3.;
    let kz = kk * dot(d, a);
    var res = 0.;
    let p = ky - kx * kx;
    let p3 = p * p * p;
    let q = kx * (2. * kx * kx - 3. * ky) + kz;
    var h = q * q + 4. * p3;
    if (h >= 0.)
    {
        h = sqrt(h);
        let x = (vec2<f32>(h, -h) - q) / 2.;
        let uv = sign(x) * pow(abs(x), vec2<f32>(1. / 3.));
        let t = clamp(uv.x + uv.y - kx, 0., 1.);
        res = dot2_(d + (c + b * t) * t);
    }
    else
    {
        let z = sqrt(-p);
        let v = acos(q / (p * z * 2.)) / 3.;
        let m = cos(v);
        let n = sin(v) * 1.732050808;
        let u = vec3<f32>(m + m, -n - m, n - m) * z - vec3<f32>(kx);
        let t = clamp(u, vec3<f32>(0.), vec3<f32>(1.));
        res = min(
            dot2_(d + (c + b * t.x) * t.x),
            dot2_(d + (c + b * t.y) * t.y)
        );
        // the third root cannot be the closest
        // res = min(res, dot2(d+(c+b*t.z)*t.z));
    }
    return sqrt(res);
}

fn sd_blobby_cross(p_in: vec2<f32>, he: f32) -> f32 {
    var pos = abs(p_in);
    pos = vec2<f32>(
        abs(pos.x - pos.y),
        1. - pos.x - pos.y
    ) / sqrt(2.);

    let p = (he - pos.y - 0.25 / he) / (6. * he);
    let q = pos.x / (he * he * 16.);
    let h = q * q - p * p * p;

    var x: f32;
    if (h > 0.) {
        let r = sqrt(h);
        x = pow(q + r, 1. / 3.) - pow(abs(q - r), 1. / 3.) * sign(r - q);
    } else {
        let r = sqrt(p);
        x = 2. * r * cos(acos(q / (p * r)) / 3.);
    }
    x = min(x, sqrt(2.) / 2.);

    let z = vec2<f32>(x, he * (1. - 2. * x * x)) - pos;
    return length(z) * sign(z.y);
}

fn sd_tunnel(p_in: vec2<f32>, wh: vec2<f32>) -> f32 {
    let p = vec2<f32>(abs(p_in.x), -p_in.y);
    var q = p - wh;

    let d1 = dot2_(vec2<f32>(max(q.x, 0.), q.y));
    q.x = select(length(p) - wh.x, q.x, p.y > 0.);
    let d2 = dot2_(vec2<f32>(q.x, max(q.y, 0.)));
    let d = sqrt(min(d1, d2));

    return select(d, -d, max(q.x, q.y) < 0.);
}

fn sd_stairs(p_in: vec2<f32>, wh: vec2<f32>, n: f32) -> f32 {
    var p = p_in;
    let ba = wh * n;
    var d = min(
        dot2_(p - vec2<f32>(clamp(p.x, 0., ba.x), 0.)),
        dot2_(p - vec2<f32>(ba.x, clamp(p.y, 0., ba.y)))
    );
    var s = sign(max(-p.y, p.x - ba.x));

    let dia = length(wh);
    p = mat2x2<f32>(wh.x, -wh.y, wh.y, wh.x) * p / dia;
    let id = clamp(round(p.x / dia), 0., n - 1.);
    p.x = p.x - id * dia;
    p = mat2x2<f32>(wh.x, wh.y,-wh.y, wh.x) * p / dia;

    let hh = wh.y / 2.;
    p.y = p.y - hh;
    if (p.y > hh * sign(p.x)) {
        s = 1.;
    }
    p = select(-p, p, id < 0.5 || p.x > 0.);
    d = min(d, dot2_(p - vec2<f32>(0., clamp(p.y, -hh, hh))));
    d = min(d, dot2_(p - vec2<f32>(clamp(p.x, 0., wh.x), hh)));

    return sqrt(d) * s;
}

fn sd_vesica(p_in: vec2<f32>, r: f32, d: f32) -> f32 {
    let p = abs(p_in);
    let b = sqrt(r * r - d * d);
    return select(
        length(p - vec2<f32>(-d, 0.)) - r,
        length(p - vec2<f32>(0., b)),
        (p.y - b) * d > p.x * b
    );
}

fn sd_moon(p_in: vec2<f32>, d: f32, ra: f32, rb: f32) -> f32 {
    var p = p_in;
    p.y = abs(p.y);
    let a = (ra * ra - rb * rb + d * d) / (2. * d);
    let b = sqrt(max(ra * ra - a * a, 0.));

    if (d * (p.x * b - p.y * a) > d * d * max(b - p.y, 0.)) {
        return length(p-vec2<f32>(a, b));
    }

    return max(
        length(p) - ra,
        -(length(p - vec2<f32>(d, 0.)) - rb)
    );
}

fn renormalize_uv(uv: vec2<f32>) -> vec2<f32> {
    return uv * 2. - vec2<f32>(1., 1.);
}

fn exponential_falloff(d: f32, size: f32, power: f32) -> f32 {
    var a = (size - d) / size;
    a = clamp(a, 0.0, 1.0);
    a = pow(a, power);
    return a;
}

fn exponential_falloff_3_(d: f32, size: f32) -> f32 {
    var a = (size - d) / size;
    a = clamp(a, 0.0, 1.0);
    a = a * a * a;
    return a;
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothstep(aaf, -aaf, distance);
}

// I think this one looks better than the fwidth version, but perhaps it's more expensive?
fn sd_fill_alpha_dpd(distance: f32) -> f32 {
    let aaf = length(vec2<f32>(dpdx(distance), dpdy(distance))) * 0.71;
    return smoothstep(aaf, -aaf, distance);
}

// Dirt cheap, but ugly
fn sd_fill_alpha_nearest(distance: f32) -> f32 {
    return step(-distance, 0.);
}

fn sd_fill_with_falloff_3_(d: f32, falloff_size: f32, falloff_color: vec4<f32>, fill_color: vec4<f32>) -> vec4<f32> {
    // todo compose with others?
    let aaf = 0.7 / fwidth(d); // TODO: this could just be a uniform instead
    let t_color = clamp(d * aaf, 0.0, 1.0);
    var color = mix(fill_color, falloff_color, t_color);
    let falloff = exponential_falloff_3_(d, falloff_size);
    color.a = color.a * falloff;
    return color;
}

fn op_union(distance_1: f32, distance_2: f32) -> f32 {
    return min(distance_1, distance_2);
}

fn op_subtract(distance_1: f32, distance_2: f32) -> f32 {
    return max(-distance_1, distance_2);
}

fn op_intersect(distance_1: f32, distance_2: f32) -> f32 {
    return max(distance_1, distance_2);
}

// rotations

fn sin_cos(a: f32) -> vec2<f32> {
    return vec2<f32>(sin(a), cos(a));
}

// Rotation given sin cos vector
fn rotate(p: vec2<f32>, sc: vec2<f32>) -> vec2<f32> {
    let s = sc.x;
    let c = sc.y;
    return vec2<f32>(
        p.x * c - p.y * s,
        p.x * s + p.y * c,
    );
}

fn rotate_rad(p: vec2<f32>, a: f32) -> vec2<f32> {
    return rotate(p, sin_cos(a));
}

fn rotate_45_(p: vec2<f32>) -> vec2<f32> {
    let c = 0.70710678118; // cos(pi / 4) == sin(pi / 4);
    let xc = p.x * c;
    let yc = p.y * c;
    return vec2<f32>(xc - yc, xc + yc);
}

fn op_smooth_subtract(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0., 1.);
    return mix(d2, -d1, h) + k * h * (1. - h);
}

fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0., 1.);
    return mix(d2, d1, h) - k * h * (1. - h);
}

fn op_smooth_intersect(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0., 1.);
    return mix(d2, d1, h) + k * h * (1. - h);
}

// // complex (and sometimes inexact shapes:)

// fn sd_arrow_head(p: vec2<f32>, w: f32, h: f32) -> f32 {
//     var p = p;
//     p.x = abs(p.x);
//     return sd_segment(p, vec2<f32>(0., 0.), vec2<f32>(w, -h));
// }