#!/usr/bin/env python3
"""R1 : tableaux réel / généré des trois nouveaux styles, depuis les sorties du labo.

Entrées (voir la section R1 du plan 013 pour les commandes qui les produisent) :
  - `trad-structure` : `<structure>/<tradition>-<type>/stats.json` (blocs `real` et `generated`) ;
  - `trad-compare` : `<compare>/<style>/report.json` ;
  - `measure_ornaments.py rates` : `<ornaments>/rates/rates.json`.
Sortie : Markdown sur stdout. Python standard seulement.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

STYLES = {
    'gigue_acadienne': 'acadienne-jig',
    'reel_britannique_coloniale': 'britannique_coloniale-reel',
    'valse_britannique_coloniale': 'britannique_coloniale-waltz',
}
ABBR = {'gigue_acadienne': 'GA', 'reel_britannique_coloniale': 'RB', 'valse_britannique_coloniale': 'VB'}


def rate(transformation, kind='exact'):
    return transformation['counts'][kind] / transformation['opportunities'] if transformation['opportunities'] else float('nan')


def pair_share(block, pattern):
    for row in block['structures']:
        if row['pattern'] == pattern:
            return row['share']
    return 0.0


def indicators(block):
    rest = block['phrases']['rest_by_position']
    t = block['transformations']
    return {
        'm.5–6 = m.1–2 (parties, %)': 100 * block['phrases']['opening_reprise_rate'],
        'paires « a b a c » (%)': 100 * pair_share(block, 'a b a c'),
        'paires « a b c d » (%)': 100 * pair_share(block, 'a b c d'),
        'copie A:m.5 = m.1 (%)': 100 * rate(t['A:m5/m1']),
        'copie A:m.6 = m.2 (%)': 100 * rate(t['A:m6/m2']),
        'copie B:m.5 = m.1 (%)': 100 * rate(t['B:m5/m1']),
        'copie B:m.6 = m.2 (%)': 100 * rate(t['B:m6/m2']),
        'copie B:m.7 = A:m.7 (%)': 100 * rate(t['B:m7/A:m7']),
        'repos tonique A:m.4 (%)': 100 * rest['A:m4']['tonic_cadence_rate'],
        'repos tonique A:m.8 (%)': 100 * rest['A:m8']['tonic_cadence_rate'],
        'repos tonique B:m.4 (%)': 100 * rest['B:m4']['tonic_cadence_rate'],
        'repos tonique B:m.8 (%)': 100 * rest['B:m8']['tonic_cadence_rate'],
        'finale longue A:m.4 (%)': 100 * rest['A:m4']['long_final_rate'],
        'finale longue A:m.8 (%)': 100 * rest['A:m8']['long_final_rate'],
        'finale longue B:m.4 (%)': 100 * rest['B:m4']['long_final_rate'],
        'finale longue B:m.8 (%)': 100 * rest['B:m8']['long_final_rate'],
        'croches continues (mesures, %)': 100 * block['rhythm']['continuous_eighth_rate'],
        'entropie rythmique (bits/mesure)': block['rhythm']['entropy_bits'],
    }


def structure_table(root):
    data = {name: json.loads((root / ref / 'stats.json').read_text()) for name, ref in STYLES.items()}
    rows = {name: (indicators(d['real']), indicators(d['generated'])) for name, d in data.items()}
    labels = list(next(iter(rows.values()))[0])
    header = '| Indicateur | ' + ' | '.join(f'{ABBR[n]} réel / généré' for n in STYLES) + ' |'
    lines = [header, '|---|' + '---:|' * len(STYLES)]
    for label in labels:
        cells = []
        for name in STYLES:
            real, generated = rows[name]
            cells.append(f'{real[label]:.1f} / {generated[label]:.1f}' if 'bits' not in label else f'{real[label]:.2f} / {generated[label]:.2f}')
        lines.append(f'| {label} | ' + ' | '.join(cells) + ' |')
    counts = ' ; '.join(f'{ABBR[n]} {d["real"]["tunes"]} airs réels ({d["real"]["positional_tunes"]} positionnables), '
                        f'{d["generated"]["tunes"]} exports ({d["generated"]["positional_tunes"]} positionnés)' for n, d in data.items())
    return '\n'.join(lines), counts


def compare_table(root):
    keys = ['intervals__stepwise_ratio', 'register__ambitus', 'register__mean_pitch_part0', 'register__b_minus_a_mean_pitch',
            'harmony__chords_per_bar', 'form__repeat_identity', 'form__share_identical_bars_5_6_vs_1_2']
    reports = {name: json.loads((root / name / 'report.json').read_text()) for name in STYLES}
    by_name = {name: {row['key']: row for row in report['deviations']} for name, report in reports.items()}
    lines = ['| Mesure `trad-compare` | ' + ' | '.join(f'{ABBR[n]} réel / généré (z)' for n in STYLES) + ' |', '|---|' + '---:|' * len(STYLES)]
    for key in keys:
        cells = []
        for name in STYLES:
            row = by_name[name].get(key)
            cells.append('—' if row is None else f'{row["real_value"]:.3g} / {row["generated_value"]:.3g} ({row["z"]:+.1f})')
        lines.append(f'| `{key}` | ' + ' | '.join(cells) + ' |')
    return '\n'.join(lines)


def ornaments_table(path):
    """Occurrences observées / attendues sur 20 graines et 34 mesures (`measure_ornaments.py rates`)."""
    data = json.loads(path.read_text())
    lines = ['| Style | Type | Plancher corpus /100 notes | Configuré ×échelle | Observé / attendu a=0 | a=0,5 | a=1 | Verdict a=0,5 |',
             '|---|---|---:|---:|---:|---:|---:|---|']
    for name in STYLES:
        for kind, row in data[name]['types'].items():
            by = row['by_arousal']
            cells = ' | '.join(f'{by[l]["observed"]:g} / {by[l]["expected"]:.1f}' for l in ('a0', 'a5', 'a10'))
            lines.append(f'| {ABBR[name]} | `{kind}` | {row["corpus_floor_per_100_notes"]:.3f} | {row["configured_per_100_notes"]:.3f} | '
                         f'{cells} | {by["a5"]["verdict"]} |')
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--structure', type=Path, required=True)
    parser.add_argument('--compare', type=Path, required=True)
    parser.add_argument('--ornaments', type=Path)
    args = parser.parse_args()
    table, counts = structure_table(args.structure)
    print(table)
    print()
    print(counts)
    print()
    print(compare_table(args.compare))
    if args.ornaments:
        print()
        print(ornaments_table(args.ornaments))


if __name__ == '__main__':
    main()
