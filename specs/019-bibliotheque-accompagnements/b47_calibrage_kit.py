#!/usr/bin/env python
"""B4 + B7 : calage du niveau de chaque voix de batterie sur le kit GM de GeneralUser GS.

Pour chaque rôle, un profil d'essai (copie de `gmd_funk`) ne joue que cette voix, 4 coups par mesure,
16 mesures, à vélocité 50 puis 100 ; `ffmpeg ebur128` donne le LUFS intégré en solo de la piste. Les deux
points alimentent `CALIBRATION` de `b47_batterie.py`.

    python b47_calibrage_kit.py [programme du kit, 0 par défaut] [binaire harmonium-cli]

Le CLI se lance depuis `~/Project/bascanada/harmonium` (pour `assets/`) ; rendus dans `/tmp/b47/`.
"""
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

WT = Path(__file__).resolve().parents[2]
ENG = os.path.expanduser('~/Project/bascanada/harmonium')
CLI = sys.argv[2] if len(sys.argv) > 2 else f'{WT}/target/debug/harmonium-cli'
os.makedirs('/tmp/b47/cal', exist_ok=True)
os.makedirs('/tmp/b47/wav', exist_ok=True)
VOICES={'kick':(36,'PercussionLow'),'snare':(38,'PercussionHigh'),'sidestick':(37,'PercussionHigh'),'hat':(42,'PercussionHigh'),'pedal':(44,'PercussionHigh'),'open':(46,'PercussionHigh'),'ride':(51,'PercussionHigh'),'bell':(53,'PercussionHigh'),'crash':(49,'PercussionHigh'),'tom_high':(48,'PercussionHigh'),'tom_mid':(45,'PercussionHigh'),'tom_low':(43,'PercussionHigh')}
prog=sys.argv[1] if len(sys.argv)>1 else '0'
def build(name,v,vel):
    d=f'/tmp/b47/cal/{name}'
    if os.path.exists(d): shutil.rmtree(d)
    shutil.copytree(f'{WT}/profiles/gmd_funk',d)
    p=open(f'{d}/profile.toml').read()
    p=p.replace('arc = "arc.toml"\n','')
    p=re.sub(r'percussion_program = \d+',f'percussion_program = {prog}',p)
    open(f'{d}/profile.toml','w').write(p)
    os.remove(f'{d}/arc.toml')
    note,track=VOICES[v]
    t=f'low_note = 36\nhigh_note = 42\nlow_velocity = 64\nhigh_velocity = 64\npercussion_gain = 1.0\nvelocity_humanize = 0\n\n[[voices]]\nname = "{v}"\nnote = {note}\nvelocity = {vel}\ntrack = "{track}"\n\n'
    for dens in (0,1,2):
        t+=f'[[patterns]]\ndensity = {dens}\nhits = [\n'+''.join(f'  {{ step = {s}, voice = "{v}" }},\n' for s in (0,4,8,12))+']\n\n'
    open(f'{d}/percussion.toml','w').write(t)
    return d
def lufs(wav):
    r=subprocess.run(['ffmpeg','-nostats','-i',wav,'-af','ebur128','-f','null','-'],capture_output=True,text=True).stderr
    m=re.findall(r'\n\s+I:\s+(-?[\d.]+) LUFS',r)
    return float(m[-1]) if m else None
out={}
for v,(note,track) in VOICES.items():
    pts=[]
    for vel in (50,100):
        d=build(f'{v}_{vel}',v,vel)
        wav=f'/tmp/b47/wav/cal_{v}_{vel}.wav'
        subprocess.run([CLI,'--export','--seed','1','--bars','16','--profile',d,'--solo-track',track,'--record-wav',wav],cwd=ENG,capture_output=True)
        pts.append((vel,lufs(wav)))
    out[v]=pts
    print(v,pts,flush=True)
json.dump(out,open(f'/tmp/b47/cal_prog{prog}.json','w'))
