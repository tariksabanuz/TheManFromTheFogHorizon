#!/usr/bin/env python3
"""Make the jumpscare picture the mod draws, from your own copy of the original's images.

    python3 tools/prepare_jumpscare.py PATH/TO/TheManFromTheFog

reads RandomPics/man_jumpscare_1.png and RandomPics/manjumpscare_2.png, scales both to 960 wide and writes
`local-assets/jumpscare.rgba` (two frames, one above the other; see mixins/the_man_from_the_fog/scare.rs).
`tools/package.py` puts it in the .hnmod in place of the stand-in. `local-assets/` is not in the
repository: the artwork is someone else's. Needs Pillow.
"""
from pathlib import Path
import struct
import sys

from PIL import Image

WIDTH = 960  # two frames of this width must fit Horizon's 1024 x 1024 image slot, one above the other


def prepare(source):
    frames = []
    for name in ('man_jumpscare_1.png', 'manjumpscare_2.png'):
        im = Image.open(Path(source) / 'RandomPics' / name).convert('RGBA')
        frames.append(im.resize((WIDTH, round(im.height * WIDTH / im.width)), Image.LANCZOS))
    height = min(f.height for f in frames)
    if height * 2 > 1024:
        raise SystemExit(f'two frames {WIDTH} x {height} do not fit the 1024 x 1024 slot')
    sheet = Image.new('RGBA', (WIDTH, height * 2), (0, 0, 0, 255))
    for i, f in enumerate(frames):
        sheet.paste(f.crop((0, 0, WIDTH, height)), (0, i * height))
    out = Path(__file__).resolve().parents[1] / 'local-assets' / 'jumpscare.rgba'
    out.parent.mkdir(exist_ok=True)
    out.write_bytes(struct.pack('<II', sheet.width, sheet.height) + sheet.tobytes())
    print(f'{out} ({sheet.width} x {sheet.height}, {out.stat().st_size // 1024} KiB)')


if __name__ == '__main__':
    prepare(sys.argv[1])
