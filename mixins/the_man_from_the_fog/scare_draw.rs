// The jumpscare screen, over everything: (only when the picture is there) black, then the picture (flickering between its two frames
// and shaking), then a fade to black. Without the picture it is only the black flash.
fn the_man_from_the_fog_scare_draw(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    use crate::game::native_the_man_from_the_fog_scare as scare;
    use crate::game::native_the_man_from_the_fog_state as st;

    let Some(ms) = st::with(|s| s.scare_elapsed_ms()) else { return };
    let Some(frame) = scare::frame_at(ms) else { return };
    let (w, h) = (batch.width, batch.height);
    let bytes: &[u8] = horizon_client::native_data!("the_man_from_the_fog", "jumpscare.rgba");
    if let Some(img) = scare::parse(bytes).filter(|i| i.is_picture()) {
        let (iw, half) = (img.width as f32, (img.height / 2) as f32);
        // Fill the screen, cropping what does not fit; the picture is dark at its edges.
        let k = (w / iw).max(h / half);
        let (dw, dh) = (iw * k, half * k);
        game.native_fill(batch, [0.0, 0.0, w, h], 0xFF00_0000);
        game.native_sprite(
            batch,
            crate::hud::MOD_IMAGE_SPRITE,
            [0.0, frame.frame as f32 * half, iw, half],
            [(w - dw) / 2.0 + frame.shake[0], (h - dh) / 2.0 + frame.shake[1], dw, dh],
        );
    }
    if frame.fade > 0.0 {
        let alpha = (frame.fade.clamp(0.0, 1.0) * 255.0) as u32;
        game.native_fill(batch, [0.0, 0.0, w, h], alpha << 24);
    }
}
