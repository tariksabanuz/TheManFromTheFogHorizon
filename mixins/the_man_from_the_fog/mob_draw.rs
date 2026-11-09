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

    if !crate::game::native_the_man_from_the_fog_state::with(|s| s.tracked().contains(&mob.id)) {
        return;
    }
    // He drops to all fours when he moves: the whole body leans forward from the hip, the arms reach for the
    // ground, the legs trail and the head stays up.
    let crawl = model::crawl_amount(mob.walk_speed);
    let lean = model::CRAWL_LEAN * crawl;
    draw.push();
    if lean != 0.0 {
        draw.rotate_about([1.0, 0.0, 0.0], lean, model::HIP);
    }
    for part in model::PARTS.iter() {
        let group = model::group_of(part.name);
        let mut degrees = model::swing_degrees(part.motion, mob.walk_position, mob.walk_speed, mob.age) * (1.0 + 0.8 * crawl);
        draw.push();
        if crawl > 0.0 {
            let extra = model::crawl_extra(group, lean, crawl);
            if group == model::Group::Head {
                draw.rotate_about([1.0, 0.0, 0.0], extra, model::NECK_BASE);
            } else {
                degrees += extra;
            }
        }
        if part.splay != 0.0 {
            draw.rotate_about([0.0, 0.0, 1.0], part.splay, part.pivot);
        }
        if degrees != 0.0 {
            draw.rotate_about([1.0, 0.0, 0.0], degrees, part.pivot);
        }
        // Every face samples the middle of its flat-coloured sprite: with no UVs given, Horizon takes the
        // box's own coordinates, and a box this far from the origin reads outside the sprite.
        let face = CubeFace { sprite: part.paint.sprite(), uv: Some([4.0, 4.0, 12.0, 12.0]) };
        // Order: down, up, north, south, west, east. South is the way he faces.
        let faces = if part.front_only {
            [None, None, None, Some(face), None, None]
        } else {
            [Some(face); 6]
        };
        game.native_draw_cube(draw, part.from, part.to, faces, 0xFFFFFF);
        draw.pop();
    }
    draw.pop();
    *drawn = true;
}
