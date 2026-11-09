#!/usr/bin/env python3
"""Make the jumpscare frames the mod draws, from your own copy of the original's animation.

    python3 tools/prepare_jumpscare.py PATH/TO/TheManFromTheFog

reads AnimationJumpscare/*-0.png ... *-9.png (the lunge and the first loop of the original's animation),
scales them to 768 x 372 and writes `local-assets/assets/the_man_from_the_fog/textures/gui/jumpscare_1.png` ...
`jumpscare_10.png`: GUI textures of the mod, drawn by scare_draw.rs. Horizon's HUD atlas only takes textures up
to 1024 x 512. `tools/package.py` puts `local-assets/assets/` in the `.hmod`. `local-assets/` is not in the
repository: the artwork is someone else's. Needs Pillow.
"""
from pathlib import Path
import re
import sys

from PIL import Image

SIZE = (768, 372)
FRAMES = 10


def prepare(source):
    files = sorted(Path(source, 'AnimationJumpscare').glob('*.png'), key=lambda f: int(re.search(r'-(\d+)\.png$', f.name).group(1)))
    if len(files) < FRAMES:
        raise SystemExit(f'{len(files)} frames found, {FRAMES} needed')
    out = Path(__file__).resolve().parents[1] / 'local-assets' / 'assets' / 'the_man_from_the_fog' / 'textures' / 'gui'
    out.mkdir(parents=True, exist_ok=True)
    for old in out.glob('jumpscare_*.png'):
        old.unlink()
    for i, f in enumerate(files[:FRAMES]):
        Image.open(f).convert('RGB').resize(SIZE, Image.LANCZOS).convert('RGBA').save(out / f'jumpscare_{i + 1}.png', optimize=True)
    print(f'{FRAMES} frames in {out}')


if __name__ == '__main__':
    prepare(sys.argv[1])
