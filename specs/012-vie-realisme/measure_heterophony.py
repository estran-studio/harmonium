#!/usr/bin/env python3
"""V4 : 20 graines × 7 profils × arousal/density 0, 0.5, 1, sur les snapshots publics.

cargo run --release -p harmonium_core --example heterophony_streams -- /tmp/harmonium-hetero/streams.jsonl 20
python3 specs/012-vie-realisme/measure_heterophony.py /tmp/harmonium-hetero/streams.jsonl

Les catégories sont des effets réalisés par porteuse écrite, pas des étiquettes internes
ni les tirages demandés qui peuvent s'abstenir. L'interprétation V2 n'est pas une variation
mélodique : offset/gate/vélocité sont ignorés pour l'identité écrite, mais utilisés pour
les collisions réellement sonores. Les notes de double corde restent dans la mesure.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import tomllib


def onset(n):
    return n['start_step'] * 1000 + n.get('performance', {}).get('offset', 0)


def end(n):
    p = n.get('performance', {})
    return onset(n) + max(1, n['duration_steps'] * p.get('gate', 1000) - p.get('breath', 0))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('streams', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    totals = defaultdict(Counter)
    profiles = {p.parent.name: tomllib.loads(p.read_text()) for p in Path('profiles').glob('*/profile.toml')}
    for line in args.streams.open():
        run = json.loads(line)
        key = (run['profile'], run['arousal'])
        c = totals[key]
        c['airs'] += 1
        for live, plain in zip(run['live'], run['plain']):
            position = live['position']
            lead = [n for n in live['notes'] if n['track'] == 0]
            second = [n for n in live['notes'] if n['track'] == 5]
            source = [n for n in plain['notes'] if n['track'] == 0]
            pulse = live['pulse']['steps_per_pulse']
            for time in range(0, live['steps'] * 1000, pulse * 1000):
                c['strong_second_collisions'] += sum(onset(a) <= time < end(a) and onset(b) <= time < end(b) and abs(a['pitch']-b['pitch']) % 12 in (1,2) for a in lead for b in second)
            if not run['enabled'] or not second or position['tour'] < 2:
                continue
            # Cadence : dernières attaques de chaque phrase explicite des profils.
            # Les profils distribués déclarent leurs frontières dans les données.
            profile = profiles[run['profile']]
            part = profile['form']['parts'][position['part']]
            if position['bar_in_strain'] in part.get('phrase_ends', []):
                last = max(n['start_step'] for n in lead)
                c['cadences'] += 1
                c['cadences_agreed'] += second[-1]['pitch'] % 12 in {n['pitch'] % 12 for n in lead if n['start_step'] == last}
            matched = []
            for i, n in enumerate(source):
                step = n['start_step']
                stop = source[i+1]['start_step'] if i+1 < len(source) else live['steps']
                group = [a for a in second if step*1000-490 <= onset(a) < stop*1000-490]
                lead_group = [a for a in lead if step*1000-490 <= onset(a) < stop*1000-490]
                def shape(notes):
                    return sorted((a['start_step'], a['pitch']) for a in notes)
                exact = [a for a in second if a['start_step'] == step and a['pitch'] == n['pitch']]
                target = next((a for a in group if a['start_step'] == step), None)
                held = next((a for a in second if a['start_step'] < step < a['start_step'] + a['duration_steps']), None)
                if held or (target and target['duration_steps'] > n['duration_steps']):
                    category = 'simplify'
                elif not group and not exact:
                    category = 'rest'
                elif group and shape(group) == shape(lead_group):
                    category = 'unison'  # identique à la mélodie, agréments partagés compris
                elif len(group) > 1 or (target and target.get('performance', {}).get('gate',1000) < 400):
                    category = 'ornament'
                elif target and abs(target['pitch']-n['pitch']) == 12:
                    category = 'octave'
                elif target and abs(target['pitch']-n['pitch']) % 12 in (3,4):
                    category = 'third'
                elif target and abs(target['pitch']-n['pitch']) % 12 in (8,9):
                    category = 'sixth'
                elif exact:
                    category = 'plain' if len(lead_group) > 1 else 'unison'
                else:
                    category = 'ornament'
                c[category] += 1
                c['carriers'] += 1
                if target:
                    matched.append((n['pitch'],target['pitch']))
            for (a,b),(x,y) in zip(matched,matched[1:]):
                c['contour_pairs'] += 1
                c['contour_agreed'] += ((x>a)-(x<a)) == ((y>b)-(y<b))
    result = []
    for (profile,arousal), c in sorted(totals.items()):
        row = {'profile':profile,'arousal':arousal,**c}
        row['rates_per_100'] = {op:100*c[op]/c['carriers'] if c['carriers'] else None for op in ('unison','plain','ornament','simplify','octave','third','sixth','rest')}
        row['varied_per_100'] = 100 - row['rates_per_100']['unison'] if c['carriers'] else None
        row['contour_agreement'] = c['contour_agreed']/c['contour_pairs'] if c['contour_pairs'] else None
        row['cadence_agreement'] = c['cadences_agreed']/c['cadences'] if c['cadences'] else None
        result.append(row)
        print(profile,arousal,json.dumps(row,ensure_ascii=False))
    output=args.output or args.streams.with_name('heterophony-measurements.json')
    output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    if any(row.get('strong_second_collisions',0) or row.get('cadence_agreement') not in (None,1) for row in result):
        raise SystemExit('Échec : collision de seconde ou cadence divergente')


if __name__ == '__main__':
    main()
