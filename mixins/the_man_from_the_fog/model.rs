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
    Pink,
    Teeth,
}

impl Paint {
    /// The vanilla block sprite the faces use.
    pub fn sprite(self) -> &'static str {
        match self {
            Paint::Body => "block/black_concrete",
            Paint::Eye => "block/white_concrete",
            Paint::Mouth => "block/red_concrete",
            Paint::Pink => "block/pink_concrete",
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
    /// Degrees about the front axis; positive swings the lower end towards +x.
    pub splay: f32,
}

const fn body(name: &'static str, from: [f32; 3], to: [f32; 3], pivot: [f32; 3], motion: Motion) -> Part {
    Part { name, from, to, pivot, paint: Paint::Body, front_only: false, motion, splay: 0.0 }
}

const fn decal(name: &'static str, from: [f32; 3], to: [f32; 3], paint: Paint) -> Part {
    Part { name, from, to, pivot: [0.0, 0.0, 0.0], paint, front_only: true, motion: Motion::Still, splay: 0.0 }
}

const fn limb(name: &'static str, from: [f32; 3], to: [f32; 3], pivot: [f32; 3], motion: Motion, splay: f32) -> Part {
    Part { name, from, to, pivot, paint: Paint::Body, front_only: false, motion, splay }
}

/// Standing height in blocks.
pub const HEIGHT: f32 = 3.04;

const SWING_ARM_R: Motion = Motion::Swing { sign: -1.0, amount: 0.6 };
const SWING_ARM_L: Motion = Motion::Swing { sign: 1.0, amount: 0.6 };
const SHOULDER_R: [f32; 3] = [-0.3125, 2.27, 0.0];
const SHOULDER_L: [f32; 3] = [0.3125, 2.27, 0.0];

/// Measured from the Sketchfab model of the original (1 texel = 1/16 block; he stands 3 blocks):
/// a 5x6 head with two tiny white eyes near its top, an open mouth hanging below it (back wall, side
/// walls and a bottom lip) with a row of four red-to-white teeth hanging from the head and four
/// rising from the lip, the outer ones shorter above and longer below, a thin neck, an 8-wide torso,
/// legs that splay a little, and long arms (upper arm, forearm, hand) ending in four bent claws.
pub const PARTS: &[Part] = &[
    limb("leg_right", [-4.5, 0.0, -1.5], [-1.5, 20.4, 1.5], [-0.188, 1.275, 0.0], Motion::Swing { sign: 1.0, amount: 0.8 }, -4.5),
    limb("leg_left", [1.5, 0.0, -1.5], [4.5, 20.4, 1.5], [0.188, 1.275, 0.0], Motion::Swing { sign: -1.0, amount: 0.8 }, 4.5),
    limb("torso", [-4.0, 20.4, -2.0], [4.0, 37.4, 2.0], [0.0, 0.0, 0.0], Motion::Still, 0.0),
    limb("neck", [-2.2, 37.4, -1.5], [2.2, 42.4, 1.5], [0.0, 0.0, 0.0], Motion::Still, 0.0),
    limb("head", [-2.5, 42.4, -2.0], [2.5, 48.5, 3.0], [0.0, 0.0, 0.0], Motion::Still, 0.0),
    limb("mouth_back", [-2.5, 33.7, 1.5], [2.5, 42.4, 2.0], [0.0, 0.0, 0.0], Motion::Still, 0.0),
    limb("mouth_side_right", [-2.5, 33.7, 2.0], [-2.0, 42.4, 3.0], [0.0, 0.0, 0.0], Motion::Still, 0.0),
    limb("mouth_side_left", [2.0, 33.7, 2.0], [2.5, 42.4, 3.0], [0.0, 0.0, 0.0], Motion::Still, 0.0),
    limb("mouth_lip", [-2.5, 33.7, 2.0], [2.5, 34.7, 3.0], [0.0, 0.0, 0.0], Motion::Still, 0.0),
    limb("arm_right", [-6.0, 24.8, -1.2], [-4.0, 36.8, 1.2], SHOULDER_R, SWING_ARM_R, -6.0),
    limb("forearm_right", [-7.8, 10.3, -1.3], [-5.2, 24.8, 1.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("hand_right", [-7.8, 8.5, -1.5], [-5.2, 10.3, 1.5], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("claw_right_0a", [-7.8, 6.8, -0.3], [-7.3, 8.5, 0.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("claw_right_0b", [-9.1, 5.0, -0.3], [-8.6, 6.8, 0.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("claw_right_1a", [-7.1, 6.8, -0.3], [-6.6, 8.5, 0.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("claw_right_1b", [-7.8, 5.0, -0.3], [-7.3, 6.8, 0.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("claw_right_2a", [-6.4, 6.8, -0.3], [-5.9, 8.5, 0.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("claw_right_2b", [-6.4, 5.0, -0.3], [-5.9, 6.8, 0.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("claw_right_3a", [-5.7, 6.8, -0.3], [-5.2, 8.5, 0.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("claw_right_3b", [-5.2, 5.0, -0.3], [-4.7, 6.8, 0.3], SHOULDER_R, SWING_ARM_R, 0.0),
    limb("arm_left", [4.0, 24.8, -1.2], [6.0, 36.8, 1.2], SHOULDER_L, SWING_ARM_L, 6.0),
    limb("forearm_left", [5.2, 10.3, -1.3], [7.8, 24.8, 1.3], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("hand_left", [5.2, 8.5, -1.5], [7.8, 10.3, 1.5], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("claw_left_0a", [7.3, 6.8, -0.3], [7.8, 8.5, 0.3], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("claw_left_0b", [8.6, 5.0, -0.3], [9.1, 6.8, 0.3], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("claw_left_1a", [6.6, 6.8, -0.3], [7.1, 8.5, 0.3], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("claw_left_1b", [7.3, 5.0, -0.3], [7.8, 6.8, 0.3], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("claw_left_2a", [5.9, 6.8, -0.3], [6.4, 8.5, 0.3], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("claw_left_2b", [5.9, 5.0, -0.3], [6.4, 6.8, 0.3], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("claw_left_3a", [5.2, 6.8, -0.3], [5.7, 8.5, 0.3], SHOULDER_L, SWING_ARM_L, 0.0),
    limb("claw_left_3b", [4.7, 5.0, -0.3], [5.2, 6.8, 0.3], SHOULDER_L, SWING_ARM_L, 0.0),
    decal("eye_right", [-1.5, 46.4, 3.0], [-1.0, 47.0, 3.05], Paint::Eye),
    decal("eye_left", [1.0, 46.4, 3.0], [1.5, 47.0, 3.05], Paint::Eye),
    decal("tooth_up_0_red", [-2.2, 41.545, 3.0], [-1.7, 42.4, 3.1], Paint::Mouth),
    decal("tooth_up_0_pink", [-2.2, 41.07, 3.0], [-1.7, 41.545, 3.1], Paint::Pink),
    decal("tooth_up_0_white", [-2.2, 40.5, 3.0], [-1.7, 41.07, 3.1], Paint::Teeth),
    decal("tooth_up_1_red", [-1.2, 40.915, 3.0], [-0.7, 42.4, 3.1], Paint::Mouth),
    decal("tooth_up_1_pink", [-1.2, 40.09, 3.0], [-0.7, 40.915, 3.1], Paint::Pink),
    decal("tooth_up_1_white", [-1.2, 39.1, 3.0], [-0.7, 40.09, 3.1], Paint::Teeth),
    decal("tooth_up_2_red", [0.7, 40.915, 3.0], [1.2, 42.4, 3.1], Paint::Mouth),
    decal("tooth_up_2_pink", [0.7, 40.09, 3.0], [1.2, 40.915, 3.1], Paint::Pink),
    decal("tooth_up_2_white", [0.7, 39.1, 3.0], [1.2, 40.09, 3.1], Paint::Teeth),
    decal("tooth_up_3_red", [1.7, 41.545, 3.0], [2.2, 42.4, 3.1], Paint::Mouth),
    decal("tooth_up_3_pink", [1.7, 41.07, 3.0], [2.2, 41.545, 3.1], Paint::Pink),
    decal("tooth_up_3_white", [1.7, 40.5, 3.0], [2.2, 41.07, 3.1], Paint::Teeth),
    decal("tooth_low_0_red", [-2.2, 33.7, 3.0], [-1.7, 35.23, 3.1], Paint::Mouth),
    decal("tooth_low_0_pink", [-2.2, 35.23, 3.0], [-1.7, 36.08, 3.1], Paint::Pink),
    decal("tooth_low_0_white", [-2.2, 36.08, 3.0], [-1.7, 37.1, 3.1], Paint::Teeth),
    decal("tooth_low_1_red", [-1.2, 33.7, 3.0], [-0.7, 34.555, 3.1], Paint::Mouth),
    decal("tooth_low_1_pink", [-1.2, 34.555, 3.0], [-0.7, 35.03, 3.1], Paint::Pink),
    decal("tooth_low_1_white", [-1.2, 35.03, 3.0], [-0.7, 35.6, 3.1], Paint::Teeth),
    decal("tooth_low_2_red", [0.7, 33.7, 3.0], [1.2, 34.555, 3.1], Paint::Mouth),
    decal("tooth_low_2_pink", [0.7, 34.555, 3.0], [1.2, 35.03, 3.1], Paint::Pink),
    decal("tooth_low_2_white", [0.7, 35.03, 3.0], [1.2, 35.6, 3.1], Paint::Teeth),
    decal("tooth_low_3_red", [1.7, 33.7, 3.0], [2.2, 35.23, 3.1], Paint::Mouth),
    decal("tooth_low_3_pink", [1.7, 35.23, 3.0], [2.2, 36.08, 3.1], Paint::Pink),
    decal("tooth_low_3_white", [1.7, 36.08, 3.0], [2.2, 37.1, 3.1], Paint::Teeth),
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
            assert!(p.from[1] >= -1.0 && p.to[1] / 16.0 <= HEIGHT + 0.01, "{} is within his height", p.name);
        }
    }

    #[test]
    fn he_is_tall_and_thin() {
        let top = PARTS.iter().map(|p| p.to[1]).fold(0.0_f32, f32::max);
        assert!((top / 16.0 - HEIGHT).abs() < 0.1, "head top {}", top / 16.0);
        let widest = PARTS.iter().map(|p| p.to[0].max(-p.from[0])).fold(0.0_f32, f32::max);
        assert!(widest / 16.0 < 0.7, "well under a block wide in total ({widest})");
        // His arms reach lower than his hips.
        let arm = PARTS.iter().find(|p| p.name == "claw_left_0b").unwrap();
        let hip = PARTS.iter().find(|p| p.name == "leg_left").unwrap().pivot[1];
        assert!(arm.from[1] / 16.0 < hip);
    }

    #[test]
    fn decals_sit_on_the_front_of_what_they_mark() {
        let head = PARTS.iter().find(|p| p.name == "head").unwrap();
        for p in PARTS.iter().filter(|p| p.front_only) {
            assert!(p.from[0] >= head.from[0] && p.to[0] <= head.to[0], "{} within the head's width", p.name);
            assert!(p.from[2] >= head.to[2] - 0.01 && p.to[2] > head.to[2], "{} on the front", p.name);
            if p.name.starts_with("eye") {
                assert!(p.from[1] >= head.from[1] && p.to[1] <= head.to[1], "{} within the head", p.name);
            } else {
                // Teeth: from the mouth's floor up to the head's chin.
                assert!(p.from[1] >= 33.6 && p.to[1] <= head.from[1] + 0.01, "{} within the mouth", p.name);
            }
        }
    }

    #[test]
    fn he_has_four_claws_on_each_hand_and_two_rows_of_teeth() {
        let count = |prefix: &str| PARTS.iter().filter(|p| p.name.starts_with(prefix)).count();
        assert_eq!(count("claw_left"), 8);
        assert_eq!(count("claw_right"), 8);
        assert_eq!(count("eye_"), 2);
        assert_eq!(count("tooth_up_"), 12);
        assert_eq!(count("tooth_low_"), 12);
        assert!(PARTS.iter().any(|p| p.paint == Paint::Teeth));
        assert!(PARTS.iter().any(|p| p.paint == Paint::Mouth));
        assert!(PARTS.iter().any(|p| p.paint == Paint::Pink));
    }

    #[test]
    fn only_the_legs_reach_the_ground() {
        for p in PARTS.iter().filter(|p| p.from[1] < 1.0) {
            assert!(p.name.starts_with("leg"), "{}", p.name);
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
