// Sends the jumpscare picture to the HUD atlas once, from the first frame that has room for it.
fn the_man_from_the_fog_scare_upload(game: &mut crate::game::Game<'_>) {
    use crate::game::native_the_man_from_the_fog_scare as scare;
    use crate::game::native_the_man_from_the_fog_state as st;

    if !st::with(|s| s.scare_needs_upload()) {
        return;
    }
    let bytes: &[u8] = horizon_client::native_data!("the_man_from_the_fog", "jumpscare.rgba");
    let done = match scare::parse(bytes) {
        Some(img) if img.is_picture() => game.native_mod_image(img.width, img.height, img.rgba),
        // The stand-in or a bad file: there is nothing to send.
        _ => true,
    };
    if done {
        st::with(|s| s.mark_scare_uploaded());
    }
}
