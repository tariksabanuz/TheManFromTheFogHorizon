// `/test`: makes him appear for the player who typed it, whatever the hour and the spawn timer say.
fn the_man_from_the_fog_test(
    game: &mut crate::game::Game<'_>,
    command: &str,
    ran: &mut bool,
) {
    use crate::game::native_the_man_from_the_fog_state as st;

    if command.trim() != "test" {
        return;
    }
    *ran = true;
    // The same key the per-player tick uses.
    let key: u128 = game.lan_acting_uuid.unwrap_or(0);
    let overworld = game.dim.id() == "minecraft:overworld";
    let cfg = st::with(|s| {
        s.request_summon(key);
        s.config()
    });
    let line = if overworld || cfg.all_dimensions {
        "The Man From The Fog is coming. Look away and he steps closer."
    } else {
        "He only appears in the overworld (all_dimensions is off). Go there and run /test again."
    };
    game.native_feedback(horizon_client::text::Text::literal(line));
}
