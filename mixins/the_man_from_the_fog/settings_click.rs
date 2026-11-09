// A click on a button of the mod's page.
fn the_man_from_the_fog_click(mod_id: &str, button: usize) {
    use crate::game::native_the_man_from_the_fog_runtime as m;
    if mod_id == m::ID {
        crate::game::native_the_man_from_the_fog_state::with(|s| s.click(button));
    }
}
