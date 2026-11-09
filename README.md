# The Man From The Fog for Horizon

By tariksabanuz.

A stalking horror mob for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft 26.3), as a
single `.hnmod`. Inspired by the Forge/Fabric mod of the same name; written from scratch (see `NOTICE.md`).

## What he does

- At night (every night or two by default) he appears far away, out in the dark, and stands watching you.
- While you ignore him he steps closer, but only when you are not looking, so you never see him move.
- Meet his eyes and he either **vanishes** (with a harmless bolt of lightning) or **hunts** you.
- Ignore him long enough and he hunts you anyway.
- A hunt follows you, breaks the blocks in its way (up to a hardness you choose) and ends after a while.
- He is tall, near-black, with long arms and a glowing red mouth on his chest.

Options > Mods > The Man From The Fog has buttons for: on/off, night only or also by day, how often,
the chance that eye contact makes him vanish, block breaking and its hardness limit, lightning, and
**Summon him now** (to see him without waiting). The choices are saved in `the-man-from-the-fog/config.txt`.

## Status: 0.1.0, early

The behaviour (`fog.rs`), settings and model data are plain Rust with 37 unit tests. The wiring to Horizon's
hooks (`player_tick.rs`, `mob_draw.rs`) was written against the pinned Horizon source but **has not been built
in a real Horizon checkout yet** (the build environment had no network). Report launcher errors as issues.

Not in 0.1.0: his own sounds (vanilla ones are used), the darkness effect and full-screen jumpscare of the
original, climbing walls, and dedicated-server use (single player and LAN host only).

## Building

    cargo test                      # unit tests of the pure modules
    python3 tools/package.py        # writes target/the_man_from_the_fog-<version>.hnmod

`mixins/the_man_from_the_fog/` is what ships; `src/lib.rs` only exists to test it. Pinned to Horizon
`fdc87aec` (see `compatibility.json`).

## License

MIT.
