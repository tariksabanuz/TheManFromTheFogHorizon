// The jumpscare screen, over everything: black, then the picture (flickering between its two frames
// and shaking), then a fade to black. The two frames are the mod's own GUI textures
// (`assets/the_man_from_the_fog/textures/gui/jumpscare_1.png` and `_2`), so no upload is needed.
fn the_man_from_the_fog_scare_draw(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    use crate::game::native_the_man_from_the_fog_scare as scare;
    use crate::game::native_the_man_from_the_fog_state as st;

    let Some(ms) = st::with(|s| s.scare_elapsed_ms()) else { return };
    let Some(frame) = scare::frame_at(ms) else { return };
    let name = if frame.frame == 0 { "the_man_from_the_fog:gui/jumpscare_1" } else { "the_man_from_the_fog:gui/jumpscare_2" };
    // Without the picture there is no black screen either.
    if !game.gui.atlas.sprites.contains_key(name) {
        return;
    }
    let (w, h) = (batch.width, batch.height);
    game.native_fill(batch, [0.0, 0.0, w, h], 0xFF00_0000);
    let (iw, ih) = (960.0_f32, 496.0_f32);
    // Fill the screen, cropping what does not fit; the picture is dark at its edges.
    let k = (w / iw).max(h / ih);
    let (dw, dh) = (iw * k, ih * k);
    game.native_sprite(
        batch,
        name,
        [0.0, 0.0, iw, ih],
        [(w - dw) / 2.0 + frame.shake[0], (h - dh) / 2.0 + frame.shake[1], dw, dh],
    );
    if frame.fade > 0.0 {
        let alpha = (frame.fade.clamp(0.0, 1.0) * 255.0) as u32;
        game.native_fill(batch, [0.0, 0.0, w, h], alpha << 24);
    }
}
