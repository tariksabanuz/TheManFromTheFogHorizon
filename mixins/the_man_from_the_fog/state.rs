// What the mod keeps while the game runs: the config, one `Fog` per player and the buttons of the
// mod's page in Options > Mods. Both halves of the game (the window and the single-player server
// thread) are in one process and share this.

use super::native_the_man_from_the_fog_config::{Config, FOG_STEPS, HARDNESS_STEPS, RATES, VANISH_STEPS};
use super::native_the_man_from_the_fog_fog::{Action, Fog, Sense};
use super::native_the_man_from_the_fog_runtime as runtime;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub struct State {
    dir: PathBuf,
    loaded: bool,
    pub cfg: Config,
    fogs: HashMap<u128, Fog>,
    summon: bool,
    clock: u64,
    /// The mob that is his body, by player.
    mobs: HashMap<u128, u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Button {
    Enabled,
    DayNight,
    Rate,
    Vanish,
    Breaks,
    Hardness,
    Search,
    Climbing,
    Darkness,
    Fog,
    AllDimensions,
    Lightning,
    Summon,
    Reset,
}

const BUTTONS: [Button; 14] = [
    Button::Enabled,
    Button::DayNight,
    Button::Rate,
    Button::Vanish,
    Button::Breaks,
    Button::Hardness,
    Button::Search,
    Button::Climbing,
    Button::Darkness,
    Button::Fog,
    Button::AllDimensions,
    Button::Lightning,
    Button::Summon,
    Button::Reset,
];

fn on_off(v: bool) -> &'static str {
    if v { "on" } else { "off" }
}

impl State {
    pub fn new(dir: PathBuf) -> State {
        State { dir, loaded: false, cfg: Config::default(), fogs: HashMap::new(), summon: false, clock: 0, mobs: HashMap::new() }
    }

    fn ensure_loaded(&mut self) {
        if !self.loaded {
            self.loaded = true;
            if let Ok(text) = std::fs::read_to_string(self.dir.join("config.txt")) {
                self.cfg.load_text(&text);
            }
        }
    }

    fn save(&self) {
        // Best effort: a folder that cannot be written to just forgets the choices.
        if std::fs::create_dir_all(&self.dir).is_ok() {
            let _ = std::fs::write(self.dir.join("config.txt"), self.cfg.to_text());
        }
    }

    // ---- the page ----

    pub fn buttons(&mut self) -> Vec<String> {
        self.ensure_loaded();
        BUTTONS.iter().map(|b| self.label(*b)).collect()
    }

    fn label(&self, b: Button) -> String {
        let c = &self.cfg;
        match b {
            Button::Enabled => format!("The Man From The Fog: {}", on_off(c.enabled)),
            Button::DayNight => format!("Spawn at day: {}", on_off(c.spawn_at_day)),
            Button::Rate => format!("Spawn rate: {}", c.rate_index().map_or("custom", |i| RATES[i].0)),
            Button::Vanish => format!("Eye contact makes him vanish: {}%", c.vanish_chance),
            Button::Breaks => format!("Break blocks: {}", on_off(c.break_blocks)),
            Button::Hardness => format!("Block hardness: {}", c.block_hardness),
            Button::Search => format!("Search mechanic: {}", on_off(c.search_mechanic)),
            Button::Climbing => format!("Climbing: {}", on_off(c.climbing)),
            Button::Darkness => format!("Darkness effect: {}", on_off(c.darkness_effect)),
            Button::Fog => {
                if c.enable_fog {
                    format!("Fog: {:.0}%", c.fog_alpha_value * 100.0)
                } else {
                    "Fog: off".to_string()
                }
            }
            Button::AllDimensions => format!("All dimensions: {}", on_off(c.all_dimensions)),
            Button::Lightning => format!("Lightning: {}", on_off(c.lightning)),
            Button::Summon => "Summon him now".to_string(),
            Button::Reset => "Reset settings".to_string(),
        }
    }

    pub fn click(&mut self, index: usize) {
        self.ensure_loaded();
        let Some(&button) = BUTTONS.get(index) else { return };
        let c = &mut self.cfg;
        match button {
            Button::Enabled => c.enabled = !c.enabled,
            Button::DayNight => c.spawn_at_day = !c.spawn_at_day,
            Button::Rate => {
                let next = c.rate_index().map_or(0, |i| i + 1);
                c.set_rate(next);
            }
            Button::Vanish => {
                c.vanish_chance = VANISH_STEPS.iter().copied().find(|v| *v > c.vanish_chance).unwrap_or(VANISH_STEPS[0]);
            }
            Button::Breaks => c.break_blocks = !c.break_blocks,
            Button::Hardness => {
                c.block_hardness = HARDNESS_STEPS.iter().copied().find(|v| *v > c.block_hardness + 1e-4).unwrap_or(HARDNESS_STEPS[0]);
            }
            Button::Search => c.search_mechanic = !c.search_mechanic,
            Button::Climbing => c.climbing = !c.climbing,
            Button::Darkness => c.darkness_effect = !c.darkness_effect,
            Button::Fog => {
                // off, then 25 / 50 / 75 / 100 %, then off again.
                if !c.enable_fog {
                    c.enable_fog = true;
                    c.fog_alpha_value = FOG_STEPS[1];
                } else {
                    match FOG_STEPS.iter().copied().find(|v| *v > c.fog_alpha_value + 1e-4) {
                        Some(v) => c.fog_alpha_value = v,
                        None => c.enable_fog = false,
                    }
                }
            }
            Button::AllDimensions => c.all_dimensions = !c.all_dimensions,
            Button::Lightning => c.lightning = !c.lightning,
            Button::Summon => {
                c.enabled = true;
                self.summon = true;
            }
            Button::Reset => *c = Config::default(),
        }
        c.clamp();
        self.save();
    }

    // ---- the game ----

    fn seed(&mut self, key: u128) -> u64 {
        self.clock = self.clock.wrapping_add(1);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        nanos ^ (key as u64) ^ ((key >> 64) as u64).rotate_left(17) ^ self.clock.rotate_left(33)
    }

    /// One tick for the player `key`.
    pub fn step(&mut self, key: u128, sense: &Sense) -> Vec<Action> {
        self.ensure_loaded();
        if !self.fogs.contains_key(&key) {
            let seed = self.seed(key);
            self.fogs.insert(key, Fog::new(seed, &self.cfg));
        }
        let forced = std::mem::take(&mut self.summon);
        let cfg = self.cfg.clone();
        let Some(fog) = self.fogs.get_mut(&key) else { return Vec::new() };
        if forced {
            fog.summon_now();
        }
        fog.tick(&cfg, sense, forced)
    }

    pub fn spawned(&mut self, key: u128) {
        if let Some(f) = self.fogs.get_mut(&key) {
            f.spawned();
        }
    }

    pub fn spawn_failed(&mut self, key: u128) {
        if let Some(f) = self.fogs.get_mut(&key) {
            f.spawn_failed();
        }
    }

    pub fn lost(&mut self, key: u128) {
        let cfg = self.cfg.clone();
        if let Some(f) = self.fogs.get_mut(&key) {
            f.lost(&cfg);
        }
    }

    pub fn forget_all(&mut self) {
        self.fogs.clear();
        self.mobs.clear();
    }

    pub fn set_mob(&mut self, key: u128, id: u32) {
        self.mobs.insert(key, id);
    }

    pub fn clear_mob(&mut self, key: u128) {
        self.mobs.remove(&key);
    }

    pub fn mob_of(&self, key: u128) -> Option<u32> {
        self.mobs.get(&key).copied()
    }

    /// Every mob that is somebody's stalker, so that a leftover one (from a saved world) is not mistaken for one.
    pub fn tracked(&self) -> Vec<u32> {
        self.mobs.values().copied().collect()
    }

    /// A counter that goes up by one each time it is asked, for work done every so many ticks.
    pub fn next_clock(&mut self) -> u64 {
        self.clock = self.clock.wrapping_add(1);
        self.clock
    }

    pub fn config(&mut self) -> Config {
        self.ensure_loaded();
        self.cfg.clone()
    }
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

/// Runs `f` on the mod's state, made on first use with its folder in the game folder.
pub fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let m = STATE.get_or_init(|| Mutex::new(State::new(PathBuf::from(".").join(runtime::DIR))));
    let mut guard = m.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::native_the_man_from_the_fog_fog::Phase;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("fog_state_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn night_sense() -> Sense {
        Sense { player: [0.0, 64.0, 0.0], eye: [0.0, 65.6, 0.0], view: [0.0, 0.0, 1.0], day_time: 18_000, mob: None }
    }

    fn index_of(b: Button) -> usize {
        BUTTONS.iter().position(|x| *x == b).unwrap()
    }

    #[test]
    fn labels_follow_the_config() {
        let mut s = State::new(temp_dir("labels"));
        let b = s.buttons();
        assert_eq!(b.len(), BUTTONS.len());
        assert_eq!(b[index_of(Button::Enabled)], "The Man From The Fog: on");
        assert_eq!(b[index_of(Button::DayNight)], "Spawn at day: off");
        assert_eq!(b[index_of(Button::Rate)], "Spawn rate: normal");
        assert_eq!(b[index_of(Button::Vanish)], "Eye contact makes him vanish: 50%");
        assert_eq!(b[index_of(Button::Hardness)], "Block hardness: 2");
        assert_eq!(b[index_of(Button::Fog)], "Fog: 60%");
        assert_eq!(b[index_of(Button::AllDimensions)], "All dimensions: off");
    }

    #[test]
    fn clicks_change_and_save_the_choices() {
        let dir = temp_dir("clicks");
        let mut s = State::new(dir.clone());
        s.buttons();
        for b in [
            Button::Enabled,
            Button::DayNight,
            Button::Rate, // normal -> often
            Button::Vanish, // 50 -> 75
            Button::Breaks,
            Button::Hardness, // 2 -> 3.5
            Button::Search,
            Button::Climbing,
            Button::Darkness,
            Button::AllDimensions,
            Button::Lightning,
        ] {
            s.click(index_of(b));
        }
        let c = s.cfg.clone();
        assert!(!c.enabled && c.spawn_at_day && !c.break_blocks && !c.lightning);
        assert!(!c.search_mechanic && !c.climbing && !c.darkness_effect && c.all_dimensions);
        assert_eq!(c.rate_index(), Some(2));
        assert_eq!(c.vanish_chance, 75);
        assert_eq!(c.block_hardness, 3.5);
        let mut again = State::new(dir.clone());
        again.buttons();
        assert_eq!(again.cfg, c);
        again.click(index_of(Button::Reset));
        assert_eq!(again.cfg, Config::default());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn steppers_wrap() {
        let mut s = State::new(temp_dir("wrap"));
        s.buttons();
        for _ in 0..3 {
            s.click(index_of(Button::Rate));
        }
        assert_eq!(s.cfg.rate_index(), Some(1)); // often, rare, normal
        s.cfg.vanish_chance = 100;
        s.click(index_of(Button::Vanish));
        assert_eq!(s.cfg.vanish_chance, 0);
        s.cfg.block_hardness = 50.0;
        s.click(index_of(Button::Hardness));
        assert_eq!(s.cfg.block_hardness, 0.5);
    }

    #[test]
    fn fog_button_cycles_through_off() {
        let mut s = State::new(temp_dir("fogbtn"));
        s.buttons();
        let i = index_of(Button::Fog);
        let mut seen = Vec::new();
        for _ in 0..6 {
            s.click(i);
            seen.push(s.buttons()[i].clone());
        }
        assert_eq!(seen, ["Fog: 75%", "Fog: 100%", "Fog: off", "Fog: 25%", "Fog: 50%", "Fog: 75%"]);
    }

    #[test]
    fn clicks_out_of_range_do_nothing() {
        let mut s = State::new(temp_dir("range"));
        let before = s.buttons();
        s.click(99);
        s.click(usize::MAX);
        assert_eq!(s.buttons(), before);
    }

    #[test]
    fn summon_button_makes_the_next_tick_spawn_him() {
        let mut s = State::new(temp_dir("summon"));
        s.cfg.enabled = false;
        s.buttons();
        let summon = index_of(Button::Summon);
        s.click(summon);
        assert!(s.cfg.enabled);
        let out = s.step(1, &Sense { day_time: 6_000, ..night_sense() });
        assert!(out.iter().any(|a| matches!(a, Action::Spawn { .. })), "{out:?}");
        // One summon makes one appearance.
        s.spawned(1);
        assert_eq!(s.fogs[&1].phase, Phase::Watching);
    }

    #[test]
    fn players_have_their_own_stalker() {
        let mut s = State::new(temp_dir("players"));
        s.cfg.spawn_min = 200;
        s.cfg.spawn_max = 200;
        s.step(1, &night_sense());
        s.step(2, &night_sense());
        s.fogs.get_mut(&1).unwrap().timer = 0;
        let a = s.step(1, &night_sense());
        let b = s.step(2, &night_sense());
        assert!(a.iter().any(|x| matches!(x, Action::Spawn { .. })));
        assert!(b.is_empty());
        s.spawn_failed(1);
        assert_eq!(s.fogs[&1].phase, Phase::Idle);
        s.set_mob(1, 42);
        assert_eq!(s.mob_of(1), Some(42));
        assert_eq!(s.tracked(), vec![42]);
        s.clear_mob(1);
        assert_eq!(s.mob_of(1), None);
        s.set_mob(2, 7);
        s.forget_all();
        assert!(s.fogs.is_empty() && s.tracked().is_empty());
    }
}
