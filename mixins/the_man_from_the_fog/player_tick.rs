// His whole life, once a tick for each living player (the server's `player_tick`): read the
// player and the world, ask `fog.rs` what he does, and do it with Horizon's mob and world helpers.
//
// His body is a husk made quiet and invulnerable, named `MOB_TAG` so that the client draws the
// model of `model.rs` instead. While he only watches, his AI is off and he is moved by hand; a
// chase switches it on so that he walks, climbs and hits with the husk's own abilities.
fn the_man_from_the_fog_player(game: &mut crate::game::Game<'_>) {
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
    let view = view_vector(yaw, pitch);

    let cfg = st::with(|s| s.config());

    // A stalker left in a saved world, or one a switch-off forgot, goes: only the tracked ones stay.
    if st::with(|s| s.next_clock()) % 100 == 0 {
        let tracked = st::with(|s| s.tracked());
        for m in game.mobs.iter_mut() {
            if !m.removed && m.custom_name.as_deref() == Some(rt::MOB_TAG) && !tracked.contains(&m.id) {
                m.removed = true;
            }
        }
    }

    // Where he is, and whether the player has a clear line to his head.
    let mut mob_id = st::with(|s| s.mob_of(key));
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
                        health: Some(60.0),
                        no_ai: true,
                        silent: true,
                        invulnerable: true,
                        ..Default::default()
                    };
                    game.native_spawn_mob(&dim, rt::BASE_KIND, [f64::from(bx) + 0.5, f64::from(y), f64::from(bz) + 0.5], &options)
                });
                match spawned {
                    Some(id) => {
                        if let Some(m) = game.mobs.iter_mut().find(|m| m.id == id) {
                            m.custom_name = Some(rt::MOB_TAG.to_owned());
                            m.name_visible = false;
                        }
                        let at = [f64::from(bx) + 0.5, ground.map_or(feet[1], f64::from), f64::from(bz) + 0.5];
                        st::with(|s| {
                            s.set_mob(key, id);
                            s.spawned(key);
                        });
                        mob_id = Some(id);
                        game.native_play_sound_in_place("minecraft:ambient.cave", feet, 5, 1.0, 0.7);
                        if cfg.lightning {
                            game.native_lightning_bolt(&dim, at, true);
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
            fog::Action::BreakAhead => {
                if let Some(id) = mob_id {
                    if let Some(m) = game.native_mob(&dim, id) {
                        break_ahead(game, &dim, m.pos, feet, cfg.break_hardness);
                    }
                }
            }
            fog::Action::Remove { with_lightning } => {
                if let Some(id) = mob_id {
                    if let Some(m) = game.native_mob(&dim, id) {
                        if with_lightning {
                            game.native_lightning_bolt(&dim, m.pos, true);
                        }
                    }
                    game.native_remove_mob(&dim, id);
                }
                st::with(|s| s.clear_mob(key));
                mob_id = None;
            }
            fog::Action::Sound(cue) => {
                let (sound, pitch) = match cue {
                    fog::Cue::Appear => ("minecraft:ambient.cave", 0.7),
                    fog::Cue::Step => ("minecraft:entity.enderman.stare", 0.6),
                    fog::Cue::Chase => ("minecraft:entity.ghast.scream", 0.5),
                    fog::Cue::Vanish => ("minecraft:entity.enderman.teleport", 0.5),
                };
                game.native_play_sound_in_place(sound, feet, 5, 1.0, pitch);
            }
        }
    }
}

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
