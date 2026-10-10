use macroquad::prelude::*;

pub fn clamp_len(v: Vec2, max: f32) -> Vec2 {
    let len2 = v.length_squared();
    if len2 > max * max && len2 > 0.00001 {
        v / len2.sqrt() * max
    } else {
        v
    }
}

pub fn safe_normalize(v: Vec2) -> Vec2 {
    let len = v.length();
    if len > 0.00001 {
        v / len
    } else {
        Vec2::ZERO
    }
}

pub fn approach(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else {
        current + (target - current).signum() * max_delta
    }
}

pub fn damp(current: Vec2, target: Vec2, rate: f32, dt: f32) -> Vec2 {
    let t = 1.0 - (-rate * dt).exp();
    current.lerp(target, t)
}

pub fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

pub fn angle_of(v: Vec2) -> f32 {
    v.y.atan2(v.x)
}

pub fn from_angle(a: f32) -> Vec2 {
    Vec2::new(a.cos(), a.sin())
}

pub fn circle_hit(a: Vec2, ar: f32, b: Vec2, br: f32) -> bool {
    a.distance_squared(b) <= (ar + br) * (ar + br)
}

pub fn circle_segment_hit(center: Vec2, radius: f32, start: Vec2, end: Vec2) -> bool {
    let d = end - start;
    let len2 = d.length_squared();
    let t = if len2 > 0.00001 { ((center - start).dot(d) / len2).clamp(0.0, 1.0) } else { 0.0 };
    let closest = start + d * t;
    center.distance_squared(closest) <= radius * radius
}

pub fn hash2(mut x: i32, mut y: i32, seed: u64) -> u32 {
    x = x.wrapping_mul(0x27d4eb2d);
    y = y.wrapping_mul(0x165667b1);
    let mut h = seed as u32 ^ x as u32 ^ y as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^ (h >> 16)
}

pub fn hash01(x: i32, y: i32, seed: u64) -> f32 {
    (hash2(x, y, seed) as f32) / (u32::MAX as f32)
}

pub fn quantize(v: f32, step: f32) -> i32 {
    (v / step).floor() as i32
}

pub fn wrap_angle(mut a: f32) -> f32 {
    while a > std::f32::consts::PI {
        a -= std::f32::consts::PI * 2.0;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::PI * 2.0;
    }
    a
}

pub fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    a + wrap_angle(b - a) * t.clamp(0.0, 1.0)
}
