#!/usr/bin/env python3
"""Build the mod's resource pack for Horizon: `target/The-Man-From-The-Fog-Pack.zip`.

It holds what `tools/package.py` also puts in the `.hmod` (sounds, sounds.json, language file and the jumpscare
frames: everything under `local-assets/assets/`), so the mod can be added either way: as the `.hmod` alone, or
as the `.hnmod` / `.hmod` plus this pack in `resourcepacks/`. Pick the pack on the Resource Packs screen. The
files in `local-assets/` are not in the repository (other people's artwork and sounds).
"""
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def build():
    assets = ROOT / 'local-assets' / 'assets'
    if not assets.is_dir():
        raise SystemExit('no local-assets/assets: run tools/build_sound_pack.py and tools/prepare_jumpscare.py first')
    meta = {'pack': {'description': 'The Man From The Fog: sounds and jumpscare. By tariksabanuz',
                     'min_format': 97, 'max_format': 97}}
    out = ROOT / 'target' / 'The-Man-From-The-Fog-Pack.zip'
    out.parent.mkdir(exist_ok=True)
    with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr('pack.mcmeta', json.dumps(meta, indent=2))
        icon = assets / 'the_man_from_the_fog' / 'textures' / 'gui' / 'jumpscare_5.png'
        if icon.is_file():
            z.writestr('pack.png', icon.read_bytes())
        for path in sorted(assets.rglob('*')):
            if path.is_file():
                z.write(path, 'assets/' + path.relative_to(assets).as_posix())
    print(out, out.stat().st_size // 1024, 'KiB')


if __name__ == '__main__':
    build()
