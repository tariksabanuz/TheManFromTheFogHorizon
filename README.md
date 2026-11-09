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

## Status: 0.1.12, early

The behaviour (`fog.rs`), settings, model data and fog maths are plain Rust with 51 unit tests. The wiring to
Horizon's hooks (`player_tick.rs`, `mob_draw.rs`, `fog_view.rs`, `sleep_block.rs`) was written against the pinned
Horizon source but **has not been built in a real Horizon checkout yet** (the build environment had no network).
Report launcher errors as issues.

Not in 0.1.12, and why: his own sounds (the original's sound files are not available to a clean-room port, so
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

## Sounds

The mod plays its sounds as events in the namespace `the_man_from_the_fog` (`spawn`, `step`, `whisper`,
`spotted`, `chase`, `jumpscare`, `vanish`). The sound files are not in this repository: they are other people's
work. Put your own `.ogg` files in a folder and run `python3 tools/build_sound_pack.py FOLDER`. That writes
`target/The-Man-From-The-Fog-Sounds.zip`, a resource pack to turn on in Horizon. Files are matched to events by a
word in their name (see the table at the top of the script); an event with no sound plays nothing.

## The jumpscare screen

When he is close in a hunt the screen shows the original's jumpscare animation (he lunges at the camera, then
jitters), shakes and fades out (`scare.rs`, `scare_draw.rs`); the blow that follows kills the player and he goes in
lightning. The frames are the mod's own GUI textures, `assets/the_man_from_the_fog/textures/gui/jumpscare_1.png` to
`_10.png`, inside the `.hmod`. They are not in this repository (they are someone else's artwork); without them
there is no jumpscare screen. To make them from your own copy of the original's images run
`python3 tools/prepare_jumpscare.py PATH/TO/TheManFromTheFog`; the next `python3 tools/package.py` puts them in the
`.hmod`. `local-assets/` is not committed.


## Resource pack

`python3 tools/build_resource_pack.py` writes `target/The-Man-From-The-Fog-Pack.zip`: the same sounds,
`sounds.json`, language file and jumpscare frames that go into the `.hmod`, as a Horizon resource pack. Put it in
`resourcepacks/` and pick it on the Resource Packs screen if the `.hmod` alone does not bring them in.
