// The fog thickens as he comes close: the client's final fog (`render_fog`) is pulled in by how near
// the closest of him is to the camera (`enable_fog` and `fog_alpha_value`).
fn the_man_from_the_fog_thick_fog(
    game: &crate::game::Game<'_>,
    camera: [f64; 3],
    fog: &mut horizon_client::fog::FogData,
) {
    use crate::game::native_the_man_from_the_fog_state as st;
    use crate::game::native_the_man_from_the_fog_view as view;

    let cfg = st::with(|s| s.config());
    if !cfg.enable_fog || cfg.fog_alpha_value <= 0.0 {
        return;
    }
    let mut nearest = f64::MAX;
    for m in game.mobs.iter() {
        if m.removed || !st::with(|s| s.tracked().contains(&m.id)) {
            continue;
        }
        let at = [m.position.x as f64, m.position.y as f64, m.position.z as f64];
        let d = ((at[0] - camera[0]).powi(2) + (at[1] - camera[1]).powi(2) + (at[2] - camera[2]).powi(2)).sqrt();
        nearest = nearest.min(d);
    }
    let k = view::closeness(nearest, view::FOG_RANGE);
    if k <= 0.0 {
        return;
    }
    let (start, end) = view::thicken(fog.environmental_start, fog.environmental_end, k, cfg.fog_alpha_value);
    fog.environmental_start = start;
    fog.environmental_end = end;
    let (start, end) = view::thicken(fog.render_distance_start, fog.render_distance_end, k, cfg.fog_alpha_value);
    fog.render_distance_start = start;
    fog.render_distance_end = end;
}
