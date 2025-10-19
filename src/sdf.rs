//! Signed Distance Field (SDF) functions for 2D shapes.
//!
//! This module provides Rust implementations of common 2D SDF functions
//! that correspond to the WGSL shaders used in bevy_smud for rendering.
//! These functions can be used for CPU-side calculations like picking,
//! collision detection, or other geometric computations.

use bevy::math::{Vec2, Vec3, Vec4};
use std::f32::consts::PI;

// Helpers, some of these have perfect implementations in rust std
// but we keep these for clarity and to 1-to-1 match with the WGSL versions

/// Helper function to calculate squared length of a 2D vector
fn dot2(p: Vec2) -> f32 {
    p.length_squared()
}

/// Helper function to clamp a value
fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.clamp(min, max)
}

/// Helper function to get sign of a value
fn sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Helper function for modulo operation
fn modulo(x: f32, y: f32) -> f32 {
    x - y * (x / y).floor()
}

// Distance functions

/// Signed distance to a circle
pub fn circle(p: Vec2, radius: f32) -> f32 {
    p.length() - radius
}

/// Signed distance to an annulus (ring)
pub fn annulus(p: Vec2, outer_radius: f32, inner_radius: f32) -> f32 {
    let middle_radius = (outer_radius + inner_radius) * 0.5;
    let thickness = (outer_radius - inner_radius) * 0.5;
    circle(p, middle_radius).abs() - thickness
}

/// Signed distance to a capsule (pill shape)
pub fn capsule(p: Vec2, radius: f32, half_length: f32) -> f32 {
    // Special case: when half_length is 0, the capsule is just a circle
    if half_length == 0.0 {
        return circle(p, radius);
    }
    let a = Vec2::new(0.0, -half_length);
    let b = Vec2::new(0.0, half_length);
    segment(p, a, b) - radius
}

/// Signed distance to a rounded box
pub fn rounded_box(p: Vec2, b: Vec2, r: f32) -> f32 {
    let q = p.abs() - b;
    q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - r
}

/// Signed distance to a box
pub fn sd_box(p: Vec2, b: Vec2) -> f32 {
    let d = p.abs() - b;
    d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0)
}

/// Signed distance to an oriented box
pub fn oriented_box(p: Vec2, a: Vec2, b: Vec2, th: f32) -> f32 {
    let l = (b - a).length();
    let d = (b - a) / l;
    let q = p - (a + b) * 0.5;
    let q_rot = Vec2::new(q.dot(d), -q.dot(d.perp()));
    rounded_box(q_rot, Vec2::new(l * 0.5, th), 0.0)
}

/// Signed distance to a line segment
pub fn segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let ba_len_sq = ba.dot(ba);

    // Degenerate case: when a == b, the segment is just a point
    if ba_len_sq == 0.0 {
        return pa.length();
    }

    let h = clamp(pa.dot(ba) / ba_len_sq, 0.0, 1.0);
    (pa - ba * h).length()
}

/// Signed distance to a rhombus
pub fn rhombus(p: Vec2, b: Vec2) -> f32 {
    let p_abs = p.abs();
    let h = clamp((-2.0 * p_abs.dot(b) + b.dot(b)) / b.dot(b), -1.0, 1.0);
    let d = (p_abs - 0.5 * b * Vec2::new(1.0 - h, 1.0 + h)).length();
    d * sign(p_abs.x * b.y + p_abs.y * b.x - b.x * b.y)
}

/// Signed distance to a trapezoid
pub fn trapezoid(p: Vec2, r1: f32, r2: f32, he: f32) -> f32 {
    let k1 = Vec2::new(r2, he);
    let k2 = Vec2::new(r2 - r1, 2.0 * he);
    let mut p = p;
    p.x = p.x.abs();
    let ca = Vec2::new(p.x - (if p.y < 0.0 { r1 } else { r2 }), p.y.abs() - he);
    let cb = Vec2::new(p.x - r2, p.y - he);
    let cb_clamp = cb - k1 * clamp(cb.dot(k1) / k1.dot(k1), 0.0, 1.0);
    let s = if cb.x * k2.y - cb.y * k2.x < 0.0 {
        -1.0
    } else {
        1.0
    };
    s * (dot2(ca).min(dot2(cb_clamp))).sqrt()
}

/// Signed distance to a parallelogram
pub fn parallelogram(p: Vec2, wi: f32, he: f32, sk: f32) -> f32 {
    let e = Vec2::new(sk, he);
    let mut p = p;
    p = Vec2::new(if p.y < 0.0 { -p.x } else { p.x }, p.y.abs());
    let w = p - e;
    let w_e = w - e * clamp(w.dot(e) / e.dot(e), 0.0, 1.0);
    let d = Vec2::new(w_e.x - wi * clamp(w_e.x / wi, 0.0, 1.0), w_e.y);
    let s = if p.x * he - p.y * wi > 0.0 { -1.0 } else { 1.0 };
    s * (d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0))
}

/// Signed distance to an equilateral triangle
pub fn equilateral_triangle(p: Vec2, r: f32) -> f32 {
    let k = (3.0_f32).sqrt();
    let mut p = p;
    p.x = p.x.abs() - r;
    p.y += r / k;
    if p.x + k * p.y > 0.0 {
        p = Vec2::new((p.x - k * p.y) * 0.5, (-k * p.x - p.y) * 0.5);
    }
    p.x -= clamp(p.x, -2.0 * r, 0.0);
    -p.length() * sign(p.y)
}

/// Signed distance to an isosceles triangle
pub fn triangle_isosceles(p: Vec2, q: Vec2) -> f32 {
    let mut p = p;
    p.x = p.x.abs();
    let a = p - q;
    let b = Vec2::new(p.x - q.x, p.y + q.y);
    let k = clamp(a.dot(q) / q.dot(q), 0.0, 1.0);
    if k < 0.0 {
        dot2(a)
    } else if k > 1.0 {
        dot2(b)
    } else {
        (a - q * k).length_squared()
    }
    .sqrt()
        * sign(a.x * q.y - a.y * q.x)
}

/// Signed distance to a triangle
pub fn triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> f32 {
    let e0 = b - a;
    let e1 = c - b;
    let e2 = a - c;
    let v0 = p - a;
    let v1 = p - b;
    let v2 = p - c;
    let pq0 = v0 - e0 * clamp(v0.dot(e0) / e0.dot(e0), 0.0, 1.0);
    let pq1 = v1 - e1 * clamp(v1.dot(e1) / e1.dot(e1), 0.0, 1.0);
    let pq2 = v2 - e2 * clamp(v2.dot(e2) / e2.dot(e2), 0.0, 1.0);
    let s = sign(e0.x * e2.y - e0.y * e2.x);
    let d = [
        (dot2(pq0), s * (v0.x * e0.y - v0.y * e0.x)),
        (dot2(pq1), s * (v1.x * e1.y - v1.y * e1.x)),
        (dot2(pq2), s * (v2.x * e2.y - v2.y * e2.x)),
    ];
    -d.iter()
        .map(|(dist, _)| *dist)
        .fold(f32::INFINITY, f32::min)
        .sqrt()
        * if d.iter().all(|(_, cross)| *cross > 0.0) {
            1.0
        } else {
            -1.0
        }
}

/// Signed distance to an uneven capsule
pub fn uneven_capsule(p: Vec2, r1: f32, r2: f32, h: f32) -> f32 {
    let mut p = p;
    p.x = p.x.abs();
    let b = (r1 - r2) / h;
    let a = (1.0 - b * b).sqrt();
    let k = p.dot(Vec2::new(-b, a));
    if k < 0.0 {
        p.length() - r1
    } else if k > a * h {
        (p - Vec2::new(0.0, h)).length() - r2
    } else {
        p.dot(Vec2::new(a, b)) - r1
    }
}

/// Signed distance to a pentagon
pub fn pentagon(p: Vec2, r: f32) -> f32 {
    let k = Vec3::new(0.809_017, 0.587_785_24, 0.726_542_53);
    let mut p = p;
    p.x = p.x.abs();
    p = p - 2.0 * f32::min(Vec2::new(-k.x, k.y).dot(p), 0.0) * Vec2::new(-k.x, k.y);
    p = p - 2.0 * f32::min(Vec2::new(k.x, k.y).dot(p), 0.0) * Vec2::new(k.x, k.y);
    p = Vec2::new(p.x - clamp(p.x, -r * k.z, r * k.z), p.y - r);
    p.length() * sign(p.y)
}

/// Signed distance to a hexagon
pub fn hexagon(p: Vec2, r: f32) -> f32 {
    let k = Vec3::new(-0.866_025_4, 0.5, 0.577_350_26);
    let mut p = p.abs();
    p = p - 2.0 * f32::min(Vec2::new(k.x, k.y).dot(p), 0.0) * Vec2::new(k.x, k.y);
    p = Vec2::new(p.x - clamp(p.x, -k.z * r, k.z * r), p.y - r);
    p.length() * sign(p.y)
}

/// Signed distance to an octagon
pub fn octagon(p: Vec2, r: f32) -> f32 {
    let k = Vec3::new(-0.923_879_5, 0.382_683_43, 0.414_213_57);
    let mut p = p.abs();
    p = p - 2.0 * f32::min(Vec2::new(k.x, k.y).dot(p), 0.0) * Vec2::new(k.x, k.y);
    p = p - 2.0 * f32::min(Vec2::new(-k.x, k.y).dot(p), 0.0) * Vec2::new(-k.x, k.y);
    p = Vec2::new(p.x - clamp(p.x, -k.z * r, k.z * r), p.y - r);
    p.length() * sign(p.y)
}

/// Signed distance to a hexagram (6-pointed star)
pub fn hexagram(p: Vec2, r: f32) -> f32 {
    let k = Vec4::new(-0.5, 0.866_025_4, 0.577_350_26, 1.732_050_8);
    let mut p = p.abs();
    p = p - 2.0 * f32::min(Vec2::new(k.x, k.y).dot(p), 0.0) * Vec2::new(k.x, k.y);
    p = p - 2.0 * f32::min(Vec2::new(k.y, k.x).dot(p), 0.0) * Vec2::new(k.y, k.x);
    p = Vec2::new(p.x - clamp(p.x, r * k.z, r * k.w), p.y - r);
    p.length() * sign(p.y)
}

/// Signed distance to a 5-pointed star
pub fn star_5(p: Vec2, r: f32, rf: f32) -> f32 {
    let k1 = Vec2::new(0.809_017, -0.587_785_24);
    let k2 = Vec2::new(-k1.x, k1.y);
    let mut p = p;
    p.x = p.x.abs();
    p = p - 2.0 * f32::max(k1.dot(p), 0.0) * k1;
    p = p - 2.0 * f32::max(k2.dot(p), 0.0) * k2;
    p.x = p.x.abs();
    p.y -= r;
    let ba = Vec2::new(rf * (-k1.y), rf * k1.x - 1.0);
    let h = clamp(p.dot(ba) / ba.dot(ba), 0.0, r);
    (p - ba * h).length() * sign(p.y * ba.x - p.x * ba.y)
}

/// Signed distance to a regular polygon
pub fn regular_polygon(p: Vec2, radius: f32, sides: i32) -> f32 {
    // Get polar angle
    let mut angle = p.y.atan2(p.x);
    // Add PI/2 to match Bevy's convention (vertex at top instead of right)
    angle += std::f32::consts::FRAC_PI_2;
    // Make angle to range [0, 2*PI]
    if angle < 0.0 {
        angle += std::f32::consts::TAU;
    }

    // Get each piece angle
    let delta = std::f32::consts::TAU / sides as f32;
    // How many pieces?
    let area_number = (angle / delta).floor();

    // Start angle of current piece
    let theta1 = delta * area_number;
    // End angle of current piece
    let theta2 = delta * (area_number + 1.0);

    // Start point on current piece
    let point_a = Vec2::new(radius * theta1.cos(), radius * theta1.sin());
    // End point on current piece
    let point_a_prime = Vec2::new(radius * theta2.cos(), radius * theta2.sin());
    // The middle of start and end point
    let point_d = (point_a + point_a_prime) / 2.0;

    // Area 1: near start vertex
    let vector1 = p - point_a;
    let axis1 = point_a;
    let a1 = (axis1.normalize().dot(vector1.normalize())).acos();
    if a1 < (delta / 2.0) {
        return vector1.length();
    }

    // Area 2: near end vertex
    let vector2 = p - point_a_prime;
    let axis2 = point_a_prime;
    let a2 = (axis2.normalize().dot(vector2.normalize())).acos();
    if (std::f32::consts::TAU - a2) < (delta / 2.0) {
        return vector2.length();
    }

    // Area 3: distance to edge
    let theta = modulo(angle, delta) - delta / 2.0;
    p.length() * theta.cos() - point_d.length()
}

/// Signed distance to a star with n points
pub fn star(p: Vec2, r: f32, n: i32, m: f32) -> f32 {
    let an = PI / n as f32;
    let en = PI / m;
    let acs = Vec2::new(an.cos(), an.sin());
    let ecs = Vec2::new(en.cos(), en.sin());

    let bn = modulo(p.y.atan2(p.x), 2.0 * an) - an;
    let mut p_star = Vec2::new(p.length() * bn.cos(), p.length() * bn.sin().abs());
    p_star -= r * acs;
    p_star = p_star + ecs * clamp(-p_star.dot(ecs), 0.0, r * acs.y / ecs.y);
    p_star.length() * sign(p_star.x)
}

/// Signed distance to a pie slice
pub fn pie(p: Vec2, c: Vec2, r: f32) -> f32 {
    let mut p = p;
    p.x = p.x.abs();
    let l = p.length() - r;
    let m = (p - c * clamp(p.dot(c), 0.0, r)).length();
    f32::max(l, m * sign(c.y * p.x - c.x * p.y))
}

/// Signed distance to a cut disk
pub fn cut_disk(p: Vec2, r: f32, h: f32) -> f32 {
    let w = (r * r - h * h).sqrt();
    let mut p = p;
    p.x = p.x.abs();
    let s = f32::max(
        (h - r) * p.x * p.x + w * w * (h + r - 2.0 * p.y),
        h * p.x - w * p.y,
    );
    if s < 0.0 {
        p.length() - r
    } else if p.x < w {
        h - p.y
    } else {
        (p - Vec2::new(w, h)).length()
    }
}

/// Signed distance to an arc
pub fn arc(p: Vec2, sc: Vec2, ra: f32, rb: f32) -> f32 {
    let mut p = p;
    p.x = p.x.abs();
    (if sc.y * p.x > sc.x * p.y {
        (p - sc * ra).length()
    } else {
        (p.length() - ra).abs()
    }) - rb
}

/// Signed distance to a horseshoe
pub fn horseshoe(p: Vec2, c: Vec2, r: f32, w: Vec2) -> f32 {
    let mut p = p;
    p.x = p.x.abs();
    let l = p.length();
    let p_rot = Vec2::new((-c.x) * p.x + c.y * p.y, c.y * p.x + c.x * p.y);
    let p_new = Vec2::new(
        if p_rot.y > 0.0 || p_rot.x > 0.0 {
            p_rot.x
        } else {
            l * sign(-c.x)
        },
        if p_rot.x > 0.0 { p_rot.y } else { l },
    );
    let p_final = Vec2::new(p_new.x, (p_new.y - r).abs() - w.y);
    let p_clamped = Vec2::new(p_final.x - w.x, p_final.y);
    p_clamped.max(Vec2::ZERO).length() + f32::min(0.0, f32::max(p_clamped.x, p_clamped.y))
}

/// Signed distance to a rounded cross
pub fn rounded_cross(p: Vec2, h: f32) -> f32 {
    let k = 0.5 * (h + 1.0 / h);
    let p_abs = p.abs();
    if p_abs.x < 1.0 && p_abs.y < p_abs.x * (k - h) + h {
        k - (dot2(p_abs - Vec2::new(1.0, k))).sqrt()
    } else {
        (f32::min(
            dot2(p_abs - Vec2::new(0.0, h)),
            dot2(p_abs - Vec2::new(1.0, 0.0)),
        ))
        .sqrt()
    }
}

/// Signed distance to an egg
pub fn egg(p: Vec2, ra: f32, rb: f32) -> f32 {
    let mut p = p;
    let k = (3.0_f32).sqrt();
    p.x = p.x.abs();
    let r = ra - rb;
    (if p.y < 0.0 {
        p.length() - r
    } else if k * (p.x + r) < p.y {
        (p - Vec2::new(0.0, k * r)).length()
    } else {
        (p - Vec2::new(-r, 0.0)).length() - 2.0 * r
    }) - rb
}

/// Signed distance to a heart
pub fn heart(p: Vec2) -> f32 {
    let mut p = p;
    p.x = p.x.abs();

    if p.y + p.x > 1.0 {
        return (dot2(p - Vec2::new(0.25, 0.75))).sqrt() - (2.0_f32).sqrt() / 4.0;
    }

    (f32::min(
        dot2(p - Vec2::new(0.0, 1.0)),
        dot2(Vec2::new(
            p.x - 0.5 * f32::max(p.x + p.y, 0.0),
            p.y - 0.5 * f32::max(p.x + p.y, 0.0),
        )),
    ))
    .sqrt()
        * sign(p.x - p.y)
}

/// Signed distance to a cross
pub fn cross(p: Vec2, b: Vec2, r: f32) -> f32 {
    let mut p = p.abs();
    p = if p.y > p.x { Vec2::new(p.y, p.x) } else { p };
    let q = p - b;
    let k = f32::max(q.y, q.x);
    let w = if k > 0.0 { q } else { Vec2::new(b.y - p.x, -k) };
    sign(k) * w.max(Vec2::ZERO).length() + r
}

/// Signed distance to a rounded X
pub fn rounded_x(p: Vec2, w: f32, r: f32) -> f32 {
    let p_abs = p.abs();
    (Vec2::new(
        p_abs.x - f32::min(p_abs.x + p_abs.y, w) * 0.5,
        p_abs.y - f32::min(p_abs.x + p_abs.y, w) * 0.5,
    ))
    .length()
        - r
}

/// Signed distance to an ellipse
pub fn ellipse(p: Vec2, a: f32, b: f32) -> f32 {
    let mut p = p.abs();
    let mut ab = Vec2::new(a, b);
    if p.x > p.y {
        p = Vec2::new(p.y, p.x);
        ab = Vec2::new(ab.y, ab.x);
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
    let co = if d < 0.0 {
        let h = (q / c3).acos() / 3.0;
        let s = h.cos();
        let t = h.sin() * (3.0_f32).sqrt();
        let rx = (-c * (s + t + 2.0) + m2).sqrt();
        let ry = (-c * (s - t + 2.0) + m2).sqrt();
        (ry + sign(l) * rx + g.abs() / (rx * ry) - m) / 2.0
    } else {
        let h = 2.0 * m * n * d.sqrt();
        let s = sign(q + h) * (q + h).abs().powf(1.0 / 3.0);
        let u = sign(q - h) * (q - h).abs().powf(1.0 / 3.0);
        let rx = -s - u - c * 4.0 + 2.0 * m2;
        let ry = (s - u) * (3.0_f32).sqrt();
        let rm = (rx * rx + ry * ry).sqrt();
        (ry / (rm - rx).sqrt() + 2.0 * g / rm - m) / 2.0
    };
    let r = Vec2::new(ab.x * co, ab.y * (1.0_f32 - co * co).sqrt());
    (r - p).length() * sign(p.y - r.y)
}

/// Signed distance to a parabola
pub fn parabola(p: Vec2, k: f32) -> f32 {
    let mut pos = p;
    pos.x = pos.x.abs();
    let ik = 1.0 / k;
    let p = ik * (pos.y - 0.5 * ik) / 3.0;
    let q = 0.25 * ik * ik * pos.x;
    let h = q * q - p * p * p;
    let r = h.abs().sqrt();
    let x = if h > 0.0 {
        (q + r).powf(1.0 / 3.0) - (q - r).abs().powf(1.0 / 3.0) * sign(r - q)
    } else {
        2.0 * ((r / q).atan() / 3.0).cos() * p.sqrt()
    };
    (pos - Vec2::new(x, k * x * x)).length() * sign(pos.x - x)
}

/// Signed distance to a parabola segment
pub fn parabola_segment(p: Vec2, wi: f32, he: f32) -> f32 {
    let mut pos = p;
    pos.x = pos.x.abs();
    let ik = wi * wi / he;
    let p = ik * (he - pos.y - 0.5 * ik) / 3.0;
    let q = pos.x * ik * ik * 0.25;
    let h = q * q - p * p * p;
    let r = h.abs().sqrt();
    let mut x = if h > 0.0 {
        (q + r).powf(1.0 / 3.0) - (q - r).abs().powf(1.0 / 3.0) * sign(r - q)
    } else {
        2.0 * ((r / q).atan() / 3.0).cos() * p.sqrt()
    };
    x = x.min(wi);
    (pos - Vec2::new(x, he - x * x / ik)).length() * sign(ik * (pos.y - he) + pos.x * pos.x)
}

/// Signed distance to a blobby cross
pub fn blobby_cross(p: Vec2, he: f32) -> f32 {
    let mut pos = p.abs();
    pos = Vec2::new(
        (pos.x - pos.y).abs(),
        (1.0 - pos.x - pos.y) / (2.0_f32).sqrt(),
    );

    let p = (he - pos.y - 0.25 / he) / (6.0 * he);
    let q = pos.x / (he * he * 16.0);
    let h = q * q - p * p * p;

    let x = if h > 0.0 {
        let r = h.sqrt();
        (q + r).powf(1.0 / 3.0) - (q - r).abs().powf(1.0 / 3.0) * sign(r - q)
    } else {
        let r = p.sqrt();
        2.0 * r * ((q / (p * r)).acos() / 3.0).cos()
    };
    let x = x.min((2.0_f32).sqrt() / 2.0);

    let z = Vec2::new(x, he * (1.0 - 2.0 * x * x) - pos.y);
    z.length() * sign(z.y)
}

/// Signed distance to a tunnel
pub fn tunnel(p: Vec2, wh: Vec2) -> f32 {
    let p_new = Vec2::new(p.x.abs(), -p.y);
    let mut q = p_new - wh;

    let d1 = dot2(Vec2::new(f32::max(q.x, 0.0), q.y));
    q.x = if p_new.y > 0.0 {
        p_new.length() - wh.x
    } else {
        q.x
    };
    let d2 = dot2(Vec2::new(q.x, f32::max(q.y, 0.0)));
    let d = (f32::min(d1, d2)).sqrt();

    if f32::max(q.x, q.y) < 0.0 { -d } else { d }
}

/// Signed distance to stairs
pub fn stairs(p: Vec2, wh: Vec2, n: f32) -> f32 {
    let mut p = p;
    let ba = wh * n;
    let mut d = f32::min(
        dot2(Vec2::new(p.x - clamp(p.x, 0.0, ba.x), p.y)),
        dot2(Vec2::new(p.x - ba.x, p.y - clamp(p.y, 0.0, ba.y))),
    );
    let mut s = sign(f32::max(-p.y, p.x - ba.x));

    let dia = wh.length();
    p = Vec2::new(
        (wh.x * p.x - wh.y * p.y) / dia,
        (wh.y * p.x + wh.x * p.y) / dia,
    );
    let id = clamp((p.x / dia).round(), 0.0, n - 1.0);
    p.x -= id * dia;
    p = Vec2::new(
        (wh.x * p.x + wh.y * p.y) / dia,
        (-wh.y * p.x + wh.x * p.y) / dia,
    );

    let hh = wh.y / 2.0;
    p.y -= hh;
    if p.y > hh * sign(p.x) {
        s = 1.0;
    }
    p = if id < 0.5 || p.x > 0.0 {
        p
    } else {
        Vec2::new(-p.x, -p.y)
    };
    d = f32::min(d, dot2(Vec2::new(p.x, p.y - clamp(p.y, -hh, hh))));
    d = f32::min(d, dot2(Vec2::new(p.x - clamp(p.x, 0.0, wh.x), p.y - hh)));

    d.sqrt() * s
}

/// Signed distance to a vesica (lens shape)
pub fn vesica(p: Vec2, r: f32, d: f32) -> f32 {
    let p_abs = p.abs();
    let b = (r * r - d * d).sqrt();
    if (p_abs.y - b) * d > p_abs.x * b {
        (p_abs - Vec2::new(0.0, b)).length()
    } else {
        (p_abs - Vec2::new(-d, 0.0)).length() - r
    }
}

/// Signed distance to a moon (crescent)
pub fn moon(p: Vec2, d: f32, ra: f32, rb: f32) -> f32 {
    let mut p = p;
    p.y = p.y.abs();
    let a = (ra * ra - rb * rb + d * d) / (2.0 * d);
    let b = (f32::max(ra * ra - a * a, 0.0)).sqrt();

    if d * (p.x * b - p.y * a) > d * d * f32::max(b - p.y, 0.0) {
        (p - Vec2::new(a, b)).length()
    } else {
        f32::max(p.length() - ra, -((p - Vec2::new(d, 0.0)).length() - rb))
    }
}

// Operations for combining SDF shapes

/// Union of two SDF shapes
pub fn op_union(d1: f32, d2: f32) -> f32 {
    d1.min(d2)
}

/// Subtraction of two SDF shapes
pub fn op_subtract(d1: f32, d2: f32) -> f32 {
    (-d1).max(d2)
}

/// Intersection of two SDF shapes
pub fn op_intersect(d1: f32, d2: f32) -> f32 {
    d1.max(d2)
}

/// Smooth union of two SDF shapes
pub fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h)
}

/// Smooth subtraction of two SDF shapes
pub fn op_smooth_subtract(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    d2 * (1.0 - h) + (-d1) * h + k * h * (1.0 - h)
}

/// Smooth intersection of two SDF shapes
pub fn op_smooth_intersect(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    d2 * (1.0 - h) + d1 * h + k * h * (1.0 - h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle() {
        // Point at center should be -radius
        assert_eq!(circle(Vec2::ZERO, 1.0), -1.0);

        // Point on circle should be 0
        assert!((circle(Vec2::new(1.0, 0.0), 1.0)).abs() < f32::EPSILON);

        // Point outside circle should be positive
        assert!(circle(Vec2::new(2.0, 0.0), 1.0) > 0.0);
    }

    #[test]
    fn test_segment_degenerate_case() {
        // When a == b, segment is just a point, should return distance to that point
        let a = Vec2::new(5.0, 5.0);
        let b = Vec2::new(5.0, 5.0);

        // Distance from origin to the point (5, 5)
        let result = segment(Vec2::ZERO, a, b);
        assert!(
            result.is_finite(),
            "segment should not return NaN for a == b"
        );
        let expected = (Vec2::new(5.0, 5.0) - Vec2::ZERO).length();
        assert!((result - expected).abs() < 0.001);

        // Distance from a point to itself should be 0
        let result = segment(Vec2::new(5.0, 5.0), a, b);
        assert!(result.is_finite());
        assert!(result.abs() < f32::EPSILON);
    }

    #[test]
    fn test_capsule_with_zero_half_length() {
        // When half_length is 0, capsule should behave like a circle
        let radius = 20.0;
        let half_length = 0.0;

        // Point at center should be -radius
        let result = capsule(Vec2::ZERO, radius, half_length);
        assert!(result.is_finite(), "capsule should not return NaN");
        assert_eq!(result, -radius);

        // Point on the circle edge
        let result = capsule(Vec2::new(radius, 0.0), radius, half_length);
        assert!(result.is_finite(), "capsule should not return NaN");
        assert!(result.abs() < 0.001);

        // Point outside
        let result = capsule(Vec2::new(30.0, 0.0), radius, half_length);
        assert!(result.is_finite(), "capsule should not return NaN");
        assert!(result > 0.0);
    }

    #[test]
    fn test_capsule_with_normal_half_length() {
        // Normal capsule with half_length > 0
        let radius = 10.0;
        let half_length = 10.0;

        // Point at center
        let result = capsule(Vec2::ZERO, radius, half_length);
        assert!(result.is_finite());
        assert_eq!(result, -radius);
    }
}
