// Draws him in place of the husk his body is made from: the boxes of `model.rs` under the mob's
// pose (feet at the origin, +z forward), legs and arms swinging with the walk cycle.
fn the_man_from_the_fog_draw(
    game: &crate::game::Game<'_>,
    mob: &crate::game::native_mob_draw::NativeMobRender<'_>,
    draw: &mut crate::game::native_block_entity_draw::BlockEntityDraw,
    drawn: &mut bool,
) {
    use crate::game::native_block_entity_draw::CubeFace;
    use crate::game::native_the_man_from_the_fog_model as model;
    use crate::game::native_the_man_from_the_fog_runtime as rt;

    if mob.custom_name != Some(rt::MOB_TAG) {
        return;
    }
    for part in model::PARTS.iter() {
        let degrees = model::swing_degrees(part.motion, mob.walk_position, mob.walk_speed, mob.age);
        draw.push();
        if degrees != 0.0 {
            draw.rotate_about([1.0, 0.0, 0.0], degrees, part.pivot);
        }
        let face = CubeFace { sprite: part.paint.sprite(), uv: None };
        // Order: down, up, north, south, west, east. South is the way he faces.
        let faces = if part.front_only {
            [None, None, None, Some(face), None, None]
        } else {
            [Some(face); 6]
        };
        game.native_draw_cube(draw, part.from, part.to, faces, 0xFFFFFF);
        draw.pop();
    }
    *drawn = true;
}
