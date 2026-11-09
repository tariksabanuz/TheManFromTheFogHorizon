// The labels of the mod's page: how often he comes and what he may do.
fn the_man_from_the_fog_page(mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::native_the_man_from_the_fog_runtime as m;
    if mod_id == m::ID {
        buttons.extend(crate::game::native_the_man_from_the_fog_state::with(|s| s.buttons()));
    }
}
