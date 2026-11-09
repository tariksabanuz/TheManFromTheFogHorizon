// What the client and the bed check need, with no game types: how close he is to the camera, how
// thick the fog gets, and a plain distance test.

/// Blocks within which his fog starts to thicken.
pub const FOG_RANGE: f64 = 64.0;
/// Blocks within which nobody can sleep.
pub const SLEEP_RADIUS: f64 = 64.0;
/// The fog never closes in further than this many blocks, however near he is.
pub const FOG_MIN_END: f32 = 24.0;

/// 1 when he is at the camera, 0 at `range` blocks and beyond.
pub fn closeness(distance: f64, range: f64) -> f32 {
    if !distance.is_finite() || range <= 0.0 {
        return 0.0;
    }
    (1.0 - distance / range).clamp(0.0, 1.0) as f32
}

/// The fog's start and end after he thickens it: `closeness` times `alpha` (0 to 1) of the way
/// towards a fog that ends at [`FOG_MIN_END`] and starts at the camera. Never returns a start
/// beyond the end, or anything that is not finite; bad input comes back unchanged.
pub fn thicken(start: f32, end: f32, closeness: f32, alpha: f32) -> (f32, f32) {
    if !(start.is_finite() && end.is_finite() && closeness.is_finite() && alpha.is_finite()) || end <= 0.0 {
        return (start, end);
    }
    let k = (closeness.clamp(0.0, 1.0) * alpha.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let target_end = end.min(FOG_MIN_END);
    let new_end = end + (target_end - end) * k;
    let new_start = (start * (1.0 - k)).min(new_end * 0.9).max(0.0);
    (new_start, new_end)
}

pub fn within(a: [f64; 3], b: [f64; 3], radius: f64) -> bool {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    dx * dx + dy * dy + dz * dz <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closeness_runs_from_one_to_zero() {
        assert_eq!(closeness(0.0, 64.0), 1.0);
        assert_eq!(closeness(32.0, 64.0), 0.5);
        assert_eq!(closeness(64.0, 64.0), 0.0);
        assert_eq!(closeness(500.0, 64.0), 0.0);
        assert_eq!(closeness(f64::NAN, 64.0), 0.0);
        assert_eq!(closeness(5.0, 0.0), 0.0);
    }

    #[test]
    fn fog_closes_in_as_he_comes() {
        let (s0, e0) = thicken(100.0, 200.0, 0.0, 1.0);
        assert_eq!((s0, e0), (100.0, 200.0));
        let (s1, e1) = thicken(100.0, 200.0, 1.0, 1.0);
        assert_eq!(e1, FOG_MIN_END);
        assert!(s1 < e1);
        let (_, mid) = thicken(100.0, 200.0, 0.5, 1.0);
        assert!(mid > e1 && mid < 200.0);
    }

    #[test]
    fn alpha_scales_the_effect() {
        assert_eq!(thicken(100.0, 200.0, 1.0, 0.0), (100.0, 200.0));
        let (_, half) = thicken(100.0, 200.0, 1.0, 0.5);
        let (_, full) = thicken(100.0, 200.0, 1.0, 1.0);
        assert!(half > full && half < 200.0);
    }

    #[test]
    fn a_short_fog_is_left_alone_and_odd_input_is_safe() {
        let (s, e) = thicken(0.0, 10.0, 1.0, 1.0);
        assert_eq!(e, 10.0); // already thicker than the target
        assert!(s <= e);
        assert_eq!(thicken(f32::NAN, 100.0, 1.0, 1.0).1, 100.0);
        assert_eq!(thicken(10.0, 0.0, 1.0, 1.0), (10.0, 0.0));
        let (s, e) = thicken(500.0, 100.0, 1.0, 1.0);
        assert!(s <= e && s >= 0.0);
    }

    #[test]
    fn distance_test() {
        assert!(within([0.0; 3], [3.0, 4.0, 0.0], 5.0));
        assert!(!within([0.0; 3], [3.0, 4.0, 0.1], 5.0));
    }
}
