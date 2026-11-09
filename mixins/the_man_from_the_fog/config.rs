// The mod's choices: how often he comes, how he behaves, what he may break. Plain `key=value`
// lines in `./the-man-from-the-fog/config.txt` in the game folder; anything unknown or out of
// range is ignored or pulled back into range, so a hand-edited file can never break the game.

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Master switch. Off removes him at once.
    pub enabled: bool,
    /// Also appear by day (by default only at night).
    pub day_spawns: bool,
    /// Ticks between two appearances for one player, picked between these (20 ticks = 1 second).
    pub spawn_min: u32,
    pub spawn_max: u32,
    /// How far away he first stands, in blocks.
    pub distance_min: f64,
    pub distance_max: f64,
    /// Ticks he must go unwatched before he steps closer.
    pub approach_after: u32,
    /// Further ticks unwatched before he gives up watching and chases.
    pub chase_after: u32,
    /// How long a chase lasts, picked between these.
    pub chase_min: u32,
    pub chase_max: u32,
    /// Chance, in percent, that meeting his eyes makes him vanish instead of chase.
    pub vanish_chance: u32,
    /// He is gone once the player is this far away.
    pub despawn_distance: f64,
    /// Whether a chase breaks blocks in its way.
    pub break_blocks: bool,
    /// The hardest block he breaks (stone is 1.5, obsidian 50).
    pub break_hardness: f32,
    /// Ticks between two blocks he breaks.
    pub break_interval: u32,
    /// A bolt of lightning, no fire and no damage, as he appears and vanishes.
    pub lightning: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enabled: true,
            day_spawns: false,
            spawn_min: 12_000,
            spawn_max: 36_000,
            distance_min: 40.0,
            distance_max: 56.0,
            approach_after: 600,
            chase_after: 2_400,
            chase_min: 600,
            chase_max: 1_200,
            vanish_chance: 50,
            despawn_distance: 128.0,
            break_blocks: true,
            break_hardness: 3.0,
            break_interval: 8,
            lightning: true,
        }
    }
}

/// How often he comes: the choices of the page's button.
pub const RATES: [(&str, u32, u32); 4] = [
    ("rare", 24_000, 72_000),
    ("normal", 12_000, 36_000),
    ("often", 4_000, 12_000),
    ("constant", 600, 1_800),
];

pub const VANISH_STEPS: [u32; 5] = [0, 25, 50, 75, 100];
pub const HARDNESS_STEPS: [f32; 4] = [1.0, 3.0, 10.0, 50.0];

impl Config {
    /// Pulls every value into a range the engine can work with.
    pub fn clamp(&mut self) {
        self.spawn_min = self.spawn_min.clamp(200, 2_400_000);
        self.spawn_max = self.spawn_max.clamp(self.spawn_min, 4_800_000);
        self.distance_min = finite(self.distance_min, 40.0).clamp(12.0, 120.0);
        self.distance_max = finite(self.distance_max, 56.0).clamp(self.distance_min, 160.0);
        self.approach_after = self.approach_after.clamp(20, 72_000);
        self.chase_after = self.chase_after.clamp(20, 144_000);
        self.chase_min = self.chase_min.clamp(40, 72_000);
        self.chase_max = self.chase_max.clamp(self.chase_min, 144_000);
        self.vanish_chance = self.vanish_chance.min(100);
        self.despawn_distance = finite(self.despawn_distance, 128.0).clamp(self.distance_max + 8.0, 512.0);
        self.break_hardness = if self.break_hardness.is_finite() { self.break_hardness.clamp(0.0, 100.0) } else { 3.0 };
        self.break_interval = self.break_interval.clamp(1, 200);
    }

    /// The index in [`RATES`] this config matches, if any.
    pub fn rate_index(&self) -> Option<usize> {
        RATES.iter().position(|r| r.1 == self.spawn_min && r.2 == self.spawn_max)
    }

    pub fn set_rate(&mut self, index: usize) {
        let (_, lo, hi) = RATES[index % RATES.len()];
        self.spawn_min = lo;
        self.spawn_max = hi;
    }

    pub fn to_text(&self) -> String {
        format!(
            "enabled={}\nday_spawns={}\nspawn_min={}\nspawn_max={}\ndistance_min={}\ndistance_max={}\n\
             approach_after={}\nchase_after={}\nchase_min={}\nchase_max={}\nvanish_chance={}\n\
             despawn_distance={}\nbreak_blocks={}\nbreak_hardness={}\nbreak_interval={}\nlightning={}\n",
            self.enabled,
            self.day_spawns,
            self.spawn_min,
            self.spawn_max,
            self.distance_min,
            self.distance_max,
            self.approach_after,
            self.chase_after,
            self.chase_min,
            self.chase_max,
            self.vanish_chance,
            self.despawn_distance,
            self.break_blocks,
            self.break_hardness,
            self.break_interval,
            self.lightning,
        )
    }

    /// Reads `key=value` lines over the current values. Lines starting with `#` are comments.
    pub fn load_text(&mut self, text: &str) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "enabled" => set_bool(&mut self.enabled, value),
                "day_spawns" => set_bool(&mut self.day_spawns, value),
                "spawn_min" => set_num(&mut self.spawn_min, value),
                "spawn_max" => set_num(&mut self.spawn_max, value),
                "distance_min" => set_num(&mut self.distance_min, value),
                "distance_max" => set_num(&mut self.distance_max, value),
                "approach_after" => set_num(&mut self.approach_after, value),
                "chase_after" => set_num(&mut self.chase_after, value),
                "chase_min" => set_num(&mut self.chase_min, value),
                "chase_max" => set_num(&mut self.chase_max, value),
                "vanish_chance" => set_num(&mut self.vanish_chance, value),
                "despawn_distance" => set_num(&mut self.despawn_distance, value),
                "break_blocks" => set_bool(&mut self.break_blocks, value),
                "break_hardness" => set_num(&mut self.break_hardness, value),
                "break_interval" => set_num(&mut self.break_interval, value),
                "lightning" => set_bool(&mut self.lightning, value),
                _ => {}
            }
        }
        self.clamp();
    }
}

fn finite(v: f64, fallback: f64) -> f64 {
    if v.is_finite() { v } else { fallback }
}

fn set_bool(slot: &mut bool, value: &str) {
    match value {
        "true" | "on" | "yes" | "1" => *slot = true,
        "false" | "off" | "no" | "0" => *slot = false,
        _ => {}
    }
}

fn set_num<T: std::str::FromStr>(slot: &mut T, value: &str) {
    if let Ok(v) = value.parse() {
        *slot = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_in_range_already() {
        let mut c = Config::default();
        let before = c.clone();
        c.clamp();
        assert_eq!(c, before);
    }

    #[test]
    fn text_round_trip() {
        let mut c = Config::default();
        c.enabled = false;
        c.vanish_chance = 75;
        c.break_hardness = 10.0;
        c.spawn_min = 5_000;
        c.spawn_max = 9_000;
        let mut back = Config::default();
        back.load_text(&c.to_text());
        assert_eq!(back, c);
    }

    #[test]
    fn bad_input_is_ignored_or_clamped() {
        let mut c = Config::default();
        c.load_text("# comment\nenabled=maybe\nvanish_chance=900\nspawn_min=abc\nbreak_hardness=NaN\nnonsense\n=\nunknown=1\nbreak_interval=0\n");
        assert!(c.enabled);
        assert_eq!(c.vanish_chance, 100);
        assert_eq!(c.spawn_min, 12_000);
        assert_eq!(c.break_hardness, 3.0);
        assert_eq!(c.break_interval, 1);
    }

    #[test]
    fn ranges_stay_ordered() {
        let mut c = Config::default();
        c.load_text("spawn_min=100000\nspawn_max=500\ndistance_min=90\ndistance_max=20\nchase_min=5000\nchase_max=100\ndespawn_distance=10\n");
        assert!(c.spawn_min <= c.spawn_max);
        assert!(c.distance_min <= c.distance_max);
        assert!(c.chase_min <= c.chase_max);
        assert!(c.despawn_distance > c.distance_max);
    }

    #[test]
    fn rate_presets() {
        let mut c = Config::default();
        assert_eq!(c.rate_index(), Some(1));
        c.set_rate(2);
        assert_eq!(c.rate_index(), Some(2));
        c.set_rate(7); // wraps
        assert_eq!(c.rate_index(), Some(3));
        c.spawn_min = 777;
        assert_eq!(c.rate_index(), None);
    }
}
