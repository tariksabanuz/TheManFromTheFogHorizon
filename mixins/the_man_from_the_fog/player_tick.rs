// His whole life, once a tick for each living player (the server's `player_tick`): read the
// player and the world, ask `fog.rs` what he does, and do it with Horizon's mob and world helpers.
//
// His body is a husk made quiet, with 300 health and a hard hit, named `MOB_TAG` so that the client draws the
// model of `model.rs` instead. While he only watches, his AI is off and he is moved by hand; a
// chase switches it on so that he walks, climbs and hits with the husk's own abilities.
fn the_man_from_the_fog_player(game: &mut crate::game::Game<'_>) {
    /// Whether nothing solid stands between `from` and `to` (sampled every half block).
    fn clear_line(game: &crate::game::Game<'_>, from: [f64; 3], to: [f64; 3]) -> bool {
        let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1.0 {
            return true;
        }
        let steps = (len / 0.5).ceil() as i32;
        for i in 1..steps {
            let t = f64::from(i) / f64::from(steps);
            let (x, y, z) = (from[0] + d[0] * t, from[1] + d[1] * t, from[2] + d[2] * t);
            let state = game.world.block(x.floor() as i32, y.floor() as i32, z.floor() as i32);
            if game.res.facts().get(state).can_occlude {
                return false;
            }
        }
        true
    }

    /// Breaks the blocks in the one-block-wide doorway ahead of him (feet, knees, head) that are not
    /// harder than `max_hardness`. Air, fluids and unbreakable blocks (hardness below 0) are left.
    // The window plays it for this player (a mod's own event from its sounds.json); a server plays it in place.
    fn play(game: &mut crate::game::Game<'_>, event: &str, at: [f64; 3]) {
        if !game.native_play_local_sound(event, at, 1.0, 1.0, 0) {
            game.native_play_sound_in_place(event, at, 5, 1.0, 1.0);
        }
    }

    fn break_ahead(game: &mut crate::game::Game<'_>, dim: &str, from: [f64; 3], towards: [f64; 3], max_hardness: f32) {
        let (dx, dz) = (towards[0] - from[0], towards[2] - from[2]);
        let len = (dx * dx + dz * dz).sqrt();
        if len < 1e-6 {
            return;
        }
        let (ax, az) = ((from[0] + dx / len * 1.2).floor() as i32, (from[2] + dz / len * 1.2).floor() as i32);
        let base = from[1].floor() as i32;
        let air = game.res.facts().air;
        for dy in 0..3 {
            let pos = [ax, base + dy, az];
            let state = game.world.block(pos[0], pos[1], pos[2]);
            let (breakable, solid) = {
                let f = game.res.facts().get(state);
                (f.hardness >= 0.0 && f.hardness <= max_hardness, f.full_collision)
            };
            if solid && breakable {
                game.native_set_block(dim, pos, air, 3);
            }
        }
    }

    /// Climbs: when a solid block is straight ahead of him at his feet, pushes him up and a little
    /// towards the player, so that he goes up a wall as a spider would.
    fn climb(game: &mut crate::game::Game<'_>, dim: &str, id: u32, from: [f64; 3], towards: [f64; 3]) {
        let (dx, dz) = (towards[0] - from[0], towards[2] - from[2]);
        let len = (dx * dx + dz * dz).sqrt();
        if len < 1e-6 {
            return;
        }
        let (ux, uz) = (dx / len, dz / len);
        let (ax, az) = ((from[0] + ux * 0.9).floor() as i32, (from[2] + uz * 0.9).floor() as i32);
        let ahead = game.world.block(ax, from[1].floor() as i32, az);
        if game.res.facts().get(ahead).full_collision {
            game.native_push_mob(dim, id, [ux * 0.12, 0.5, uz * 0.12]);
        }
    }

    use crate::game::native_the_man_from_the_fog_fog as fog;
    use crate::game::native_the_man_from_the_fog_runtime as rt;
    use crate::game::native_the_man_from_the_fog_state as st;

    let Some(p) = game.player.as_ref() else { return };
    let feet = [p.position.x, p.position.y, p.position.z];
    let (yaw, pitch) = (p.yaw, p.pitch);
    let key: u128 = game.lan_acting_uuid.unwrap_or(0);
    let dim: String = game.dim.id().to_owned();
    let day_time: i64 = game.world.day_time;
    let eye = [feet[0], feet[1] + 1.62, feet[2]];
    let view = crate::game::view_vector(yaw, pitch);

    let cfg = st::with(|s| s.config());
    let mut mob_id = st::with(|s| s.mob_of(key));

    // `all_dimensions` is off by default: he stays in the overworld.
    if !cfg.all_dimensions && dim != "minecraft:overworld" {
        if let Some(id) = mob_id {
            game.native_remove_mob(&dim, id);
            st::with(|s| {
                s.clear_mob(key);
                s.lost(key);
            });
        }
        return;
    }

    // Where he is, and whether the player has a clear line to his head.
    let mut sense_mob = None;
    if let Some(id) = mob_id {
        match game.native_mob(&dim, id) {
            Some(m) => {
                let head = [m.pos[0], m.pos[1] + 2.6, m.pos[2]];
                let in_sight = clear_line(game, eye, head);
                sense_mob = Some(fog::MobSense { pos: m.pos, in_sight });
            }
            None => {
                st::with(|s| {
                    s.clear_mob(key);
                    s.lost(key);
                });
                mob_id = None;
            }
        }
    }

    // The jumpscare starts when he is close in a hunt, whatever the state machine thinks.
    if let Some(id) = mob_id {
        if cfg.jumpscare && !st::with(|s| s.scare_started()) {
            let hunting = game.mobs.iter().any(|m| m.id == id && !m.no_ai && !m.removed);
            if let (true, Some(m)) = (hunting, game.native_mob(&dim, id)) {
                let (dx, dz) = (m.pos[0] - feet[0], m.pos[2] - feet[2]);
                if (dx * dx + dz * dz).sqrt() < 7.0 {
                    play(game, "the_man_from_the_fog:jumpscare", feet);
                    st::with(|s| s.start_scare());
                }
            }
        }
    }

    // He hits: 15 damage about once a second while he hunts and is within reach. With the jumpscare on he
    // waits for it to play out, then the blow kills the player and he goes too, in lightning.
    let mut killed = false;
    if let Some(id) = mob_id {
        let hunting = game.mobs.iter().any(|m| m.id == id && !m.no_ai && !m.removed);
        let scaring = st::with(|s| s.scare_elapsed_ms()).is_some();
        let lethal = cfg.jumpscare && st::with(|s| s.scare_finished());
        if hunting && !scaring && (lethal || !cfg.jumpscare) && st::with(|s| s.next_clock()) % 20 == 0 {
            if let Some(m) = game.native_mob(&dim, id) {
                let (dx, dy, dz) = (m.pos[0] - feet[0], m.pos[1] - feet[1], m.pos[2] - feet[2]);
                if (dx * dx + dz * dz).sqrt() < 3.5 && dy.abs() < 3.5 {
                    let me = game.player.as_ref().map_or(0, |p| p.entity_id as u32);
                    let amount = if lethal { 1000.0 } else { rt::DAMAGE };
                    let hurt = game.native_hurt_entity(&dim, me, amount, "minecraft:mob_attack", Some(id));
                    killed = hurt && lethal;
                }
            }
        }
    }
    if killed {
        if let Some(id) = mob_id {
            if let Some(m) = game.native_mob(&dim, id) {
                game.spawn_bolt(m.pos, true);
            }
            game.native_remove_mob(&dim, id);
        }
        st::with(|s| {
            s.clear_mob(key);
            s.lost(key);
        });
        return;
    }

    let sense = fog::Sense { player: feet, eye, view, day_time, mob: sense_mob };
    let actions = st::with(|s| s.step(key, &sense));

    for action in actions {
        match action {
            fog::Action::Spawn { distance, direction } => {
                let (x, z) = (feet[0] + distance * direction.sin(), feet[2] + distance * direction.cos());
                let (bx, bz) = (x.floor() as i32, z.floor() as i32);
                let ground = {
                    let g: &crate::game::Game<'_> = game;
                    let solid = |x: i32, y: i32, z: i32| g.res.facts().get(g.world.block(x, y, z)).full_collision;
                    fog::ground_y(&solid, bx, bz, feet[1] as i32 + 24, feet[1] as i32 - 24)
                };
                let spawned = ground.and_then(|y| {
                    let options = crate::game::native_mobs::NativeSpawn {
                        health: Some(rt::HEALTH),
                        no_ai: true,
                        silent: true,
                        ..Default::default()
                    };
                    game.native_spawn_mob(&dim, rt::BASE_KIND, [f64::from(bx) + 0.5, f64::from(y), f64::from(bz) + 0.5], &options)
                });
                match spawned {
                    Some(id) => {
                        if let Some(m) = game.mobs.iter_mut().find(|m| m.id == id) {
                            // 300 health and 15 damage, fast and hard to push, as the original's.
                            m.attrs.max_health = rt::HEALTH;
                            m.health = rt::HEALTH;
                            // His blows are the ones below (15 damage, or the jumpscare's), not the husk's own.
                            m.attrs.attack_damage = 0.0;
                            m.attrs.speed = rt::SPEED;
                            m.attrs.follow_range = rt::FOLLOW_RANGE;
                            m.attrs.knockback_resistance = rt::KNOCKBACK_RESISTANCE;
                        }
                        let at = [f64::from(bx) + 0.5, ground.map_or(feet[1], f64::from), f64::from(bz) + 0.5];
                        st::with(|s| {
                            s.set_mob(key, id);
                            s.spawned(key);
                        });
                        mob_id = Some(id);
                        play(game, fog::Cue::Appear.event(), at);
                        if st::with(|s| s.announce_once()) {
                            let events = ["spawn", "step", "whisper", "spotted", "chase", "jumpscare", "vanish"];
                            let missing: Vec<&str> = events
                                .iter()
                                .copied()
                                .filter(|e| !game.audio.engine.registry.contains(&format!("the_man_from_the_fog:{e}")))
                                .collect();
                            let picture = game.gui.atlas.sprites.contains_key("the_man_from_the_fog:gui/jumpscare_1");
                            let line = format!(
                                "The Man From The Fog {}: sounds {}, jumpscare picture {}.",
                                rt::VERSION,
                                if missing.is_empty() { "loaded".to_owned() } else { format!("MISSING ({})", missing.join(", ")) },
                                if picture { "loaded" } else { "MISSING" },
                            );
                            game.native_chat(&line);
                        }
                        if cfg.lightning {
                            game.spawn_bolt(at, true);
                        }
                    }
                    None => st::with(|s| s.spawn_failed(key)),
                }
            }
            fog::Action::Face(at) => {
                if let Some(id) = mob_id {
                    if let Some(m) = game.native_mob(&dim, id) {
                        let face = fog::yaw_towards(m.pos, at);
                        game.native_set_entity_rotation(&dim, id, face, 0.0, Some(face));
                    }
                }
            }
            fog::Action::StepCloser { distance } => {
                if let Some(id) = mob_id {
                    if let Some(m) = game.native_mob(&dim, id) {
                        let (dx, dz) = (m.pos[0] - feet[0], m.pos[2] - feet[2]);
                        let len = (dx * dx + dz * dz).sqrt();
                        if len > 1e-6 {
                            let (x, z) = (feet[0] + dx / len * distance, feet[2] + dz / len * distance);
                            let (bx, bz) = (x.floor() as i32, z.floor() as i32);
                            let ground = {
                                let g: &crate::game::Game<'_> = game;
                                let solid = |x: i32, y: i32, z: i32| g.res.facts().get(g.world.block(x, y, z)).full_collision;
                                fog::ground_y(&solid, bx, bz, m.pos[1] as i32 + 8, m.pos[1] as i32 - 8)
                            };
                            if let Some(y) = ground {
                                game.native_teleport_mob(&dim, id, [f64::from(bx) + 0.5, f64::from(y), f64::from(bz) + 0.5]);
                            }
                        }
                    }
                }
            }
            fog::Action::Hunt => {
                if let Some(id) = mob_id {
                    if let Some(m) = game.mobs.iter_mut().find(|m| m.id == id) {
                        m.no_ai = false;
                    }
                    game.native_mob_navigate(&dim, id, feet, 1, 1.5);
                }
            }
            fog::Action::BreakAhead { search } => {
                if let Some(id) = mob_id {
                    if let Some(m) = game.native_mob(&dim, id) {
                        let hardness = if search { cfg.block_hardness_search } else { cfg.block_hardness };
                        break_ahead(game, &dim, m.pos, feet, hardness);
                    }
                }
            }
            fog::Action::Climb => {
                if let Some(id) = mob_id {
                    if let Some(m) = game.native_mob(&dim, id) {
                        climb(game, &dim, id, m.pos, feet);
                    }
                }
            }
            fog::Action::Darkness => {
                // `darkness_effect`: five seconds of Darkness as the hunt begins.
                game.add_player_effect(horizon_client::effects::EffectInstance::new("darkness", 40, 0));
            }
            fog::Action::Jumpscare => {
                if !st::with(|s| s.scare_started()) {
                    play(game, "the_man_from_the_fog:jumpscare", feet);
                    st::with(|s| s.start_scare());
                }
            }
            fog::Action::Remove { with_lightning } => {
                if let Some(id) = mob_id {
                    if let Some(m) = game.native_mob(&dim, id) {
                        if with_lightning {
                            game.spawn_bolt(m.pos, true);
                        }
                    }
                    game.native_remove_mob(&dim, id);
                }
                st::with(|s| s.clear_mob(key));
                mob_id = None;
            }
            fog::Action::Sound(cue) => {
                // From where he stands for the mono sounds, so they carry; at the player's ears for the rest.
                let at = if cue.from_him() {
                    mob_id.and_then(|id| game.native_mob(&dim, id)).map_or(feet, |m| m.pos)
                } else {
                    feet
                };
                play(game, cue.event(), at);
            }
        }
    }
}
