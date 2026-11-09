// The mod's choices, named after the options the original mod's config page lists (spawn_rate,
// min_chase_duration, max_chase_duration, vanish_distance, spawn_at_day, darkness_effect,
// all_dimensions, break_blocks, climbing, search_mechanic, block_hardness, block_hardness_search,
// enable_fog, fog_alpha_value), plus a few of this port's own. Plain `key=value` lines in
// `./the-man-from-the-fog/config.txt` in the game folder; anything unknown or out of range is
// ignored or pulled back into range, so a hand-edited file can never break the game.
//
// Where the original's default is published it is used; the rest are this port's own choices
// (marked below).

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Master switch. Off removes him at once.
    pub enabled: bool,
    /// `spawn_at_day`: also appear by day (default: only at night).
    pub spawn_at_day: bool,
    /// `all_dimensions`: appear outside the overworld too.
    pub all_dimensions: bool,
    /// `spawn_rate`: ticks between two appearances for one player, picked between these (20 ticks
    /// = 1 second). `rare`, `normal` and `often` are the original's three choices; the numbers are ours.
    pub spawn_min: u32,
    pub spawn_max: u32,
    /// How far away he first stands, in blocks (ours).
    pub distance_min: f64,
    pub distance_max: f64,
    /// Ticks he must go unwatched before he steps closer: about 30 seconds.
    pub approach_after: u32,
    /// Further ticks unwatched before he gives up watching and hunts (about 45 seconds in all).
    pub chase_after: u32,
    /// `min_chase_duration` / `max_chase_duration`: how long a hunt lasts (original: 400 and 1000).
    pub chase_min: u32,
    pub chase_max: u32,
    /// Chance, in percent, that meeting his eyes makes him vanish instead of hunt (ours).
    pub vanish_chance: u32,
    /// `vanish_distance`: a hunter that has lost the player and is this far away is gone (original: 30).
    pub vanish_distance: f64,
    /// He is gone once a player who only watches is this far away (ours).
    pub despawn_distance: f64,
    /// `break_blocks`: whether a hunt breaks blocks in its way.
    pub break_blocks: bool,
    /// `block_hardness`: the hardest block he breaks while he sees you (original: 2, dirt, cobblestone, wood).
    pub block_hardness: f32,
    /// `search_mechanic`: after losing sight of you he searches for the nearest player and digs through walls.
    pub search_mechanic: bool,
    /// `block_hardness_search`: the hardest block he breaks while searching (ours: 3.5, chests and doors fall).
    pub block_hardness_search: f32,
    /// Ticks between two blocks he breaks (ours).
    pub break_interval: u32,
    /// `climbing`: he climbs walls while hunting.
    pub climbing: bool,
    /// `jumpscare`: a scare (loud scream, brief darkness) when he reaches you in a hunt. The original
    /// shows a full-screen image on the killing blow; this port cannot draw one.
    pub jumpscare: bool,
    /// `darkness_effect`: a hunt starts with the Darkness effect on the player.
    pub darkness_effect: bool,
    /// `enable_fog`: the fog thickens as he comes close.
    pub enable_fog: bool,
    /// `fog_alpha_value`: how thick, 0 to 1.
    pub fog_alpha_value: f32,
    /// A harmless bolt of lightning as he appears and vanishes.
    pub lightning: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enabled: true,
            spawn_at_day: false,
            all_dimensions: false,
            spawn_min: 12_000,
            spawn_max: 36_000,
            distance_min: 40.0,
            distance_max: 56.0,
            approach_after: 600,
            chase_after: 300,
            chase_min: 400,
            chase_max: 1_000,
            vanish_chance: 50,
            vanish_distance: 30.0,
            despawn_distance: 128.0,
            break_blocks: true,
            block_hardness: 2.0,
            search_mechanic: true,
            block_hardness_search: 3.5,
            break_interval: 8,
            climbing: true,
            darkness_effect: true,
            jumpscare: true,
            enable_fog: true,
            fog_alpha_value: 0.6,
            lightning: true,
        }
    }
}

/// `spawn_rate`: the original's three choices.
pub const RATES: [(&str, u32, u32); 3] = [("rare", 24_000, 72_000), ("normal", 12_000, 36_000), ("often", 4_000, 12_000)];

pub const VANISH_STEPS: [u32; 5] = [0, 25, 50, 75, 100];
pub const HARDNESS_STEPS: [f32; 5] = [0.5, 2.0, 3.5, 10.0, 50.0];
pub const FOG_STEPS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

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
        self.vanish_distance = finite(self.vanish_distance, 30.0).clamp(8.0, 200.0);
        self.despawn_distance = finite(self.despawn_distance, 128.0).clamp(self.distance_max + 8.0, 512.0);
        self.block_hardness = hardness(self.block_hardness, 2.0);
        self.block_hardness_search = hardness(self.block_hardness_search, 3.5);
        self.break_interval = self.break_interval.clamp(1, 200);
        self.fog_alpha_value = if self.fog_alpha_value.is_finite() { self.fog_alpha_value.clamp(0.0, 1.0) } else { 0.6 };
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
        let rate = self.rate_index().map_or("custom", |i| RATES[i].0);
        format!(
            "enabled={}\nspawn_rate={}\nspawn_min={}\nspawn_max={}\nspawn_at_day={}\nall_dimensions={}\n\
             distance_min={}\ndistance_max={}\napproach_after={}\nchase_after={}\n\
             min_chase_duration={}\nmax_chase_duration={}\nvanish_chance={}\nvanish_distance={}\n\
             despawn_distance={}\nbreak_blocks={}\nblock_hardness={}\nsearch_mechanic={}\n\
             block_hardness_search={}\nbreak_interval={}\nclimbing={}\ndarkness_effect={}\n\
             enable_fog={}\nfog_alpha_value={}\nlightning={}\njumpscare={}\n",
            self.enabled,
            rate,
            self.spawn_min,
            self.spawn_max,
            self.spawn_at_day,
            self.all_dimensions,
            self.distance_min,
            self.distance_max,
            self.approach_after,
            self.chase_after,
            self.chase_min,
            self.chase_max,
            self.vanish_chance,
            self.vanish_distance,
            self.despawn_distance,
            self.break_blocks,
            self.block_hardness,
            self.search_mechanic,
            self.block_hardness_search,
            self.break_interval,
            self.climbing,
            self.darkness_effect,
            self.enable_fog,
            self.fog_alpha_value,
            self.lightning,
            self.jumpscare,
        )
    }

    /// Reads `key=value` lines over the current values. Lines starting with `#` are comments.
    /// `spawn_rate` (rare, normal, often) is read before `spawn_min`/`spawn_max` so the numbers win.
    pub fn load_text(&mut self, text: &str) {
        let mut lines: Vec<(&str, &str)> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter_map(|l| l.split_once('='))
            .map(|(k, v)| (k.trim(), v.trim()))
            .collect();
        lines.sort_by_key(|(k, _)| *k != "spawn_rate");
        for (key, value) in lines {
            match key {
                "enabled" => set_bool(&mut self.enabled, value),
                "spawn_rate" => {
                    if let Some(i) = RATES.iter().position(|r| r.0 == value) {
                        self.set_rate(i);
                    }
                }
                "spawn_at_day" | "day_spawns" => set_bool(&mut self.spawn_at_day, value),
                "all_dimensions" => set_bool(&mut self.all_dimensions, value),
                "spawn_min" => set_num(&mut self.spawn_min, value),
                "spawn_max" => set_num(&mut self.spawn_max, value),
                "distance_min" => set_num(&mut self.distance_min, value),
                "distance_max" => set_num(&mut self.distance_max, value),
                "approach_after" => set_num(&mut self.approach_after, value),
                "chase_after" => set_num(&mut self.chase_after, value),
                "min_chase_duration" => set_num(&mut self.chase_min, value),
                "max_chase_duration" => set_num(&mut self.chase_max, value),
                "vanish_chance" => set_num(&mut self.vanish_chance, value),
                "vanish_distance" => set_num(&mut self.vanish_distance, value),
                "despawn_distance" => set_num(&mut self.despawn_distance, value),
                "break_blocks" => set_bool(&mut self.break_blocks, value),
                "block_hardness" => set_num(&mut self.block_hardness, value),
                "search_mechanic" => set_bool(&mut self.search_mechanic, value),
                "block_hardness_search" => set_num(&mut self.block_hardness_search, value),
                "break_interval" => set_num(&mut self.break_interval, value),
                "climbing" => set_bool(&mut self.climbing, value),
                "darkness_effect" => set_bool(&mut self.darkness_effect, value),
                "enable_fog" => set_bool(&mut self.enable_fog, value),
                "fog_alpha_value" => set_num(&mut self.fog_alpha_value, value),
                "lightning" => set_bool(&mut self.lightning, value),
                "jumpscare" => set_bool(&mut self.jumpscare, value),
                _ => {}
            }
        }
        self.clamp();
    }
}

fn finite(v: f64, fallback: f64) -> f64 {
    if v.is_finite() { v } else { fallback }
}

fn hardness(v: f32, fallback: f32) -> f32 {
    if v.is_finite() { v.clamp(0.0, 100.0) } else { fallback }
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
    fn defaults_follow_the_published_original() {
        let c = Config::default();
        assert_eq!((c.chase_min, c.chase_max), (400, 1_000));
        assert_eq!(c.vanish_distance, 30.0);
        assert!(!c.spawn_at_day && !c.all_dimensions);
        assert!(c.jumpscare);
        assert!(c.break_blocks && c.climbing && c.darkness_effect && c.search_mechanic && c.enable_fog);
        assert_eq!(c.block_hardness, 2.0);
        assert_eq!(c.rate_index(), Some(1)); // normal
        // 30 s unwatched to step closer, 45 s in all to hunt.
        assert_eq!(c.approach_after, 600);
        assert_eq!(c.approach_after + c.chase_after, 900);
    }

    #[test]
    fn text_round_trip() {
        let mut c = Config::default();
        c.enabled = false;
        c.vanish_chance = 75;
        c.block_hardness = 10.0;
        c.spawn_min = 5_000;
        c.spawn_max = 9_000;
        c.climbing = false;
        c.fog_alpha_value = 0.25;
        let mut back = Config::default();
        back.load_text(&c.to_text());
        assert_eq!(back, c);
    }

    #[test]
    fn spawn_rate_names_are_read() {
        let mut c = Config::default();
        c.load_text("spawn_rate=often\n");
        assert_eq!(c.rate_index(), Some(2));
        c.load_text("spawn_rate=rare\n");
        assert_eq!(c.rate_index(), Some(0));
        // Explicit numbers beat the name, whatever the line order.
        c.load_text("spawn_min=1000\nspawn_max=2000\nspawn_rate=normal\n");
        assert_eq!((c.spawn_min, c.spawn_max), (1000, 2000));
        c.load_text("spawn_rate=nonsense\n");
        assert_eq!((c.spawn_min, c.spawn_max), (1000, 2000));
    }

    #[test]
    fn bad_input_is_ignored_or_clamped() {
        let mut c = Config::default();
        c.load_text("# comment\nenabled=maybe\nvanish_chance=900\nspawn_min=abc\nblock_hardness=NaN\nnonsense\n=\nunknown=1\nbreak_interval=0\nfog_alpha_value=7\n");
        assert!(c.enabled);
        assert_eq!(c.vanish_chance, 100);
        assert_eq!(c.spawn_min, 12_000);
        assert_eq!(c.block_hardness, 2.0);
        assert_eq!(c.break_interval, 1);
        assert_eq!(c.fog_alpha_value, 1.0);
    }

    #[test]
    fn ranges_stay_ordered() {
        let mut c = Config::default();
        c.load_text("spawn_min=100000\nspawn_max=500\ndistance_min=90\ndistance_max=20\nmin_chase_duration=5000\nmax_chase_duration=100\ndespawn_distance=10\n");
        assert!(c.spawn_min <= c.spawn_max);
        assert!(c.distance_min <= c.distance_max);
        assert!(c.chase_min <= c.chase_max);
        assert!(c.despawn_distance > c.distance_max);
    }

    #[test]
    fn rate_presets() {
        let mut c = Config::default();
        c.set_rate(2);
        assert_eq!(c.rate_index(), Some(2));
        c.set_rate(7); // wraps
        assert_eq!(c.rate_index(), Some(1));
        c.spawn_min = 777;
        assert_eq!(c.rate_index(), None);
    }

    #[test]
    fn jumpscare_is_read_and_written() {
        let mut c = Config::default();
        c.load_text("jumpscare=false");
        assert!(!c.jumpscare);
        assert!(c.to_text().contains("jumpscare=false"));
    }
}
