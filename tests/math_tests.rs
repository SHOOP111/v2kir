use aetherfall::math::*;
use macroquad::prelude::*;

#[test]
fn safe_normalize_handles_zero() {
    assert_eq!(safe_normalize(Vec2::ZERO), Vec2::ZERO);
}

#[test]
fn clamp_len_preserves_direction() {
    let v = Vec2::new(3.0, 4.0);
    let c = clamp_len(v, 2.0);
    assert!((c.length() - 2.0).abs() < 0.0001);
    assert!(c.normalize().distance(v.normalize()) < 0.0001);
}

#[test]
fn circle_collision_is_symmetric() {
    let a = vec2(0.0, 0.0);
    let b = vec2(5.0, 0.0);
    assert!(circle_hit(a, 3.0, b, 3.0));
    assert!(circle_hit(b, 3.0, a, 3.0));
    assert!(!circle_hit(a, 1.0, b, 1.0));
}

#[test]
fn hash_is_stable() {
    assert_eq!(hash2(4, 9, 12345), hash2(4, 9, 12345));
    assert_ne!(hash2(4, 9, 12345), hash2(5, 9, 12345));
}
