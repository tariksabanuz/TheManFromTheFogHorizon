# Design

| File | Role |
|---|---|
| `config.rs` | The settings, `key=value` persistence, clamping. |
| `fog.rs` | The stalker as a state machine (Idle, Watching, Chasing). No game types. |
| `model.rs` | His body as boxes, and the walk/idle swing. |
| `view.rs` | Fog thickening maths and the sleep radius. |
| `state.rs` | Shared state, one `Fog` per player, the page's buttons. |
| `player_tick.rs` | Server hook: reads the player, runs `fog.rs`, spawns/moves/removes his husk, breaks blocks. |
| `fog_view.rs` | Client hook (`render_fog`): thicker fog near him. |
| `sleep_block.rs` | Server hook (`can_player_sleep`): no sleeping near him. |
| `mob_draw.rs` | Client hook: draws `model.rs` for the husk tagged `the_man_from_the_fog`. |
| `listed.rs`, `settings_page.rs`, `settings_click.rs` | The row and page in Options > Mods. |

His body is a `minecraft:husk` (no sun burn) that is silent, with 300 health, 15 damage and extra speed, with its AI off while he
watches and on while he hunts. The server tags it with a custom name (name tag hidden) so that the client's
`mob_submit` hook can replace its model.
