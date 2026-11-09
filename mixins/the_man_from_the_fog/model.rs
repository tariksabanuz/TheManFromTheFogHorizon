// The Man From The Fog's body, as data: a tall, thin, near-black figure with long arms that
// reach below the knees and a glowing red-and-white mouth across the chest. The client draws
// these boxes under the mob's pose; there is no texture file to ship (the faces are plain
// vanilla block sprites), so the model is entirely the mod's own.
//
// Boxes are in texels (16 to the block, as a block model's elements), x to the mob's left, y up,
// z forward. Pivots are in blocks.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paint {
    Body,
    Eye,
    Mouth,
    Teeth,
}

impl Paint {
    /// The vanilla block sprite the faces use.
    pub fn sprite(self) -> &'static str {
        match self {
            Paint::Body => "block/black_concrete",
            Paint::Eye => "block/white_concrete",
            Paint::Mouth => "block/red_concrete",
            Paint::Teeth => "block/white_concrete",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    Still,
    /// Swings about x with the walk; `sign` picks the phase (+1, -1), `amount` scales it.
    Swing { sign: f32, amount: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub name: &'static str,
    pub from: [f32; 3],
    pub to: [f32; 3],
    /// Where it swings from, in blocks.
    pub pivot: [f32; 3],
    pub paint: Paint,
    /// Only the face towards the front is drawn (eyes and mouth are decals).
    pub front_only: bool,
    pub motion: Motion,
}

const fn body(name: &'static str, from: [f32; 3], to: [f32; 3], pivot: [f32; 3], motion: Motion) -> Part {
    Part { name, from, to, pivot, paint: Paint::Body, front_only: false, motion }
}

const fn decal(name: &'static str, from: [f32; 3], to: [f32; 3], paint: Paint) -> Part {
    Part { name, from, to, pivot: [0.0, 0.0, 0.0], paint, front_only: true, motion: Motion::Still }
}

/// Standing height in blocks.
pub const HEIGHT: f32 = 3.0;

const SWING_ARM_R: Motion = Motion::Swing { sign: -1.0, amount: 0.6 };
const SWING_ARM_L: Motion = Motion::Swing { sign: 1.0, amount: 0.6 };
const SHOULDER_R: [f32; 3] = [-0.28, 2.3, 0.0];
const SHOULDER_L: [f32; 3] = [0.28, 2.3, 0.0];

/// A bar of the mouth on the chest: red, with a white tooth at its lower end.
const fn bar(name: &'static str, x0: f32, x1: f32, y0: f32, y1: f32, paint: Paint) -> Part {
    decal(name, [x0, y0, 2.0], [x1, y1, 2.05], paint)
}

const fn tooth(name: &'static str, x0: f32, x1: f32, y0: f32, y1: f32) -> Part {
    decal(name, [x0, y0, 2.05], [x1, y1, 2.1], Paint::Teeth)
}

/// The photos show: a small square head with two white eyes, a thin torso, arms that hang past the
/// knees ending in three claws, and across the chest a mouth made of red bars with white teeth,
/// a row of four under the head and a smile of two side bars and a bottom bar below it.
pub const PARTS: &[Part] = &[
    // Legs: thin and long, hip at 1.5 blocks.
    body("leg_right", [-3.0, 0.0, -1.0], [-1.0, 24.0, 1.0], [-0.125, 1.5, 0.0], Motion::Swing { sign: 1.0, amount: 0.8 }),
    body("leg_left", [1.0, 0.0, -1.0], [3.0, 24.0, 1.0], [0.125, 1.5, 0.0], Motion::Swing { sign: -1.0, amount: 0.8 }),
    body("torso", [-3.5, 24.0, -2.0], [3.5, 38.0, 2.0], [0.0, 0.0, 0.0], Motion::Still),
    body("neck", [-1.0, 38.0, -1.0], [1.0, 40.0, 1.0], [0.0, 0.0, 0.0], Motion::Still),
    body("head", [-3.0, 40.0, -3.0], [3.0, 47.0, 3.0], [0.0, 0.0, 0.0], Motion::Still),
    // Arms hang past the knees, shoulder at 2.3 blocks.
    body("arm_right", [-5.5, 6.0, -1.0], [-3.5, 36.0, 1.0], SHOULDER_R, SWING_ARM_R),
    body("arm_left", [3.5, 6.0, -1.0], [5.5, 36.0, 1.0], SHOULDER_L, SWING_ARM_L),
    // Three claws on each hand.
    body("finger_right_0", [-5.5, 0.0, -0.6], [-4.9, 6.0, 0.6], SHOULDER_R, SWING_ARM_R),
    body("finger_right_1", [-4.8, 0.0, -0.6], [-4.2, 6.0, 0.6], SHOULDER_R, SWING_ARM_R),
    body("finger_right_2", [-4.1, 0.0, -0.6], [-3.5, 6.0, 0.6], SHOULDER_R, SWING_ARM_R),
    body("finger_left_0", [3.5, 0.0, -0.6], [4.1, 6.0, 0.6], SHOULDER_L, SWING_ARM_L),
    body("finger_left_1", [4.2, 0.0, -0.6], [4.8, 6.0, 0.6], SHOULDER_L, SWING_ARM_L),
    body("finger_left_2", [4.9, 0.0, -0.6], [5.5, 6.0, 0.6], SHOULDER_L, SWING_ARM_L),
    // Two small white eyes.
    decal("eye_right", [-2.0, 43.0, 3.0], [-1.0, 44.0, 3.05], Paint::Eye),
    decal("eye_left", [1.0, 43.0, 3.0], [2.0, 44.0, 3.05], Paint::Eye),
    // The upper row of the mouth: four red bars with a white tooth at the lower end.
    bar("mouth_0", -2.4, -1.6, 32.0, 37.0, Paint::Mouth),
    bar("mouth_1", -0.9, -0.1, 32.0, 37.0, Paint::Mouth),
    bar("mouth_2", 0.1, 0.9, 32.0, 37.0, Paint::Mouth),
    bar("mouth_3", 1.6, 2.4, 32.0, 37.0, Paint::Mouth),
    tooth("tooth_0", -2.4, -1.6, 32.0, 33.5),
    tooth("tooth_1", -0.9, -0.1, 32.0, 33.5),
    tooth("tooth_2", 0.1, 0.9, 32.0, 33.5),
    tooth("tooth_3", 1.6, 2.4, 32.0, 33.5),
    // The smile below it: a bar at each side and one along the bottom, with white teeth.
    bar("smile_left", -2.4, -1.6, 27.0, 30.5, Paint::Mouth),
    bar("smile_right", 1.6, 2.4, 27.0, 30.5, Paint::Mouth),
    bar("smile_bottom", -1.6, 1.6, 27.0, 28.5, Paint::Mouth),
    tooth("smile_tooth_left", -2.4, -1.6, 29.0, 30.5),
    tooth("smile_tooth_right", 1.6, 2.4, 29.0, 30.5),
    tooth("smile_tooth_mid", -0.4, 0.4, 27.0, 28.5),
];

/// Degrees a part swings at this moment of the walk cycle and its idle sway.
///
/// `walk_position` and `walk_speed` are the mob's walk cycle (a vanilla quadruped swings its legs
/// `cos(position * 0.6662) * 1.4 * speed` radians); `age` drives a slow sway of the arms.
pub fn swing_degrees(motion: Motion, walk_position: f32, walk_speed: f32, age: f32) -> f32 {
    match motion {
        Motion::Still => 0.0,
        Motion::Swing { sign, amount } => {
            let walk = (walk_position * 0.6662).cos() * 1.4 * walk_speed.clamp(0.0, 1.5) * amount;
            let sway = (age * 0.05 + sign).sin() * 2.0 * (1.0 - walk_speed.clamp(0.0, 1.0));
            (sign * walk).to_degrees() + sway
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boxes_are_well_formed() {
        for p in PARTS.iter() {
            for i in 0..3 {
                assert!(p.from[i] < p.to[i], "{} axis {i}", p.name);
            }
            assert!(p.from[1] >= 0.0 && p.to[1] / 16.0 <= HEIGHT + 0.01, "{} is within his height", p.name);
        }
    }

    #[test]
    fn he_is_tall_and_thin() {
        let top = PARTS.iter().map(|p| p.to[1]).fold(0.0_f32, f32::max);
        assert!((top / 16.0 - HEIGHT).abs() < 0.1, "head top {}", top / 16.0);
        let widest = PARTS.iter().map(|p| p.to[0].max(-p.from[0])).fold(0.0_f32, f32::max);
        assert!(widest / 16.0 < 0.5, "less than a block wide in total ({widest})");
        // His arms reach lower than his hips.
        let arm = PARTS.iter().find(|p| p.name == "finger_left_0").unwrap();
        let hip = PARTS.iter().find(|p| p.name == "leg_left").unwrap().pivot[1];
        assert!(arm.from[1] / 16.0 < hip);
    }

    #[test]
    fn decals_sit_on_the_front_of_what_they_mark() {
        let torso = PARTS.iter().find(|p| p.name == "torso").unwrap();
        let head = PARTS.iter().find(|p| p.name == "head").unwrap();
        for p in PARTS.iter().filter(|p| p.front_only) {
            let host = if p.name.starts_with("eye") { head } else { torso };
            assert!(p.from[2] >= host.to[2] - 1.1 && p.to[2] > host.to[2] - 0.01, "{} is on the front face", p.name);
            assert!(p.from[0] >= host.from[0] && p.to[0] <= host.to[0], "{} within the width", p.name);
            assert!(p.from[1] >= host.from[1] && p.to[1] <= host.to[1], "{} within the height", p.name);
        }
    }

    #[test]
    fn he_has_three_claws_on_each_hand_and_a_toothy_mouth() {
        let count = |prefix: &str| PARTS.iter().filter(|p| p.name.starts_with(prefix)).count();
        assert_eq!(count("finger_left"), 3);
        assert_eq!(count("finger_right"), 3);
        assert_eq!(count("eye_"), 2);
        assert_eq!(count("mouth_"), 4);
        assert!(PARTS.iter().any(|p| p.paint == Paint::Teeth));
        assert!(PARTS.iter().any(|p| p.paint == Paint::Mouth));
    }

    #[test]
    fn claws_are_the_only_things_at_the_ground_besides_the_legs() {
        for p in PARTS.iter().filter(|p| p.from[1] < 1.0) {
            assert!(p.name.starts_with("finger") || p.name.starts_with("leg"), "{}", p.name);
        }
    }

    #[test]
    fn legs_swing_in_opposite_phase() {
        let l = swing_degrees(Motion::Swing { sign: 1.0, amount: 0.8 }, 3.0, 1.0, 0.0);
        let r = swing_degrees(Motion::Swing { sign: -1.0, amount: 0.8 }, 3.0, 1.0, 0.0);
        assert!((l + r).abs() < 3.0, "{l} {r}");
        assert!(l.abs() > 10.0);
    }

    #[test]
    fn standing_still_only_sways() {
        let m = Motion::Swing { sign: 1.0, amount: 0.6 };
        for age in [0.0, 10.0, 77.0, 1234.5] {
            assert!(swing_degrees(m, 0.0, 0.0, age).abs() <= 2.0001);
        }
        assert_eq!(swing_degrees(Motion::Still, 1.0, 1.0, 1.0), 0.0);
    }

    #[test]
    fn no_nans_for_odd_input() {
        let m = Motion::Swing { sign: 1.0, amount: 0.6 };
        assert!(swing_degrees(m, 1e6, 100.0, -5.0).is_finite());
        assert!(swing_degrees(m, -3.0, -1.0, 1e9).is_finite());
    }
}
