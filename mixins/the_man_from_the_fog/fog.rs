// The stalker's behaviour, with no game types in it: one `Fog` per player, fed a `Sense` once a
// tick, answering with the `Action`s the game side carries out.
//
//   Idle      a timer runs down; at night (or by day when allowed) he appears.
//   Watching  he stands far away and faces the player. Unwatched long enough he steps closer
//             (only while he is not being looked at), then starts a chase. Looked at, he either
//             vanishes or chases.
//   Chasing   he walks at the player, breaks what is in the way, and goes after a while.
//
// Nothing here is taken from another mod's code: the behaviour follows its public description.

use super::native_the_man_from_the_fog_config::Config;

pub type V3 = [f64; 3];

/// Where the player has to look, in degrees from the line to him, to count as eye contact.
pub const LOOK_CONE_DEGREES: f64 = 14.0;
/// Ticks of steady eye contact it takes (a flick of the mouse is not enough).
pub const GAZE_TICKS: u32 = 8;
/// While watching, ticks between two steps closer.
pub const STEP_EVERY: u32 = 100;
/// Blocks gained by a step.
pub const STEP_BLOCKS: f64 = 6.0;
/// He never steps closer than this while only watching.
pub const WATCH_MIN_DISTANCE: f64 = 14.0;
/// Ticks a retry waits after the game could not place him.
pub const RETRY_TICKS: u32 = 200;

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15 | 1)
    }
    pub fn next_u32(&mut self) -> u32 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
    }
    /// A value in `lo..=hi`.
    pub fn range(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo {
            return lo;
        }
        lo + self.next_u32() % (hi - lo + 1)
    }
    pub fn unit(&mut self) -> f64 {
        f64::from(self.next_u32()) / f64::from(u32::MAX)
    }
    pub fn chance(&mut self, percent: u32) -> bool {
        self.next_u32() % 100 < percent
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    /// Asked the game to place him; waiting to hear whether it did.
    Spawning,
    Watching,
    Chasing,
}

/// What the game side sees of him.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MobSense {
    pub pos: V3,
    /// A clear line from the player's eyes to his head.
    pub in_sight: bool,
}

/// What the game side sees of the world, once a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sense {
    pub player: V3,
    pub eye: V3,
    /// Unit vector the player looks along.
    pub view: V3,
    /// Time of day in ticks (0 at sunrise).
    pub day_time: i64,
    pub mob: Option<MobSense>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Appear,
    Step,
    Chase,
    /// The chase's own sound, repeated while he hunts.
    ChaseLoop,
    Vanish,
    /// A whisper now and then while he watches.
    Whisper,
}

impl Cue {
    /// The sound event the resource pack `tools/sound-pack` defines (namespace `the_man_from_the_fog`).
    pub fn event(self) -> &'static str {
        match self {
            Cue::Appear => "the_man_from_the_fog:spawn",
            Cue::Step => "the_man_from_the_fog:step",
            Cue::Chase => "the_man_from_the_fog:spotted",
            Cue::ChaseLoop => "the_man_from_the_fog:chase",
            Cue::Vanish => "the_man_from_the_fog:vanish",
            Cue::Whisper => "the_man_from_the_fog:whisper",
        }
    }

    /// Whether the sound is heard from where he stands (mono files) rather than at the player's ears.
    pub fn from_him(self) -> bool {
        matches!(self, Cue::Appear | Cue::Step | Cue::Vanish | Cue::Whisper)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Place him `distance` blocks from the player in `direction` (radians, 0 along +z).
    Spawn { distance: f64, direction: f64 },
    /// Turn him to face a point (while he only watches; a hunting mob turns itself).
    Face(V3),
    /// Move him to a point `distance` from the player on the line to where he stands.
    StepCloser { distance: f64 },
    /// Switch his own AI on (a chase) and send him towards the player.
    Hunt,
    /// Break what stands between him and the player. `search`: he cannot see the player.
    BreakAhead { search: bool },
    /// Climb the wall in front of him.
    Climb,
    /// The Darkness effect on the player as a hunt starts.
    Darkness,
    /// Take him away. `with_lightning` also strikes a harmless bolt where he stood.
    Remove { with_lightning: bool },
    Sound(Cue),
    /// He has reached the player in a hunt (`jumpscare`).
    Jumpscare,
}

#[derive(Clone, Debug)]
pub struct Fog {
    pub phase: Phase,
    /// Ticks until his next appearance.
    pub timer: u32,
    pub rng: Rng,
    unseen: u32,
    gaze: u32,
    since_step: u32,
    chase_left: u32,
    since_break: u32,
    hunt_tick: u32,
    last_pos: Option<V3>,
    still: u32,
    scared: bool,
    whisper_in: u32,
    loop_in: u32,
}

/// Blocks in front of the player where "Summon him now" puts him.
pub const SUMMON_DISTANCE: f64 = 4.0;
/// Ticks until the chase sound first repeats (the opening sound is about 7 s), and between repeats
/// (the chase sound is about 5.4 s).
const CHASE_LOOP_FIRST: u32 = 140;
const CHASE_LOOP_EVERY: u32 = 108;

pub fn is_night(day_time: i64) -> bool {
    (13_000..=23_000).contains(&day_time.rem_euclid(24_000))
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn length(a: V3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

pub fn horizontal_distance(a: V3, b: V3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Whether the player's view points at `target` closely enough, from `eye`.
pub fn is_looking_at(eye: V3, view: V3, target: V3) -> bool {
    let to = sub(target, eye);
    let d = length(to);
    let v = length(view);
    if d < 1e-6 || v < 1e-6 {
        return false;
    }
    let cos = (to[0] * view[0] + to[1] * view[1] + to[2] * view[2]) / (d * v);
    cos >= LOOK_CONE_DEGREES.to_radians().cos()
}

/// Yaw in degrees that makes something at `from` face `to` (0 faces south, +z; 90 faces west, -x).
pub fn yaw_towards(from: V3, to: V3) -> f32 {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    (-dx).atan2(dz).to_degrees() as f32
}

/// The feet height where something 3 blocks tall can stand in column (`x`, `z`), searching from
/// `top` down to `bottom`: a solid block below, three free blocks above.
pub fn ground_y(solid: &dyn Fn(i32, i32, i32) -> bool, x: i32, z: i32, top: i32, bottom: i32) -> Option<i32> {
    let mut y = top;
    while y >= bottom {
        if solid(x, y - 1, z) && !solid(x, y, z) && !solid(x, y + 1, z) && !solid(x, y + 2, z) {
            return Some(y);
        }
        y -= 1;
    }
    None
}

impl Fog {
    pub fn new(seed: u64, cfg: &Config) -> Fog {
        let mut rng = Rng::new(seed);
        let timer = rng.range(cfg.spawn_min, cfg.spawn_max);
        Fog {
            phase: Phase::Idle,
            timer,
            rng,
            unseen: 0,
            gaze: 0,
            since_step: 0,
            chase_left: 0,
            since_break: 0,
            hunt_tick: 0,
            last_pos: None,
            still: 0,
            scared: false,
            whisper_in: 0,
            loop_in: 0,
        }
    }

    fn rest(&mut self, cfg: &Config) {
        self.phase = Phase::Idle;
        self.timer = self.rng.range(cfg.spawn_min, cfg.spawn_max);
        self.unseen = 0;
        self.gaze = 0;
        self.since_step = 0;
        self.last_pos = None;
        self.still = 0;
    }

    /// The game placed him.
    pub fn spawned(&mut self) {
        self.phase = Phase::Watching;
        self.whisper_in = self.rng.range(300, 900);
        self.unseen = 0;
        self.gaze = 0;
        self.since_step = 0;
        self.last_pos = None;
        self.still = 0;
    }

    /// The game could not place him (no ground, chunk not loaded, peaceful): try again soon.
    pub fn spawn_failed(&mut self) {
        self.phase = Phase::Idle;
        self.timer = RETRY_TICKS;
    }

    /// He is gone without our asking (killed, unloaded, the player changed dimension).
    pub fn lost(&mut self, cfg: &Config) {
        self.rest(cfg);
    }

    /// Asks for his appearance now, whatever the timer and the hour say.
    pub fn summon_now(&mut self) {
        if self.phase == Phase::Idle {
            self.timer = 0;
        }
    }

    fn begin_chase(&mut self, cfg: &Config, out: &mut Vec<Action>) {
        self.phase = Phase::Chasing;
        self.chase_left = self.rng.range(cfg.chase_min, cfg.chase_max);
        self.since_break = 0;
        self.hunt_tick = 0;
        self.still = 0;
        self.scared = false;
        self.loop_in = CHASE_LOOP_FIRST;
        self.last_pos = None;
        out.push(Action::Sound(Cue::Chase));
        if cfg.darkness_effect {
            out.push(Action::Darkness);
        }
        out.push(Action::Hunt);
    }

    fn vanish(&mut self, cfg: &Config, out: &mut Vec<Action>) {
        out.push(Action::Sound(Cue::Vanish));
        out.push(Action::Remove { with_lightning: cfg.lightning });
        self.rest(cfg);
    }

    /// One game tick. `forced` is true while the player's own switch (the page's "summon" button)
    /// allows an appearance whatever the hour is.
    pub fn tick(&mut self, cfg: &Config, s: &Sense, forced: bool) -> Vec<Action> {
        let mut out = Vec::new();

        if !cfg.enabled {
            if self.phase != Phase::Idle && s.mob.is_some() {
                out.push(Action::Remove { with_lightning: false });
            }
            if self.phase != Phase::Spawning {
                self.rest(cfg);
            }
            return out;
        }

        match self.phase {
            Phase::Idle => {
                if forced {
                    self.timer = 0;
                }
                if self.timer > 0 {
                    self.timer -= 1;
                    return out;
                }
                if !(forced || cfg.spawn_at_day || is_night(s.day_time)) {
                    return out; // he waits for the dark
                }
                self.phase = Phase::Spawning;
                let view_yaw = s.view[0].atan2(s.view[2]);
                if forced {
                    // "Summon him now": four blocks straight ahead of the player.
                    out.push(Action::Spawn { distance: SUMMON_DISTANCE, direction: view_yaw });
                    return out;
                }
                let distance = cfg.distance_min + self.rng.unit() * (cfg.distance_max - cfg.distance_min);
                // Not in front of the player: somewhere in the half of the circle behind or beside them.
                let offset = (0.6 + self.rng.unit() * 1.9) * if self.rng.chance(50) { 1.0 } else { -1.0 };
                let direction = view_yaw + offset;
                out.push(Action::Spawn { distance, direction });
            }
            Phase::Spawning => {
                // The game answers in the same tick; if it never did, give up.
                self.spawn_failed();
            }
            Phase::Watching => {
                let Some(mob) = s.mob else {
                    self.lost(cfg);
                    return out;
                };
                let dist = length(sub(mob.pos, s.player));
                if dist > cfg.despawn_distance {
                    out.push(Action::Remove { with_lightning: false });
                    self.rest(cfg);
                    return out;
                }
                let looked = mob.in_sight && is_looking_at(s.eye, s.view, [mob.pos[0], mob.pos[1] + 2.6, mob.pos[2]]);
                if looked {
                    self.gaze += 1;
                    if self.gaze >= GAZE_TICKS {
                        if self.rng.chance(cfg.vanish_chance) {
                            self.vanish(cfg, &mut out);
                        } else {
                            self.begin_chase(cfg, &mut out);
                        }
                        return out;
                    }
                } else {
                    self.gaze = 0;
                    self.unseen += 1;
                }
                if self.whisper_in > 0 {
                    self.whisper_in -= 1;
                } else {
                    out.push(Action::Sound(Cue::Whisper));
                    self.whisper_in = self.rng.range(700, 1_800);
                }
                out.push(Action::Face(s.player));
                if self.unseen >= cfg.approach_after + cfg.chase_after {
                    self.begin_chase(cfg, &mut out);
                    return out;
                }
                if self.unseen >= cfg.approach_after && !looked {
                    self.since_step += 1;
                    if self.since_step >= STEP_EVERY && dist > WATCH_MIN_DISTANCE + 1.0 {
                        self.since_step = 0;
                        // Moves only while the player is not looking, so he is never seen moving.
                        if !mob.in_sight || !is_looking_at(s.eye, s.view, mob.pos) {
                            out.push(Action::StepCloser { distance: (dist - STEP_BLOCKS).max(WATCH_MIN_DISTANCE) });
                            out.push(Action::Sound(Cue::Step));
                        }
                    }
                }
            }
            Phase::Chasing => {
                let Some(mob) = s.mob else {
                    self.lost(cfg);
                    return out;
                };
                if self.chase_left == 0 {
                    self.vanish(cfg, &mut out);
                    return out;
                }
                self.chase_left -= 1;
                if self.loop_in > 0 {
                    self.loop_in -= 1;
                } else {
                    out.push(Action::Sound(Cue::ChaseLoop));
                    self.loop_in = CHASE_LOOP_EVERY;
                }
                self.hunt_tick += 1;
                if self.hunt_tick >= 5 {
                    self.hunt_tick = 0;
                    out.push(Action::Hunt);
                }
                // Stuck: hardly moved for a few ticks while still far from the player.
                if let Some(last) = self.last_pos {
                    if length(sub(mob.pos, last)) < 0.05 {
                        self.still += 1;
                    } else {
                        self.still = 0;
                    }
                }
                self.last_pos = Some(mob.pos);
                self.since_break += 1;
                let dist = length(sub(mob.pos, s.player));
                if cfg.jumpscare && !self.scared && dist < 7.0 {
                    self.scared = true;
                    out.push(Action::Jumpscare);
                }
                // Out of the player's sight and far enough away: he fades.
                if !mob.in_sight && dist > cfg.vanish_distance {
                    out.push(Action::Remove { with_lightning: false });
                    out.push(Action::Sound(Cue::Vanish));
                    self.rest(cfg);
                    return out;
                }
                let stuck = self.still >= 6 && dist > 2.0;
                if stuck && cfg.climbing && self.still % 4 == 2 {
                    out.push(Action::Climb);
                }
                let searching = !mob.in_sight;
                if cfg.break_blocks && stuck && self.since_break >= cfg.break_interval && (!searching || cfg.search_mechanic) {
                    self.since_break = 0;
                    out.push(Action::BreakAhead { search: searching });
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        let mut c = Config::default();
        c.spawn_min = 200;
        c.spawn_max = 200;
        c.approach_after = 40;
        c.chase_after = 100;
        c.chase_min = 60;
        c.chase_max = 60;
        c
    }

    fn sense(mob: Option<MobSense>, view: V3, day_time: i64) -> Sense {
        Sense { player: [0.0, 64.0, 0.0], eye: [0.0, 65.62, 0.0], view, day_time, mob }
    }

    const NIGHT: i64 = 18_000;
    const SOUTH: V3 = [0.0, 0.0, 1.0];
    const NORTH: V3 = [0.0, 0.0, -1.0];

    fn run_to_watching(f: &mut Fog, c: &Config) {
        for _ in 0..1000 {
            let out = f.tick(c, &sense(None, SOUTH, NIGHT), false);
            if out.iter().any(|a| matches!(a, Action::Spawn { .. })) {
                f.spawned();
                return;
            }
        }
        panic!("never asked to spawn");
    }

    fn at(z: f64, in_sight: bool) -> Option<MobSense> {
        Some(MobSense { pos: [0.0, 64.0, z], in_sight })
    }

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
        let mut r = Rng::new(1);
        for _ in 0..1000 {
            let v = r.range(5, 9);
            assert!((5..=9).contains(&v));
            let u = r.unit();
            assert!((0.0..=1.0).contains(&u));
        }
        assert_eq!(r.range(4, 4), 4);
        assert!(!r.chance(0));
        assert!(r.chance(100));
    }

    #[test]
    fn night_is_dusk_to_dawn() {
        assert!(!is_night(0));
        assert!(!is_night(6_000));
        assert!(!is_night(12_999));
        assert!(is_night(13_000));
        assert!(is_night(18_000));
        assert!(!is_night(23_001));
        assert!(is_night(24_000 * 5 + 15_000));
        assert!(is_night(-6_000)); // 18000 mod 24000
    }

    #[test]
    fn looking_uses_a_cone() {
        let eye = [0.0, 65.0, 0.0];
        let target = [0.0, 65.0, 30.0];
        assert!(is_looking_at(eye, SOUTH, target));
        assert!(is_looking_at(eye, [0.1, 0.0, 1.0], target)); // ~6 degrees off
        assert!(!is_looking_at(eye, [1.0, 0.0, 1.0], target)); // 45 degrees off
        assert!(!is_looking_at(eye, NORTH, target));
        assert!(!is_looking_at(eye, [0.0; 3], target));
        assert!(!is_looking_at(eye, SOUTH, eye));
    }

    #[test]
    fn yaw_matches_the_game_convention() {
        let o = [0.0, 0.0, 0.0];
        assert!((yaw_towards(o, [0.0, 0.0, 5.0]) - 0.0).abs() < 1e-4); // south
        assert!((yaw_towards(o, [-5.0, 0.0, 0.0]) - 90.0).abs() < 1e-4); // west
        assert!((yaw_towards(o, [0.0, 0.0, -5.0]).abs() - 180.0).abs() < 1e-4); // north
        assert!((yaw_towards(o, [5.0, 0.0, 0.0]) + 90.0).abs() < 1e-4); // east
    }

    #[test]
    fn ground_search() {
        // Solid below y=60 only.
        let flat = |_: i32, y: i32, _: i32| y < 60;
        assert_eq!(ground_y(&flat, 3, 4, 80, 40), Some(60));
        // A roof at y=61 leaves no room.
        let roofed = |_: i32, y: i32, _: i32| y < 60 || y == 62;
        assert_eq!(ground_y(&roofed, 0, 0, 80, 40), Some(63)); // the roof's top is the first floor found
        assert_eq!(ground_y(&roofed, 0, 0, 62, 40), None); // below it, 60 has the roof at head height
        // Nothing solid in range.
        let air = |_: i32, _: i32, _: i32| false;
        assert_eq!(ground_y(&air, 0, 0, 80, 40), None);
        // Solid all the way up: no space.
        let full = |_: i32, _: i32, _: i32| true;
        assert_eq!(ground_y(&full, 0, 0, 80, 40), None);
    }

    #[test]
    fn he_waits_for_night_and_for_the_timer() {
        let c = cfg();
        let mut f = Fog::new(1, &c);
        for _ in 0..500 {
            // Daytime: the timer runs out but he does not come.
            assert!(f.tick(&c, &sense(None, SOUTH, 6_000), false).is_empty());
        }
        assert_eq!(f.timer, 0);
        assert_eq!(f.phase, Phase::Idle);
        let out = f.tick(&c, &sense(None, SOUTH, NIGHT), false);
        assert!(matches!(out.as_slice(), [Action::Spawn { .. }]));
        assert_eq!(f.phase, Phase::Spawning);
    }

    #[test]
    fn day_spawns_and_forced_summons_ignore_the_hour() {
        let mut c = cfg();
        c.spawn_at_day = true;
        let mut f = Fog::new(2, &c);
        f.timer = 0;
        assert!(f.tick(&c, &sense(None, SOUTH, 6_000), false).iter().any(|a| matches!(a, Action::Spawn { .. })));
        c.spawn_at_day = false;
        let mut g = Fog::new(3, &c);
        let out = g.tick(&c, &sense(None, SOUTH, 6_000), true);
        assert!(out.iter().any(|a| matches!(a, Action::Spawn { .. })));
    }

    #[test]
    fn a_summon_puts_him_four_blocks_straight_ahead() {
        let c = cfg();
        for (view, yaw) in [(SOUTH, 0.0_f64), (NORTH, std::f64::consts::PI)] {
            let mut f = Fog::new(5, &c);
            let out = f.tick(&c, &sense(None, view, 6_000), true);
            let Some(Action::Spawn { distance, direction }) = out.first().cloned() else { panic!("no spawn") };
            assert_eq!(distance, 4.0);
            assert!((direction.sin() - yaw.sin()).abs() < 1e-9 && (direction.cos() - yaw.cos()).abs() < 1e-9);
        }
    }

    #[test]
    fn spawn_stays_out_of_the_players_view() {
        let c = cfg();
        for seed in 0..200 {
            let mut f = Fog::new(seed, &c);
            f.timer = 0;
            let out = f.tick(&c, &sense(None, SOUTH, NIGHT), false);
            let Some(Action::Spawn { distance, direction }) = out.first().cloned() else { panic!("no spawn") };
            assert!((c.distance_min..=c.distance_max).contains(&distance));
            // The player looks along +z (direction 0): he is at least ~34 degrees away from that.
            let diff = direction.sin().atan2(direction.cos()).abs();
            assert!(diff > 0.55, "seed {seed}: direction {direction} too close to the view");
        }
    }

    #[test]
    fn a_failed_spawn_retries_soon() {
        let c = cfg();
        let mut f = Fog::new(1, &c);
        f.timer = 0;
        f.tick(&c, &sense(None, SOUTH, NIGHT), false);
        f.spawn_failed();
        assert_eq!((f.phase, f.timer), (Phase::Idle, RETRY_TICKS));
        // If the game never answered, the next tick gives up the same way.
        let mut g = Fog::new(1, &c);
        g.timer = 0;
        g.tick(&c, &sense(None, SOUTH, NIGHT), false);
        g.tick(&c, &sense(None, SOUTH, NIGHT), false);
        assert_eq!(g.phase, Phase::Idle);
    }

    #[test]
    fn unwatched_he_steps_closer_then_chases() {
        let c = cfg();
        let mut f = Fog::new(5, &c);
        run_to_watching(&mut f, &c);
        let mut z = 50.0;
        let (mut stepped, mut chased) = (false, false);
        for _ in 0..1000 {
            // The player looks away from him (north) the whole time.
            let out = f.tick(&c, &sense(at(z, false), NORTH, NIGHT), false);
            for a in out {
                match a {
                    Action::StepCloser { distance } => {
                        assert!(distance < z && distance >= WATCH_MIN_DISTANCE);
                        z = distance;
                        stepped = true;
                    }
                    Action::Hunt => chased = true,
                    _ => {}
                }
            }
            if chased {
                break;
            }
        }
        assert!(stepped, "he should have come closer");
        assert!(chased, "and then hunted");
        assert_eq!(f.phase, Phase::Chasing);
    }

    #[test]
    fn he_never_moves_while_watched() {
        let mut c = cfg();
        c.vanish_chance = 0;
        c.approach_after = 20;
        c.chase_after = 100_000;
        let mut f = Fog::new(9, &c);
        run_to_watching(&mut f, &c);
        // Looking at him for fewer ticks than a gaze, then away: no step is taken while looking.
        for _ in 0..(GAZE_TICKS - 1) {
            let out = f.tick(&c, &sense(at(40.0, true), SOUTH, NIGHT), false);
            assert!(!out.iter().any(|a| matches!(a, Action::StepCloser { .. })));
        }
        assert_eq!(f.phase, Phase::Watching);
    }

    #[test]
    fn eye_contact_vanishes_or_chases() {
        let mut c = cfg();
        c.vanish_chance = 100;
        let mut f = Fog::new(11, &c);
        run_to_watching(&mut f, &c);
        let mut got = Vec::new();
        for _ in 0..GAZE_TICKS {
            got = f.tick(&c, &sense(at(40.0, true), SOUTH, NIGHT), false);
        }
        assert!(got.contains(&Action::Remove { with_lightning: true }));
        assert!(got.contains(&Action::Sound(Cue::Vanish)));
        assert_eq!(f.phase, Phase::Idle);
        assert_eq!(f.timer, 200);

        c.vanish_chance = 0;
        let mut g = Fog::new(12, &c);
        run_to_watching(&mut g, &c);
        for _ in 0..GAZE_TICKS {
            got = g.tick(&c, &sense(at(40.0, true), SOUTH, NIGHT), false);
        }
        assert!(got.contains(&Action::Hunt));
        assert!(got.contains(&Action::Sound(Cue::Chase)));
        assert_eq!(g.phase, Phase::Chasing);
    }

    #[test]
    fn a_glance_through_a_wall_is_not_eye_contact() {
        let mut c = cfg();
        c.vanish_chance = 100;
        let mut f = Fog::new(13, &c);
        run_to_watching(&mut f, &c);
        for _ in 0..100 {
            f.tick(&c, &sense(at(40.0, false), SOUTH, NIGHT), false); // facing him, but a wall between
        }
        assert_eq!(f.phase, Phase::Watching);
    }

    #[test]
    fn a_chase_ends_and_he_goes() {
        let mut c = cfg();
        c.vanish_chance = 0;
        let mut f = Fog::new(14, &c);
        run_to_watching(&mut f, &c);
        for _ in 0..GAZE_TICKS {
            f.tick(&c, &sense(at(30.0, true), SOUTH, NIGHT), false);
        }
        assert_eq!(f.phase, Phase::Chasing);
        let mut removed = false;
        let mut hunts = 0;
        for i in 0..200 {
            let out = f.tick(&c, &sense(at(30.0 - i as f64 * 0.2, true), SOUTH, NIGHT), false);
            hunts += out.iter().filter(|a| **a == Action::Hunt).count();
            if out.iter().any(|a| matches!(a, Action::Remove { .. })) {
                removed = true;
                break;
            }
        }
        assert!(removed);
        assert!(hunts >= 10, "he re-aims every few ticks ({hunts})");
        assert_eq!(f.phase, Phase::Idle);
    }

    #[test]
    fn a_stuck_chaser_breaks_blocks_only_if_allowed() {
        let mut c = cfg();
        c.vanish_chance = 0;
        c.chase_min = 1000;
        c.chase_max = 1000;
        for allowed in [true, false] {
            c.break_blocks = allowed;
            let mut f = Fog::new(15, &c);
            run_to_watching(&mut f, &c);
            for _ in 0..GAZE_TICKS {
                f.tick(&c, &sense(at(20.0, true), SOUTH, NIGHT), false);
            }
            let mut breaks = 0;
            for _ in 0..100 {
                // He does not move: a wall.
                breaks += f.tick(&c, &sense(at(20.0, true), SOUTH, NIGHT), false).iter().filter(|a| matches!(a, Action::BreakAhead { .. })).count();
            }
            assert_eq!(breaks > 0, allowed, "allowed={allowed} breaks={breaks}");
            if allowed {
                assert!(breaks <= 100 / c.break_interval as usize + 1, "paced by the interval ({breaks})");
            }
        }
    }

    #[test]
    fn a_walking_chaser_breaks_nothing() {
        let mut c = cfg();
        c.vanish_chance = 0;
        c.chase_min = 1000;
        c.chase_max = 1000;
        let mut f = Fog::new(16, &c);
        run_to_watching(&mut f, &c);
        for _ in 0..GAZE_TICKS {
            f.tick(&c, &sense(at(40.0, true), SOUTH, NIGHT), false);
        }
        for i in 0..100 {
            let out = f.tick(&c, &sense(at(40.0 - i as f64 * 0.3, true), SOUTH, NIGHT), false);
            assert!(!out.iter().any(|a| matches!(a, Action::BreakAhead { .. })));
        }
    }

    #[test]
    fn far_away_he_goes() {
        let c = cfg();
        let mut f = Fog::new(17, &c);
        run_to_watching(&mut f, &c);
        let out = f.tick(&c, &sense(at(c.despawn_distance + 5.0, false), NORTH, NIGHT), false);
        assert_eq!(out, vec![Action::Remove { with_lightning: false }]);
        assert_eq!(f.phase, Phase::Idle);
    }

    #[test]
    fn losing_him_resets() {
        let c = cfg();
        let mut f = Fog::new(18, &c);
        run_to_watching(&mut f, &c);
        assert!(f.tick(&c, &sense(None, SOUTH, NIGHT), false).is_empty());
        assert_eq!(f.phase, Phase::Idle);
        assert!(f.timer >= c.spawn_min);
    }

    #[test]
    fn switching_off_removes_him() {
        let mut c = cfg();
        let mut f = Fog::new(19, &c);
        run_to_watching(&mut f, &c);
        c.enabled = false;
        let out = f.tick(&c, &sense(at(40.0, false), SOUTH, NIGHT), false);
        assert_eq!(out, vec![Action::Remove { with_lightning: false }]);
        assert_eq!(f.phase, Phase::Idle);
        // And nothing happens while off.
        for _ in 0..500 {
            assert!(f.tick(&c, &sense(None, SOUTH, NIGHT), true).is_empty());
        }
    }

    #[test]
    fn summon_now_works_from_idle_only() {
        let c = cfg();
        let mut f = Fog::new(20, &c);
        f.summon_now();
        assert_eq!(f.timer, 0);
        run_to_watching(&mut f, &c);
        f.timer = 77;
        f.summon_now();
        assert_eq!(f.timer, 77);
    }

    #[test]
    fn a_hunt_starts_with_darkness_if_enabled() {
        for on in [true, false] {
            let mut c = cfg();
            c.vanish_chance = 0;
            c.darkness_effect = on;
            let mut f = Fog::new(30, &c);
            run_to_watching(&mut f, &c);
            let mut got = Vec::new();
            for _ in 0..GAZE_TICKS {
                got = f.tick(&c, &sense(at(40.0, true), SOUTH, NIGHT), false);
            }
            assert_eq!(got.contains(&Action::Darkness), on);
        }
    }

    #[test]
    fn a_hunter_who_lost_the_player_fades_at_the_vanish_distance() {
        let mut c = cfg();
        c.vanish_chance = 0;
        c.vanish_distance = 30.0;
        c.chase_min = 5000;
        c.chase_max = 5000;
        let mut f = Fog::new(31, &c);
        run_to_watching(&mut f, &c);
        for _ in 0..GAZE_TICKS {
            f.tick(&c, &sense(at(40.0, true), SOUTH, NIGHT), false);
        }
        assert_eq!(f.phase, Phase::Chasing);
        // In sight and far: he keeps hunting.
        for _ in 0..20 {
            let out = f.tick(&c, &sense(at(45.0, true), SOUTH, NIGHT), false);
            assert!(!out.iter().any(|a| matches!(a, Action::Remove { .. })));
        }
        // Out of sight but near: he keeps hunting.
        let out = f.tick(&c, &sense(at(20.0, false), SOUTH, NIGHT), false);
        assert!(!out.iter().any(|a| matches!(a, Action::Remove { .. })));
        // Out of sight and farther than 30: gone, without lightning.
        let out = f.tick(&c, &sense(at(35.0, false), SOUTH, NIGHT), false);
        assert!(out.contains(&Action::Remove { with_lightning: false }));
        assert_eq!(f.phase, Phase::Idle);
    }

    fn stuck_chase(c: &Config, in_sight: bool, ticks: usize) -> Vec<Action> {
        let mut f = Fog::new(32, c);
        run_to_watching(&mut f, c);
        for _ in 0..GAZE_TICKS {
            f.tick(c, &sense(at(20.0, true), SOUTH, NIGHT), false);
        }
        let mut all = Vec::new();
        for _ in 0..ticks {
            all.extend(f.tick(c, &sense(at(20.0, in_sight), SOUTH, NIGHT), false));
        }
        all
    }

    #[test]
    fn stuck_hunters_climb_if_allowed() {
        let mut c = cfg();
        c.vanish_chance = 0;
        c.chase_min = 5000;
        c.chase_max = 5000;
        c.vanish_distance = 200.0;
        for on in [true, false] {
            c.climbing = on;
            let acts = stuck_chase(&c, true, 60);
            assert_eq!(acts.contains(&Action::Climb), on);
        }
    }

    #[test]
    fn searching_breaks_only_with_the_search_mechanic() {
        let mut c = cfg();
        c.vanish_chance = 0;
        c.chase_min = 5000;
        c.chase_max = 5000;
        c.vanish_distance = 200.0;
        c.search_mechanic = true;
        let acts = stuck_chase(&c, false, 60);
        assert!(acts.contains(&Action::BreakAhead { search: true }));
        c.search_mechanic = false;
        let acts = stuck_chase(&c, false, 60);
        assert!(!acts.iter().any(|a| matches!(a, Action::BreakAhead { .. })));
        // Seeing the player he still breaks.
        let acts = stuck_chase(&c, true, 60);
        assert!(acts.contains(&Action::BreakAhead { search: false }));
    }

    #[test]
    fn every_cue_names_a_sound_event_in_the_mods_namespace() {
        let all = [Cue::Appear, Cue::Step, Cue::Chase, Cue::ChaseLoop, Cue::Vanish, Cue::Whisper];
        let mut seen = std::collections::HashSet::new();
        for c in all {
            let e = c.event();
            assert!(e.starts_with("the_man_from_the_fog:") && e.len() > 21, "{e}");
            assert!(seen.insert(e), "{e} is used twice");
        }
    }

    #[test]
    fn he_whispers_now_and_then_while_watching() {
        let mut c = cfg();
        c.approach_after = 20_000;
        c.chase_after = 20_000;
        let mut f = Fog::new(11, &c);
        run_to_watching(&mut f, &c);
        let mut whispers = 0;
        for _ in 0..3_000 {
            let out = f.tick(&c, &sense(Some(MobSense { pos: [0.0, 0.0, 30.0], in_sight: false }), SOUTH, NIGHT), false);
            whispers += out.iter().filter(|a| **a == Action::Sound(Cue::Whisper)).count();
            if f.phase != Phase::Watching {
                break;
            }
        }
        assert!(whispers >= 1, "no whisper in 3000 ticks ({whispers})");
    }

    #[test]
    fn the_chase_sound_repeats_while_he_hunts() {
        let mut c = cfg();
        c.chase_min = 1_000;
        c.chase_max = 1_000;
        let mut f = Fog::new(12, &c);
        run_to_watching(&mut f, &c);
        let mut out = Vec::new();
        f.begin_chase(&c, &mut out);
        let mut loops = 0;
        for _ in 0..500 {
            let o = f.tick(&c, &sense(Some(MobSense { pos: [0.0, 0.0, 20.0], in_sight: true }), SOUTH, NIGHT), false);
            loops += o.iter().filter(|a| **a == Action::Sound(Cue::ChaseLoop)).count();
        }
        assert!(loops >= 3, "{loops}");
    }
}
