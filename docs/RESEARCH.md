# Research notes

What this port follows, and how sure each fact is. Sources: the CurseForge project page and its 1.0.0 and 1.3 file
changelogs, the Modrinth page (config table), the CurseForge blog FAQ, wiki-mine, minecraft-guides.com and modbay.

**Confirmed by the project pages (high):** author M_Productions; Forge, GeckoLib; config options `spawn_rate`
(often/normal/rare, default normal), `jumpscare` (true), `max_chase_duration` (1000), `min_chase_duration` (400),
`climbing` (true), `vanish_distance` (30), `spawn_at_day` (false), `darkness_effect` (true), `all_dimensions` (false),
`break_blocks` (true), and from 1.3 `search_mechanic`, `block_hardness`, `block_hardness_search`, `enable_fog`,
`fog_alpha_value`; 1.0.0: 300 health, 15 damage, lightning when he vanishes and attacks, climbing, crawling through
one-block gaps, breaking glass panes, no spawn on Peaceful, jumpscare, two spawn sounds and an attack loop;
1.3: the search mechanic (he searches for the nearest player after losing sight and digs through blocks of hardness 2
or less in a chase).

**Medium:** minecraft-guides.com gives the timings used here (about 30 s unwatched to step closer, about 45 s to hunt,
a hunt up to about 50 s ending in fake lightning), that sleeping is blocked near him and that a hunt starts with
Darkness. It also describes a Hanging Man, cultist houses, the Tear of the Man and cassettes; those are not
implemented because nothing else confirms them.

**Low (not used):** themanfromthefog.com (a JSON config and a `/manfog` command that no other source mentions).

Numbers the sources do not give (spawn distance, step size, chance that eye contact vanishes him, the hardness while
searching, the exact spawn-rate delays) are this port's own and marked as such in `config.rs`.
