// What the mod calls itself, and the tag the server puts on the mob that is him so that the
// client draws the right model.

pub const ID: &str = "the_man_from_the_fog";
pub const NAME: &str = "The Man From The Fog";
pub const VERSION: &str = "0.1.10";
pub const AUTHOR_LINE: &str = "By tariksabanuz";
pub const DESCRIPTION: &str = "A stalking horror mob. He watches from far away in the dark, steps closer when you ignore him, and when you meet his eyes he either vanishes or hunts you. Open this page to set how often he comes and what he may break. By tariksabanuz";

/// The custom name of the mob that is him (never shown: its name tag is off).
pub const MOB_TAG: &str = "the_man_from_the_fog";
/// The vanilla mob his body is made from.
pub const BASE_KIND: &str = "minecraft:husk";

/// Where the config lives, in the game folder.
pub const DIR: &str = "the-man-from-the-fog";

/// His body's stats: 300 health and 15 damage as the original's 1.0.0 changelog gives, fast, and
/// hard to knock back.
pub const HEALTH: f32 = 300.0;
pub const DAMAGE: f32 = 15.0;
pub const SPEED: f64 = 0.35;
pub const FOLLOW_RANGE: f64 = 128.0;
pub const KNOCKBACK_RESISTANCE: f64 = 0.8;
