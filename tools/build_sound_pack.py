#!/usr/bin/env python3
"""Build the resource pack that holds the mod's sounds, from a folder of .ogg files you own.

The sound files are NOT part of this repository (they are other people's work: the original mod's
and sound libraries'). Put them in a folder, run

    python3 tools/build_sound_pack.py PATH/TO/FOLDER

and turn on `target/The-Man-From-The-Fog-Sounds.zip` as a resource pack in Horizon. The mod plays the
events below by name (`fog.rs`, `Cue::event`); a missing sound just plays nothing.

Files are matched to events by a word in their name. The sounds heard from where he stands are made
mono (a stereo file has no direction and no distance in Minecraft's sound engine).
Needs ffmpeg with libvorbis.
"""
import json
from pathlib import Path
import re
import subprocess
import sys
import zipfile

NS = 'the_man_from_the_fog'
# event: (words that pick its files, mono?, stream?, attenuation distance)
EVENTS = {
    'spawn': (['spawn'], True, False, 64),
    'step': (['passby', 'look-version'], True, False, 48),
    'whisper': (['whisper', 'creepy-ambient'], True, False, 48),
    'spotted': (['spotted', 'riser'], False, False, 16),
    'chase': (['chased'], False, False, 16),
    'jumpscare': (['jumpscare', 'screams'], False, False, 16),
    'vanish': (['reversed'], True, False, 48),
}
STREAM_OVER_SECONDS = 20


def duration(path):
    out = subprocess.run(['ffprobe', '-v', 'error', '-show_entries', 'format=duration',
                          '-of', 'csv=p=0', str(path)], capture_output=True, text=True).stdout
    return float(out.strip() or 0)


def clean(name):
    name = re.sub(r'^[0-9a-f]{8}-', '', Path(name).stem.lower())  # an upload's random prefix
    return re.sub(r'[^a-z0-9_]+', '_', name).strip('_')


def build(folder):
    sources = sorted(Path(folder).glob('*.ogg'))
    if not sources:
        raise SystemExit(f'no .ogg files in {folder}')
    out_dir = Path(__file__).resolve().parents[1] / 'target'
    out_dir.mkdir(exist_ok=True)
    pack = out_dir / 'The-Man-From-The-Fog-Sounds.zip'
    used, sounds_json, converted = set(), {}, {}
    for event, (words, mono, _stream, attenuation) in EVENTS.items():
        entries = []
        for src in sources:
            if not any(w in src.name.lower() for w in words) or src in used:
                continue
            used.add(src)
            name = clean(src.name)
            tmp = out_dir / f'_{name}.ogg'
            cmd = ['ffmpeg', '-y', '-v', 'error', '-i', str(src), '-vn'] + (['-ac', '1'] if mono else []) \
                + ['-c:a', 'libvorbis', '-q:a', '4', str(tmp)]
            subprocess.run(cmd, check=True)
            converted[f'assets/{NS}/sounds/{name}.ogg'] = tmp
            entry = {'name': f'{NS}:{name}', 'volume': 1.0}
            if duration(src) > STREAM_OVER_SECONDS:
                entry['stream'] = True
            if mono:
                entry['attenuation_distance'] = attenuation
            entries.append(entry)
        if entries:
            sounds_json[event] = {'sounds': entries, 'subtitle': f'subtitles.{NS}.{event}'}
    meta = {'pack': {'description': 'Sounds for The Man From The Fog. By tariksabanuz',
                     'min_format': 97, 'max_format': 97}}
    with zipfile.ZipFile(pack, 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr('pack.mcmeta', json.dumps(meta, indent=2))
        z.writestr(f'assets/{NS}/sounds.json', json.dumps(sounds_json, indent=2))
        z.writestr(f'assets/{NS}/lang/en_us.json', json.dumps(
            {f'subtitles.{NS}.{e}': 'The Man From The Fog ' + e for e in sounds_json}, indent=2))
        for name, tmp in converted.items():
            z.write(tmp, name)
    for tmp in converted.values():
        tmp.unlink()
    print(f'{pack} ({pack.stat().st_size // 1024} KiB)')
    for event, data in sounds_json.items():
        print(f'  {event}: {len(data["sounds"])} sound(s)')
    left = [s.name for s in sources if s not in used]
    if left:
        print('not used:', *left, sep='\n  ')


if __name__ == '__main__':
    build(sys.argv[1] if len(sys.argv) > 1 else '.')
