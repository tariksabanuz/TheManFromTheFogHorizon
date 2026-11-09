// The mod's row in Options > Mods; its page (settings_page.rs) holds the buttons.
fn the_man_from_the_fog_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::native_the_man_from_the_fog_runtime as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
