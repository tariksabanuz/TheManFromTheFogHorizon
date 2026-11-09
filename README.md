# The Man From The Fog for Horizon

By tariksabanuz.

A stalking horror mob for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft 26.3), as a
single `.hnmod`. Inspired by the Forge/Fabric mod of the same name; written from scratch (see `NOTICE.md`).

## What he does

Built from the original's public description and its published config (see `docs/RESEARCH.md`):

- At night (every night or two by default) he appears far away and stands watching you.
- Unwatched for about 30 seconds he steps closer, but only when you are not looking; after about 45 seconds
  unwatched he hunts you.
- Meet his eyes and he either **vanishes** (with a harmless bolt of lightning) or **hunts** you.
- A hunt starts with the **Darkness** effect, lasts 400 to 1000 ticks, **climbs** walls, **breaks blocks** in its way
  (hardness 2 while he sees you; harder ones, chests and doors included, while he **searches** for you after losing sight),
  and ends in a flash of lightning. A hunter who lost you and is more than 30 blocks away fades.
- The **fog** thickens as he comes near. Nobody can **sleep** with him within 64 blocks.
- 300 health, 15 damage, fast, hard to push. He stays in the overworld unless `all_dimensions` is on, and does not
  appear on Peaceful. Each player has their own.
- He is tall and near-black with a small square head, two white eyes, arms that hang past the knees ending in
  three claws, and across the chest a mouth of red bars with white teeth.

Options > Mods > The Man From The Fog has buttons for the original's options (`spawn_rate`, `spawn_at_day`,
`all_dimensions`, `break_blocks`, `block_hardness`, `search_mechanic`, `climbing`, `darkness_effect`, `enable_fog` and
`fog_alpha_value`) and a few of this port's own (vanish chance, lightning), and **Summon him now**. They are saved in
`the-man-from-the-fog/config.txt`, where every key can also be edited by hand (`min_chase_duration`,
`max_chase_duration` and `vanish_distance` too).

## Status: 0.1.2, early

The behaviour (`fog.rs`), settings, model data and fog maths are plain Rust with 51 unit tests. The wiring to
Horizon's hooks (`player_tick.rs`, `mob_draw.rs`, `fog_view.rs`, `sleep_block.rs`) was written against the pinned
Horizon source but **has not been built in a real Horizon checkout yet** (the build environment had no network).
Report launcher errors as issues.

Not in 0.1.2, and why: his own sounds (the original's sound files are not available to a clean-room port, so
vanilla sounds are used); the full-screen jumpscare on the killing blow (`jumpscare`); crawling through one-block
gaps; his fights with iron golems; the original's later additions (Hanging Man, cultists, Tear of the Man, cassettes,
which only one unverified source describes); dedicated servers (single player and LAN host only).

## Building

    cargo test                      # unit tests of the pure modules
    python3 tools/package.py        # writes target/the_man_from_the_fog-<version>.hnmod

`mixins/the_man_from_the_fog/` is what ships; `src/lib.rs` only exists to test it. Pinned to Horizon
`fdc87aec` (see `compatibility.json`).

## License

MIT.
