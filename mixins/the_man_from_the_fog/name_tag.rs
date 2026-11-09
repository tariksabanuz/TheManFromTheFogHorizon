// No name over his head: a named mob shows its name while you look at it, and his body is named so
// that the client knows to draw him (`MOB_TAG`).
fn the_man_from_the_fog_hide_name(game: &crate::game::Game<'_>, entity_id: u32, visible: &mut bool) {

    let is_him = game
        .mobs
        .iter()
        .any(|m| m.id == entity_id && crate::game::native_the_man_from_the_fog_state::with(|s| s.tracked().contains(&m.id)));
    if is_him {
        *visible = false;
    }
}
