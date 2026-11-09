// Nobody sleeps with him near: the bed refuses within `SLEEP_RADIUS` blocks of one of him
// ("you may not rest now, there are monsters nearby").
fn the_man_from_the_fog_sleep(game: &crate::game::Game<'_>, pos: [i32; 3], problem: &mut Option<String>) {
    use crate::game::native_the_man_from_the_fog_runtime as rt;
    use crate::game::native_the_man_from_the_fog_view as view;

    let bed = [f64::from(pos[0]) + 0.5, f64::from(pos[1]), f64::from(pos[2]) + 0.5];
    for m in game.mobs.iter() {
        if m.removed || m.custom_name.as_deref() != Some(rt::MOB_TAG) {
            continue;
        }
        let at = [m.position.x as f64, m.position.y as f64, m.position.z as f64];
        if view::within(at, bed, view::SLEEP_RADIUS) {
            *problem = Some("not_safe".to_owned());
            return;
        }
    }
}
