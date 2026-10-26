#!/usr/bin/env python3
"""R1 : profils de style construits depuis le corpus du labo (aucune valeur saisie à la main
pour la structure, les cellules, les ornements ni l'interprétation écrite).

    harmonium_lab/.venv/bin/python specs/013-moteur-dans-le-jeu/r1_profils.py selftest
    harmonium_lab/.venv/bin/python specs/013-moteur-dans-le-jeu/r1_profils.py build

`selftest` rejoue les formules sur quatre profils distribués (gigue québécoise, reel acadien,
hornpipe et gigue britanniques, valse québécoise) et exige les mêmes `form.parts`, `ornaments`
et règles `performance` que leurs fichiers : la mécanique est celle de P3-T1/T2/T3, V2 et V3b.
`build` écrit `profiles/<id>/` pour les trois nouveaux styles. Entrées : références versionnées du
labo (`references/structure`, `references/ornaments`), `mine-cells`, `measure_endings.py` et le
manifeste du corpus ; sortie purement déterministe.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
import tomllib
from collections import Counter
from fractions import Fraction
from pathlib import Path

ENGINE = Path(__file__).resolve().parents[2]
LAB = Path('/Users/william.quintal/Project/bascanada/harmonium_lab')
PROFILES = ENGINE / 'profiles'
POSITIONS = (1, 2, 3, 5, 6, 7)
LOCAL_REF = {2: 1, 3: 1, 5: 1, 6: 2, 7: 3}
CORPORA = {'quebecoise': 'trad', 'acadienne': 'acadienne', 'britannique_coloniale': 'britannique_coloniale'}


def w4(rate):
    """Poids sur 10 000, arrondi à la demi supérieure (valeur positive)."""
    return int(rate * 10000 + 0.5)


def structure(ref):
    return json.loads((LAB / 'references/structure' / f'{ref}.json').read_text())['real']


def ornament_ref(ref):
    return json.loads((LAB / 'references/ornaments' / f'{ref}.json').read_text())


# ------------------------------------------------------------------ phrases (P3-T1)

def shifts_toml(real, part, position, ref):
    items = real['sequence_shifts'].get(f'{part}:m{position}/m{ref}', [])
    out = []
    for item in items:
        try:
            degrees = int(item['pattern'])
        except ValueError:
            continue
        if degrees == 0 or not -14 <= degrees <= 14 or item['count'] <= 0:
            continue
        out.append(f'{{ degrees = {degrees}, weight = {item["count"]} }}')
    return out


def choose(items):
    return '{ choose = [' + ', '.join(f'{{ role = {role}, weight = {weight} }}' for role, weight in items) + '] }'


def exact_rate(transformation, kind):
    """Effectif / occasions : le taux publié à six décimales ferait basculer les arrondis à 10 000."""
    return transformation['counts'][kind] / transformation['opportunities']


def local_role(real, part, position):
    ref = LOCAL_REF[position]
    t = real['transformations'][f'{part}:m{position}/m{ref}']
    weights = [('copy', w4(exact_rate(t, 'exact'))), ('seq', w4(exact_rate(t, 'sequence'))),
               ('end', w4(exact_rate(t, 'varied_ending')))]
    other = None
    if part == 'B' and position == 7:
        other = w4(exact_rate(real['transformations']['B:m7/A:m7'], 'exact'))
    return ref, weights, other


def bar_role(real, part, position):
    ref, weights, part_copy = local_role(real, part, position)
    scale = 1.0
    if part_copy is not None:
        scale = 1.0 - exact_rate(real['transformations']['B:m7/A:m7'], 'exact')
    shifts = shifts_toml(real, part, position, ref)
    items = []
    used = 0
    for kind, weight in weights:
        weight = int(weight * scale + 0.5) if part_copy is not None else weight
        if weight <= 0:
            continue
        if kind == 'seq':
            if not shifts:
                continue
            role = f'{{ SequenceOf = {{ bar = {ref}, shift = [{", ".join(shifts)}] }} }}'
        elif kind == 'copy':
            role = f'{{ CopyOf = {ref} }}'
        else:
            role = f'{{ VariedEndOf = {ref} }}'
        items.append((role, weight))
        used += weight
    if part_copy:
        items.append(('{ CopyOfPart = { part = "A", bar = 7 } }', part_copy))
        used += part_copy
    items.append(('"Free"', 10000 - used))
    return choose(items) if len(items) > 1 else '"Free"'


def cadence_role(real, part, position):
    rest = real['phrases']['rest_by_position'][f'{part}:m{position}']
    tonic = w4(rest['tonic_cadence_rate'])
    half = w4(max(0.0, rest['long_final_rate'] - rest['tonic_cadence_rate']))
    return choose([item for item in (('"CadTonic"', tonic), ('"CadHalf"', half), ('"CadOpen"', 10000 - tonic - half))
                   if item[1] > 0])


def rhythm_patterns(real, cell_units):
    rows = []
    for cell in real['rhythm']['cells']:
        tokens = cell['pattern'].split()
        if not all(re.fullmatch(r'[1-9]\d*', t) for t in tokens):
            continue
        durations = [int(t) for t in tokens]
        if sum(durations) != cell_units or len(set(durations)) < 2:
            continue
        rows.append((durations, cell['count']))
    return rows


def rhythm_toml(real, cell_units):
    rows = rhythm_patterns(real, cell_units)
    lines = ['rhythm = [']
    for bar in (4, 8):
        lines.append(f'  {{ bar = {bar}, patterns = [')
        for durations, count in rows:
            lines.append(f'      {{ durations = {durations}, weight = {count} }},')
        lines.append('  ] },')
    lines.append(']')
    return '\n'.join(lines)


def second_ending(endings, ref, part):
    counts = endings[ref]['counts'].get(f'eight_{part}:second_written', {})
    tonic, fifth, other = counts.get('tonic', 0), counts.get('fifth', 0), counts.get('other', 0)
    if tonic + fifth + other == 0:
        return None
    items = [f'{{ role = "{role}", weight = {weight} }}'
             for role, weight in (('CadTonic', tonic), ('CadFifth', fifth), ('CadOther', other)) if weight > 0]
    return 'second_ending = { choose = [' + ', '.join(items) + '] }'


def form_toml(real, endings, ref, cell_units, b_offset=2):
    out = ['[form]', 'sequence = [0, 0, 1, 1]']
    for index, part in enumerate(('A', 'B')):
        out += ['[[form.parts]]', f'name = "{part}"', f'melody_center_offset = {(0, b_offset)[index]}',
                'phrase_ends = [4, 8]', rhythm_toml(real, cell_units), 'roles = [', '  "Open",']
        for position in range(2, 9):
            role = cadence_role(real, part, position) if position in (4, 8) else bar_role(real, part, position)
            out.append(f'  {role},')
        out.append(']')
        ending = second_ending(endings, ref, part)
        if ending:
            out.append(ending)
    return '\n'.join(out) + '\n'


# ------------------------------------------------------------------ ornements (V3b)

KINDS = ['grace_single', 'grace_multiple', 'roll', 'turn', 'triplet_repeated', 'double_stop', 'chord', 'trill']
VELOCITY = {'grace_single': 0.75, 'grace_multiple': 0.7, 'roll': 0.8, 'turn': 0.8, 'triplet_repeated': 0.9,
            'double_stop': 0.8, 'chord': 0.75, 'trill': 0.8}


def fmt(value):
    text = f'{value:.6f}'.rstrip('0').rstrip('.')
    return text if '.' in text else text + '.0'


def min_carrier(ref, kind):
    """Durée porteuse minimale, en croches : minimum observé sous 30 occurrences ; sinon plus petite durée dont
    le taux par note atteint 10 % du meilleur (classes d'au moins 5 occurrences). Triolet : durée totale remplacée (3 × première note)."""
    proposal = ref['engine_proposal']['types'][kind]
    factor = 3 if kind == 'triplet_repeated' else 1
    if proposal['occurrences'] < 30:
        return Fraction(proposal['min_carrier_eighths_observed']) * factor
    rates = {Fraction(d): v['by_type'][kind]['per_100_notes'] if kind in v['by_type'] else 0.0
             for d, v in ref['by_position']['carrier_duration_eighths'].items()
             if v['notes'] > 0 and v['by_type'].get(kind, {}).get('count', 0) >= 5}
    best = max(rates.values())
    return min(d for d, r in rates.items() if r >= 0.1 * best) * factor


def number(value):
    value = Fraction(value)
    return fmt(float(value)) if value.denominator != 1 else f'{int(value)}.0'


def ornaments_toml(ref_name, rate_scale=4.0):
    ref = ornament_ref(ref_name)
    types = ref['engine_proposal']['types']
    positioned = {kind: sum(ref['by_position']['bar_position'].get(str(b), {}).get('by_type', {}).get(kind, {}).get('count', 0)
                            for b in range(1, 9)) for kind in KINDS}
    lines = [f"# Ornements idiomatiques (V3b). Taux : plancher écrit du corpus, références/ornaments/{ref_name}.json",
             "# du labo (DS-T10), pour 100 notes de la mélodie ; `rate_scale` les relève (les ABC sous-notent).",
             '[ornaments]', f'rate_scale = {rate_scale}', '', '[ornaments.types.passing]', 'min_carrier_eighths = 1.5',
             'density_curve = [[0.0, 0.0], [0.5, 0.0], [1.0, 1.0]]']
    for kind in KINDS:
        spec = types[kind]
        if not spec['enabled'] or spec['occurrences'] == 0:
            continue
        rate = spec['written_per_100_notes']
        lines += ['', f'[ornaments.types.{kind}]', f'written_per_100_notes = {fmt(rate)}']
        shape = [spec['per_100_notes_by_bar'][str(i + 1)] for i in range(8)]
        if positioned[kind] >= 30 and sum(shape) > 0:
            level = rate / (sum(shape) / 8)
            lines.append('per_100_notes_by_bar = [' + ', '.join(fmt(s * level) for s in shape) + ']')
        lines.append(f'min_carrier_eighths = {number(min_carrier(ref, kind))}')
        if kind == 'grace_single':
            lines.append('span = 0.4')
        if kind == 'grace_multiple':
            lines += ['span = 0.49', 'count = 2']
        if kind in ('double_stop', 'chord'):
            lines.append('strings = [55, 62, 69, 76]')
            lines.append('min_interval = 3')
            lines.append('max_interval = ' + ('24' if kind == 'chord' else '12'))
        lines.append(f'velocity_scale = {VELOCITY[kind]}')
    return '\n'.join(lines) + '\n'


# ------------------------------------------------------------------ interprétation écrite (V2)

def performance_groups(tradition, kind, meter, lab=LAB):
    """Conditionnelles conjointes position × durée × contexte, par le parseur commun du labo."""
    sys.path.insert(0, str(lab / 'src'))
    from harmonium_lab.trad.ornaments import extract_ornaments
    root = lab / 'corpus' / CORPORA[tradition]
    factor = Fraction(meter['steps_per_bar'] * meter['denominator'], 8 * meter['numerator'])
    groups = {}
    entries = [json.loads(line) for line in (root / 'manifest.jsonl').read_text().splitlines() if line.strip()]

    def group(row):
        step = Fraction(row['onset_eighths']) * factor
        duration = Fraction(row['duration_eighths']) * factor
        if step.denominator != 1 or duration.denominator != 1 or not (0 <= step < meter['steps_per_bar']) \
                or not (1 <= duration <= meter['steps_per_bar']):
            return None
        previous = row['previous_interval']
        context = 'any' if step == 0 else ('repeated' if previous == 0 else 'ascending' if previous is not None and previous > 0
                                            else 'descending' if previous is not None else None)
        if context is None:
            return None
        return f'{step}|{"short" if duration <= 2 else "long"}|{context}'

    for entry in entries:
        if entry['type'] != kind:
            continue
        data = extract_ornaments((root / 'tunes' / kind / (entry['id'] + '.abc')).read_text())
        if data['inline_changes']:
            continue
        for row in data['notes']:
            key = group(row)
            if key is not None:
                groups.setdefault(key, {'notes': 0, 'Linked': 0, 'Detached': 0, 'Accent': 0})['notes'] += 1
        for event in data['articulations']:
            key = group(event)
            mark = {'slur': 'Linked', 'tie': 'Linked', 'tenuto': 'Linked', 'staccato': 'Detached',
                    'accent': 'Accent', 'marcato': 'Accent'}.get(event['type'])
            if key is not None and mark is not None:
                groups.setdefault(key, {'notes': 0, 'Linked': 0, 'Detached': 0, 'Accent': 0})[mark] += 1
    return dict(sorted(groups.items()))


def performance_rules(groups, steps_per_bar):
    lines = []
    for key, counts in groups.items():
        step, length, context = key.split('|')
        if counts['notes'] < 30:
            continue
        marks = counts['Linked'] + counts['Detached'] + counts['Accent']
        for articulation in ('Linked', 'Detached', 'Accent'):
            count = counts[articulation]
            if count == 0:
                continue
            weight = 100 * count / (counts['notes'] - marks)
            low, high = (1, 2) if length == 'short' else (3, steps_per_bar)
            lines += [f'# DS-T10b : {count} marques {articulation} / {counts["notes"]} porteuses ; {key}.',
                      '[[performance.tracks.Melody.rules]]', f'articulation = "{articulation}"',
                      f'weight = {weight:.6f}', f'steps = [{step}]', f'min_duration = {low}', f'max_duration = {high}',
                      f'context = "{context}"']
            if articulation == 'Detached':
                lines.append('shorten = 0.20')
    return '\n'.join(lines) + '\n'


# ------------------------------------------------------------------ comparaison avec les profils existants

def selftest():
    import copy
    checks = [('gigue_quebecoise', 'quebecoise-jig', 'quebecoise', 'jig'),
              ('reel_acadien', 'acadienne-reel', 'acadienne', 'reel'),
              ('hornpipe_britannique_coloniale', 'britannique_coloniale-hornpipe', 'britannique_coloniale', 'hornpipe'),
              ('gigue_britannique_coloniale', 'britannique_coloniale-jig', 'britannique_coloniale', 'jig'),
              ('valse_quebecoise', 'quebecoise-waltz', 'quebecoise', 'waltz')]
    endings = json.loads((Path(tempfile.gettempdir()) / 'r1' / 'endings.json').read_text()) if False else run_endings()
    failures = 0
    for name, ref, tradition, kind in checks:
        existing = tomllib.loads((PROFILES / name / 'profile.toml').read_text())
        meter = existing['meter']
        real = structure(ref)
        built = tomllib.loads(form_toml(real, endings, ref, existing['melody']['cell_units'], existing['form']['parts'][1]['melody_center_offset']))
        built_parts, old_parts = built['form']['parts'], existing['form']['parts']
        for a, b in zip(built_parts, old_parts):
            for key in ('roles', 'second_ending', 'rhythm', 'phrase_ends', 'melody_center_offset'):
                if a.get(key) != b.get(key):
                    failures += 1
                    print(f'{name} form.{b["name"]}.{key} différent')
        orn = tomllib.loads(ornaments_toml(ref))['ornaments']
        if orn != existing['ornaments']:
            failures += 1
            for key in set(orn['types']) | set(existing['ornaments']['types']):
                if orn['types'].get(key) != existing['ornaments']['types'].get(key):
                    print(f'{name} ornaments.{key}: {orn["types"].get(key)} / {existing["ornaments"]["types"].get(key)}')
        meter_dict = {'steps_per_bar': meter['steps_per_bar'], 'numerator': meter['time_signature']['numerator'],
                      'denominator': meter['time_signature']['denominator']}
        groups = performance_groups(tradition, kind, meter_dict | {})
        rules = tomllib.loads('[performance.tracks.Melody]\n' + performance_rules(groups, meter['steps_per_bar']))
        built_rules = rules['performance']['tracks']['Melody'].get('rules', [])
        old_rules = [r for r in existing['performance']['tracks']['Melody']['rules'] if 'context' in r]
        worst = 0.0
        for a, b in zip(built_rules, old_rules):
            if {k: v for k, v in a.items() if k != 'weight'} != {k: v for k, v in b.items() if k != 'weight'}:
                failures += 1
                print(f'{name} règle différente\n  {a}\n  {b}')
            gap = abs(a['weight'] / b['weight'] - 1)
            if gap > worst:
                worst, where = gap, (a, b)
        if len(built_rules) != len(old_rules):
            failures += 1
            print(f'{name} règles : {len(built_rules)} contre {len(old_rules)}')
        print(f'{name} : {len(built_rules)} règles, plus grand écart relatif de poids {worst:.3%}')
        if worst > 0.05:
            print('   ', where)
        print(f'{name} : formes, ornements, règles contrôlés')
    sys.exit(1 if failures else 0)


def run_endings():
    out = Path(tempfile.gettempdir()) / 'r1-endings.json'
    subprocess.run([sys.executable, str(ENGINE / 'specs/011-phase3-phrase/measure_endings.py'),
                    '--lab-root', str(LAB), '--output', str(out)], check=True)
    return json.loads(out.read_text())


# ------------------------------------------------------------------ styles à construire

def blocks(text):
    """Blocs (en-tête, texte) d'un TOML ; le premier a l'en-tête None."""
    result, header, lines = [], None, []
    for line in text.splitlines(keepends=True):
        match = re.match(r'^\[\[?([^\]\n]+)\]\]?\s*(?:#.*)?$', line)
        if match:
            result.append((header, ''.join(lines)))
            header, lines = match.group(1).strip(), []
        lines.append(line)
    result.append((header, ''.join(lines)))
    return result


def pick(text, *headers):
    """Concatène les blocs dont l'en-tête est exactement l'un des `headers` ou commence par `h.`."""
    out = []
    for header, body in blocks(text):
        if header is not None and any(header == h or header.startswith(h + '.') for h in headers):
            out.append(body)
    return ''.join(out)


def drop(text, *headers):
    out = []
    for header, body in blocks(text):
        if header is None or not any(header == h or header.startswith(h + '.') for h in headers):
            out.append(body)
    return ''.join(out)


def key_modes(corpus_dir, kind):
    """Cinq tonalités/modes les plus fréquents ; poids entiers de somme 20, minimum 1 (plus grand reste)."""
    sys.path.insert(0, str(LAB / 'src'))
    from harmonium_lab.trad.form import analyze_form
    counts = Counter()
    for path in sorted((LAB / 'corpus' / corpus_dir / 'tunes' / kind).glob('*.abc')):
        try:
            plan = analyze_form(path.read_text())
        except Exception:
            continue
        counts[(plan.key_tonic_pc, plan.mode)] += 1
    top = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))[:5]
    total = sum(counts.values())
    raw = [(n * 20 / total) for _, n in top]
    weights = [max(1, int(r)) for r in raw]
    order = sorted(range(len(raw)), key=lambda i: -(raw[i] - int(raw[i])))
    while sum(weights) < 20:
        weights[order[0]] += 1
        order = order[1:] + order[:1]
    while sum(weights) > 20:
        weights[weights.index(max(weights))] -= 1
    return [(k, m, w, n) for ((k, m), n), w in zip(top, weights)], total


def key_modes_toml(entries):
    lines = ['key_modes = [']
    for key, mode, weight, _ in entries:
        lines.append(f'  {{ key = {key}, mode = "{mode}", weight = {weight} }},')
    lines.append(']')
    return '\n'.join(lines)


def color_toml(entries):
    """Palette `[[color]]` de la valence : majeurs/mixolydien plus probables aux valences claires, dorien/éolien aux sombres."""
    out = []
    for key, mode, weight, _ in entries:
        if mode in ('ionian', 'mixolydian'):
            curve = f'[[0.0, 0.1], [0.5, {weight * 0.5:.1f}], [1.0, {weight * 0.75:.1f}]]'
        else:
            curve = f'[[0.0, {weight * 0.75:.1f}], [0.5, {weight * 0.125:.1f}], [1.0, 0.0]]'
        out.append(f'[[color]]\nkey = {key}\nmode = "{mode}"\nweight = {curve}\n')
    return '\n'.join(out)


def mine(kind, corpus_dir):
    """cells.toml : notes de `mine-cells`, poids = effectifs du fichier compagnon."""
    with tempfile.TemporaryDirectory() as tmp:
        target = Path(tmp) / 'cells.toml'
        subprocess.run([str(LAB / '.venv/bin/harmonium-lab'), 'mine-cells', '--type', kind, '--corpus',
                        str(LAB / 'corpus' / corpus_dir), '-o', str(target)], check=True, capture_output=True, cwd=LAB)
        mined = target.read_text()
        counts = {(c['role'], c['notes']): c['count'] for c in json.loads((Path(tmp) / 'cells.weights.json').read_text())}
    airs = re.search(r'\((\d+) airs\)', mined).group(1)
    cells = tomllib.loads(mined)['cells']
    return cells, counts, int(airs)


def cells_toml(title, tune_label, corpus_version, cells, counts, airs, units):
    by_role = Counter(c['role'] for c in cells)
    lines = [f'# Banque de cellules mélodiques d\'une mesure ({units} croches) pour {title}.', '#',
             f'# Source : cellules minées du corpus harmonium_lab ({airs} airs de {tune_label}), `harmonium-lab mine-cells`,',
             f'# corpus {corpus_version}, top 30 par rôle : {len(cells)} cellules.',
             '# `weight` = fréquence de la cellule au corpus ; le tirage d\'un rôle est pondéré par ce poids.', 'cells = [']
    for cell in cells:
        lines.append(f'  {{ role = "{cell["role"]}", notes = "{cell["notes"]}", weight = {counts[(cell["role"], cell["notes"])]} }},')
    lines.append(']')
    return '\n'.join(lines) + '\n', by_role


def corpus_version(corpus_dir):
    return (LAB / 'corpus' / corpus_dir / 'VERSION').read_text().splitlines()[0]


def set_toml(template_set, comment, transitions, ramp_bpm, ramp_lift, ceiling, key_source):
    key = tomllib.loads((PROFILES / template_set / 'set.toml').read_text())['key']
    lines = ['# Forme longue : un set est une suite d\'airs de ce profil (même mètre, même forme).',
             '# Schéma et effet à l\'oreille : profiles/README.md#forme-longue-set.', '',
             '# Nombre d\'airs d\'un set : distribution pondérée (The Session, 105 959 sets de 2 airs ou plus :',
             '# 2 airs 45,2 %, 3 : 44,3 %, 4 : 6,8 %, 5 et plus : 3,8 %, repliés sur 5).',
             'length = [', '  { tunes = 2, weight = 45 },', '  { tunes = 3, weight = 44 },', '  { tunes = 4, weight = 7 },',
             '  { tunes = 5, weight = 4 },', ']', '', '# Facteurs sur le poids de base de chaque tonalité candidate selon sa classe de transition',
             f'# depuis l\'air précédent. {key_source}', '[key]']
    notes = {'same': 'même tonique, même mode', 'parallel': 'même tonique, autre mode',
             'relative': 'même collection diatonique, autre tonique', 'fifth_up': 'tonique +7 demi-tons',
             'fourth_up': 'tonique +5', 'step_up': 'tonique +2', 'step_down': 'tonique −2', 'other': 'autre',
             'last_dark': 'neutre : effet de position non mesuré pour ce type'}
    for name in ('same', 'parallel', 'relative', 'fifth_up', 'fourth_up', 'step_up', 'step_down', 'other', 'last_dark'):
        lines.append(f'{name:<9} = {key[name]:.2f}   # {notes[name]}')
    lines += ['', comment]
    for kind, weight, bars in transitions:
        lines += ['[[transitions]]', f'kind = "{kind}"', f'weight = {weight}'] + ([f'bars = {bars}'] if bars else [])
    lines += ['', '[end]', 'outro = true', 'gap_bars = [{ bars = 0, weight = 1 }]', '', '[ramp]',
              f'tempo_bpm = [[0.0, 0.0], [1.0, {ramp_bpm}]]', f'intensity_lift = [[0.0, 0.0], [1.0, {ramp_lift}]]',
              'gate = [[0.0, 0.0], [0.2, 1.0], [0.8, 1.0], [1.0, 0.0]]', f'ceiling_bpm = {ceiling}', '',
              '[signature]', 'return_rate = 0.17', 'memory = 3']
    return '\n'.join(lines) + '\n'


def retitle(text, title):
    lines = text.splitlines(keepends=True)
    lines[0] = f'# {title}\n'
    return ''.join(lines)


def set_master(mix_text, master):
    return re.sub(r'(?m)^master_db = .*$', f'master_db = {master}', mix_text, count=1)


def set_level(mix_text, track, level):
    pattern = re.compile(rf'(\[mix\.tracks\.{track}\]\nlevel_db = )[-0-9.]+')
    return pattern.sub(lambda m: m.group(1) + str(level), mix_text, count=1)


PIANO_JIG = '''[accompaniment]
strong_steps = [0, 6]
strong_weight = 2
weak_weight = 1
tie_priority = ["I", "V", "IV"]

[accompaniment.bass]
low = 36
high = 47
walk_low = 34
walk_high = 50
hits = [
  { step = 0, degree = "root", duration = 3, velocity = 82, doubling = ["octave"] },
  { step = 6, degree = "fifth_if_held", duration = 3, velocity = 82, doubling = ["octave"] },
]

[accompaniment.bass.passing]
none_weight = 3
density = [[0.0, 0.0], [0.5, 0.8], [1.0, 1.2]]
phrase = [[0.0, 0.7], [0.75, 1.0], [1.0, 1.3]]
avoid_melody_intervals = [1, 6, 11]
rules = [
  { when = "next_target_differs", before = [4], motion = "scale_step", weight = 3, duration = 2, velocity_delta = -12, min_leap = 0 },
  { when = "next_target_differs", before = [4, 2], motion = "scale_walk", weight = 4, duration = 1, velocity_delta = -12, min_leap = 5 },
  { when = "next_target_differs", before = [2], motion = "chromatic", weight = 1, duration = 2, velocity_delta = -12, min_leap = 0 },
]

# Recette « piano main gauche » de profiles/README.md, grille de gigue : basse sur 1 et 4 (pas 0 et 6),
# accords sur 2-3 et 5-6 (pas 2, 4, 8, 10) ; chaque pulsation s'entend (pratique cap-bretonnaise, étude V5a §2).
[accompaniment.chords]
voicing = "close_triad"
center = 58
max_drift = 5
strings = 3
default_pattern = 1
patterns = [
  { hits = [{ step = 4, stroke = "block", duration = 2, velocity = 60 }, { step = 10, stroke = "block", duration = 2, velocity = 56 }] },
  { hits = [{ step = 2, stroke = "block", duration = 2, velocity = 56 }, { step = 4, stroke = "block", duration = 2, velocity = 62 }, { step = 8, stroke = "block", duration = 2, velocity = 56 }, { step = 10, stroke = "block", duration = 2, velocity = 62 }] },
]

[instruments]
melody_program = 40
countermelody_program = 40
chords_program = 0
bass_program = 0
percussion_bank = 128
percussion_program = 0

'''

REEL_BRIT_ACCOMP = '''[accompaniment]
strong_steps = [0, 8]
strong_weight = 2
weak_weight = 1
tie_priority = ["I", "V", "IV", "II"]

[accompaniment.bass]
low = 40
high = 50
walk_low = 38
walk_high = 52
hits = [
  { step = 0, degree = "root", duration = 6, velocity = 72 },
  { step = 8, degree = "fifth_if_held", duration = 6, velocity = 72 },
]

[accompaniment.bass.passing]
none_weight = 4
density = [[0.0, 0.0], [0.5, 0.6], [1.0, 1.0]]
phrase = [[0.0, 0.7], [0.75, 1.0], [1.0, 1.3]]
avoid_melody_intervals = [1, 6, 11]
arousal_low_multiplier = 0.5
arousal_high_multiplier = 1.5
rules = [
  { when = "next_target_differs", before = [4], motion = "scale_step", weight = 3, duration = 2, velocity_delta = -12, min_leap = 0 },
  { when = "next_target_differs", before = [4, 2], motion = "scale_walk", weight = 2, duration = 1, velocity_delta = -12, min_leap = 5 },
  { when = "next_target_differs", before = [2], motion = "chromatic", weight = 1, duration = 2, velocity_delta = -12, min_leap = 0 },
]

# Accords tenus sur les deux pulsations de la blanche (pas de contretemps) : convention, comme le hornpipe et la gigue
# britanniques ; aucune source ne décrit l'accompagnement des reels de bal coloniaux (étude V5a §2.1).
[accompaniment.chords]
center = 60
max_drift = 6
strings = 4
default_pattern = 1
patterns = [
  { hits = [{ step = 0, stroke = "down", duration = 8, velocity = 60 }] },
  { hits = [{ step = 0, stroke = "down", duration = 6, velocity = 62 }, { step = 8, stroke = "down", duration = 6, velocity = 60 }] },
  { hits = [{ step = 0, stroke = "down", duration = 3, velocity = 62 }, { step = 4, stroke = "down", duration = 3, velocity = 54 }, { step = 8, stroke = "down", duration = 3, velocity = 60 }, { step = 12, stroke = "down", duration = 3, velocity = 54 }] },
]

[accompaniment.chords.strum]
down_gap_ms = 8.0
up_gap_ms = 6.0
up_strings = 3
gap_jitter = 0.3
anchor = 0.5
tempo_exponent = 0.0
ref_bpm = 208.0
max_spread_ratio = 0.4
velocity_slope = 4
velocity_jitter = 5

[instruments]
melody_program = 40
chords_program = 24
bass_program = 32
percussion_bank = 128
percussion_program = 0

'''

HARMONY_BRIT = '''[harmony]
model = "trad_functional"
steps_per_chord = {steps}
[[harmony.candidates]]
mode = "ionian"
chords = [{{degree="I",quality="Major"}}, {{degree="IV",quality="Major"}}, {{degree="V",quality="Major"}}, {{degree="II",quality="Minor"}}, {{degree="VI",quality="Minor"}}]
[[harmony.candidates]]
mode = "mixolydian"
chords = [{{degree="I",quality="Major"}}, {{degree="IV",quality="Major"}}, {{degree="V",quality="Minor"}}, {{degree="BVII",quality="Major"}}]
[[harmony.candidates]]
mode = "dorian"
chords = [{{degree="I",quality="Minor"}}, {{degree="IV",quality="Major"}}, {{degree="BVII",quality="Major"}}, {{degree="V",quality="Minor"}}]
[[harmony.candidates]]
mode = "aeolian"
chords = [{{degree="I",quality="Minor"}}, {{degree="IV",quality="Minor"}}, {{degree="V",quality="Minor"}}, {{degree="BVII",quality="Major"}}, {{degree="VI",quality="Major"}}]

'''


STYLES = {
    'gigue_acadienne': dict(
        ref='acadienne-jig', tradition='acadienne', corpus='acadienne', kind='jig', cell_units=6, b_offset=2,
        template='gigue_quebecoise', title='la gigue acadienne', de='de la gigue acadienne', label='gigues acadiennes',
        name='Gigue acadienne',
        tradition_text="Acadie / Maritimes (Nouveau-Brunswick, Île-du-Prince-Édouard, Cap-Breton / Chéticamp, Îles-de-la-Madeleine), répertoire traditionnel de violon, de piano et de podorythmie",
        tags=['gigue', 'jig', 'acadien', 'acadie', 'maritimes', 'violon', 'piano', 'podorythmie'],
        meter=dict(num=6, den=8, steps=12, pulse=6, pulses=2, swing=0.5, bpm=180.0, lo=168.0, hi=192.0),
        master=15.0, set_template='gigue_quebecoise', ramp=(6.0, 0.08),
        transitions=[('direct', 70, None), ('call', 20, 1), ('held', 10, None)]),
    'reel_britannique_coloniale': dict(
        ref='britannique_coloniale-reel', tradition='britannique_coloniale', corpus='britannique_coloniale', kind='reel',
        cell_units=8, b_offset=None, template='reel_acadien', title='le reel britannique colonial', de='du reel britannique colonial', label='reels britanniques coloniaux',
        name='Reel britannique colonial',
        tradition_text="Musique britannique coloniale, bals de garnison et danses de campagne d'Amérique du Nord britannique, c. 1760–1840",
        tags=['reel', 'britannique', 'coloniale', 'violon', '1837', 'garnison', 'danse', 'country dance'],
        meter=dict(num=2, den=2, steps=16, pulse=8, pulses=2, swing=0.5, bpm=208.0, lo=192.0, hi=224.0),
        master=13.5, set_template='reel_quebecois', ramp=(5.0, 0.06),
        transitions=[('direct', 90, None), ('held', 10, None)]),
    'valse_britannique_coloniale': dict(
        ref='britannique_coloniale-waltz', tradition='britannique_coloniale', corpus='britannique_coloniale', kind='waltz',
        cell_units=6, b_offset=None, template='valse_quebecoise', title='la valse britannique coloniale', de='de la valse britannique coloniale', label='valses britanniques coloniales',
        name='Valse britannique coloniale',
        tradition_text="Musique britannique coloniale, bals de garnison et salons d'Amérique du Nord britannique, c. 1760–1840",
        tags=['valse', 'waltz', 'britannique', 'coloniale', 'violon', '1837', 'garnison', 'bal'],
        meter=dict(num=3, den=4, steps=12, pulse=4, pulses=3, swing=0.5, bpm=138.0, lo=126.0, hi=150.0),
        master=12.0, set_template='valse_quebecoise', ramp=(2.0, 0.05),
        transitions=[('direct', 100, None)]),
}


def corpus_stats(style):
    """Effectifs réels du corpus pour le type : airs, sources, notes écrites."""
    corpus_dir = style['corpus']
    sources = Counter()
    for line in (LAB / 'corpus' / corpus_dir / 'manifest.jsonl').read_text().splitlines():
        row = json.loads(line)
        if row['type'] == style['kind']:
            sources[row['source']] += 1
    ref = ornament_ref(style['ref'])
    return {'airs': sum(sources.values()), 'sources': dict(sources), 'notes': ref['notes'],
            'tunes_measured': ref['tunes'], 'bar_equivalents': ref['bar_equivalents']}


def build_style(name, endings, only_text=False):
    style = STYLES[name]
    template = PROFILES / style['template']
    base = (template / 'profile.toml').read_text()
    real = structure(style['ref'])
    meter = style['meter']
    version = corpus_version(style['corpus'])
    stats = corpus_stats(style)
    entries, total = key_modes(style['corpus'], style['kind'])
    b_offset = style['b_offset']
    if b_offset is None:
        profile = json.loads((LAB / f'references/trad-{style["ref"]}/profile.json').read_text())
        b_offset = int(round(profile['scalars']['register__b_minus_a_mean_pitch']['mean']))
    head = f'''[files]
cells = "cells.toml"
feet = "feet.toml"
arrangement = "arrangement.toml"
dynamics = "dynamics.toml"
set = "set.toml"

[meta]
id = "{name}"
name = "{style['name']}"
tradition = "{style['tradition_text']}"
tags = [{', '.join('"' + t + '"' for t in style['tags'])}]
corpus = "harmonium_lab: corpus {style['corpus']} ({version}, {stats['airs']} airs de type {style['kind']} : {', '.join(f'{v} {k}' for k, v in stats['sources'].items())}); provenance dans corpus/sources.md et le manifeste du labo"

[meter]
time_signature = {{ numerator = {meter['num']}, denominator = {meter['den']} }}
steps_per_bar = {meter['steps']}
steps_per_unit = 2
steps_per_pulse = {meter['pulse']}
pulses_per_bar = {meter['pulses']}
default_swing_ratio = {meter['swing']}
default_bpm = {meter['bpm']}
min_bpm = {meter['lo']}
max_bpm = {meter['hi']}

'''
    form = form_toml(real, endings, style['ref'], style['cell_units'], b_offset)
    melody = pick(base, 'melody.params')
    melody_head = f'[melody]\ncell_units = {style["cell_units"]}\n' + {
        'gigue_acadienne': 'strong_units = [0, 3]\nweak_units = [1, 2, 4, 5]\n',
        'reel_britannique_coloniale': 'strong_units = [0, 4]\nweak_units = [1, 3, 5, 7]\n',
        'valse_britannique_coloniale': 'strong_units = [0, 2, 4]\nweak_units = [1, 3, 5]\n'}[name] + key_modes_toml(entries) + '\n\n'
    if name == 'gigue_acadienne':
        harmony = pick(base, 'harmony') + '\n'
        accomp = PIANO_JIG
        mix = set_master(drop(pick(base, 'mix'), 'mix.tracks.Countermelody'), style['master'])
        hetero = pick((PROFILES / 'reel_acadien' / 'profile.toml').read_text(), 'mix.tracks.Countermelody',
                      'performance.tracks.Countermelody', 'heterophony')
    elif name == 'reel_britannique_coloniale':
        harmony = HARMONY_BRIT.format(steps=16)
        accomp = REEL_BRIT_ACCOMP
        hornpipe = (PROFILES / 'hornpipe_britannique_coloniale' / 'profile.toml').read_text()
        mix = set_master(pick(hornpipe, 'mix'), style['master'])
        hetero = ''
    else:
        harmony = HARMONY_BRIT.format(steps=12)
        valse = base
        accomp = pick(valse, 'accompaniment') + pick(valse, 'instruments').replace('countermelody_program = 21\n', '') + '\n'
        accomp = re.sub(r'ref_bpm = [0-9.]+', f'ref_bpm = {meter["bpm"]}', accomp)
        hornpipe = (PROFILES / 'hornpipe_britannique_coloniale' / 'profile.toml').read_text()
        mix = set_master(pick(hornpipe, 'mix'), style['master'])
        hetero = ''
    perf_template = pick(base, 'performance.tracks.Melody')
    perf_head = perf_template.split('# DS-T10b')[0]
    emotions = ''.join(pick(base, 'performance.tracks.Melody.emotions').splitlines(keepends=True)[:6])
    groups = performance_groups(style['tradition'], style['kind'], {
        'steps_per_bar': meter['steps'], 'numerator': meter['num'], 'denominator': meter['den']})
    rules = performance_rules(groups, meter['steps'])
    perf = perf_head + rules + emotions.rstrip('\n') + '\n\n'
    orn = ornaments_toml(style['ref'])
    profile_text = (head + form + '\n' + melody_head + melody.rstrip('\n') + '\n\n' + harmony.rstrip('\n') + '\n\n' + accomp
                    + mix.rstrip('\n') + '\n\n' + perf + orn + ('\n' + hetero.rstrip('\n') + '\n' if hetero else ''))
    cells, counts, airs = mine(style['kind'], style['corpus'])
    cells_text, by_role = cells_toml(style['title'], style['label'], version, cells, counts, airs, style['cell_units'])
    return {'profile': profile_text, 'cells': cells_text, 'stats': stats, 'key_modes': entries, 'tunes_total': total,
            'cells_airs': airs, 'cells_by_role': dict(by_role), 'b_offset': b_offset, 'real': real, 'groups': groups}


NARRATIVE = {
    'gigue_acadienne': dict(
        title='Gigue acadienne (violon, piano de veillée et podorythmie)',
        why="""Le corpus acadien compte 55 gigues (The Session, ODbL, airs enregistrés par des artistes acadiens ; même source que le reel acadien,
dont l'échantillon est dominé par le Cap-Breton). Elle se distingue de la gigue québécoise (88 airs) par des **données** — taux
d'ornements écrits de 2 à 8 fois plus élevés (appogiatures 0,39 contre 0,21 pour 100 notes, multiples 0,26 contre 0,03, rouleaux 0,41
contre 0,21, doubles cordes 0,39 contre 0,08), la majeur bien plus présent (11 airs sur 55, 20 %, contre 7 sur 88 au Québec), paires de
mesures « a b c d » à 32,5 % contre 24,5 % — et par **l'accompagnement**, qui est le point audible : le piano de
veillée en recette « main gauche » (basse sur 1 et 4, accords sur 2-3 et 5-6) remplace la guitare. Le piano est la pratique
documentée du Cap-Breton voisin (étude V5a §2.1 : MacDonald, Fraser, MacLean) ; rien de comparable n'est mesuré pour l'Acadie
elle-même : extrapolation **[V]**, à valider à l'oreille.""",
        meter="""6/8 en 12 pas, pulse à la noire pointée (2 pulses de 6 pas), croches égales (swing 0,5). Tempo **proposé** : 180 à la noire,
soit 120 par noire pointée, plage 168–192 — **aucune mesure** : le corpus ne porte aucun `Q:` exploitable (`mine-tempo` : tempos manquants).
Comme la gigue québécoise (172,5), la valeur reste dans l'ordre de grandeur cap-breton ; elle sera réglée à l'écoute.""",
        accomp="""**Piano** (programmes 0/0) selon la recette « piano main gauche » de [profiles/README.md](../README.md#recettes-piano-et-accordéon),
adaptée à la grille de gigue : basse (doublée à l'octave) sur les pas 0 et 6, accords en blocs sur 2/4/8/10 (motif par défaut) ou 4/10
(motif clairsemé à faible densité), marches de basse et conduite de voix de la recette ; **aucun grattage**, le piano n'en a pas.
Pieds : même table que la gigue québécoise (talon sur 1 et 4, pointes sur 2-3 et 5-6, pratique décrite pour le Cap-Breton, Fraser 2012).
L'ABC ne fournit presque aucun accord (56 mesures chiffrées sur 55 airs) : harmonie I/IV/V/vi par mode comme la gigue québécoise.
Mesure : mélodie − piano 7,4 dB, mélodie − basse 10,3 dB (critère ≥ 4 dB) ; la basse du piano GM penche à gauche
(G−D +3,8 dB avec `pan = 0`) : c'est le placement des touches de la banque, laissé tel quel.""",
        hetero="""Second violon, entrée au tour 2, unisson dominant : **mêmes poids que le reel acadien** (76 / 10 / 8 / 6 / 2 / 2 / 1 / 2). Proposition
d'arrangement à écouter, pas un taux d'hétérophonie mesuré dans le corpus monodique ABC.""",
        mix="""Salle communautaire courte comme les gigues et reels à pieds (taille 0,38, amorti 0,58, niveau 0,28). Piano et basse à −5 / −8 dB,
pieds −6 dB, bus **+15 dB** : le piano GM est ~1,5 dB moins énergique que la guitare nylon à niveau égal (RMS moyen de cinq graines en
neutre −21,1 dBFS à 13,5 dB de bus, −19,6 à 15 dB), le bus est relevé pour retrouver le niveau des autres gigues ; pic le pire des cinq
graines aux émotions maximales −2,3 dBFS.""",
        limits="""55 airs d'une seule source (The Session) : positions de mesure, taux par mesure et secondes fins reposent sur de petits effectifs
(B : 36 secondes fins, A : 48). Aucun ornement `turn`, `chord` ni trille mesuré dans ce type : désactivés. Les tailles de l'échantillon
interdisent de conclure sur la tradition acadienne dans son ensemble.""",
        sources_note="The Session, ODbL ; airs enregistrés par des artistes acadiens"),
    'reel_britannique_coloniale': dict(
        title='Reel britannique colonial (violon, guitare tenue, sans pieds)',
        why="""Le corpus britannique colonial compte 493 reels (479 Nottingham Music Database, 8 Cahusac 1809, 6 AmRev de Chambers) : c'est le plus
gros type non québécois du corpus. Il se distingue du reel québécois (574) et du reel acadien (169) **à l'oreille** par l'absence de podorythmie
et de chops (accords tenus sur les deux pulsations de la blanche), un tempo plus posé de danse de bal, des ornements écrits presque absents
(aucun rouleau ; 5 appogiatures simples et 5 triolets répétés sur 51 266 notes) et une harmonie à un accord par mesure (7 973 mesures sur 11 992 chiffrées).
Les tonalités sont ré (193 airs) et sol (168) ionien, avec peu de modal. Le libellé « colonial » est celui du lot du labo : la source
dominante est un recueil anglais (NMD), pas des relevés nord-américains ; même réserve pour la gigue et le hornpipe britanniques.""",
        meter="""2/2 en 16 pas, pulse à la blanche (2 pulses de 8 pas) comme les autres reels, **croches égales** (swing 0,5 : le 4/4 du corpus ne
documente aucun swing). Tempo **proposé** 208 à la noire (104 à la blanche), plage 192–224 : danse de bal plus lente que les reels
de veillée (230) — aucune mesure (corpus sans `Q:`), à régler à l'oreille.""",
        accomp="""Accords tenus (durée de 6 pas) sur les pas 0 et 8, motif à quatre accords à haute densité ; basse sur 0 et 8 (durée 6) ; notes de passage
à poids de reel (`none_weight` 4). Guitare nylon et basse acoustique comme les profils britanniques ; **aucune** podorythmie
(table de pieds vide, entrées d'arrangement conservées pour garder la même forme de fichier). Accompagnement par convention
(étude V5a §2.1 : aucune source sur les reels de bal coloniaux). Un accord par mesure (`steps_per_chord = 16`) : le rythme harmonique
dominant du corpus (7 973 mesures à un accord, 3 879 à deux).""",
        hetero="""**Aucune seconde voix** : pas de source d'ensemble pour ce répertoire, comme pour la gigue et le hornpipe britanniques.""",
        mix="""Salle de danse courte des profils britanniques (taille 0,40, amorti 0,55, niveau 0,28), guitare à droite, violon légèrement à gauche,
bus 13,5 dB (cinq graines : RMS moyen −19,7 dBFS en neutre, pic le pire aux émotions maximales −2,9 dBFS).""",
        limits="""L'essentiel du lot vient d'un recueil (NMD, notation sans ornements ni articulations : 73,6 % des airs sans aucune marque). Les taux
d'ornements et d'articulations sont donc des planchers très bas, et rien dans l'ABC ne distingue un jeu colonial d'un jeu anglais.
Seules les deux premières parties sont modélisées : le profil joue AABB comme tous les autres, alors que 130 airs sur 437 sont notés
AABB, 115 AAB, 47 ABCD (parties toutes différentes) et 47 en croches continues sans parties.""",
        sources_note="Nottingham Music Database 479, Cahusac 8, AmRev (Chambers) 6"),
    'valse_britannique_coloniale': dict(
        title='Valse britannique coloniale (violon, guitare, sans pieds)',
        why="""60 valses (58 Nottingham Music Database, 2 Cahusac) ; la valse québécoise en compte 67 (42 Chambers, 25 The Session). Le profil se distingue
**dans les données** — forme plus souvent en quatre parties différentes (« a b c d » à 66,7 % des paires contre 45,5 % au Québec),
doubles cordes beaucoup plus fréquentes (2,54 pour 100 notes contre 1,28), ré et sol majeurs dominants, finales longues quasi systématiques
aux m.4/m.8 (76–82 %) — et **à l'oreille** par l'absence de pieds (la valse québécoise en a), un tempo plus posé et une guitare identique
mais sans podorythmie. Époque du lot : c. 1760–1840 selon le manifeste du labo.""",
        meter="""3/4 en 12 pas, pulse à la noire (3 pulses de 4 pas), swing 0,5. Tempo **proposé** 138 à la noire (plage 126–150), un peu en dessous
de la valse québécoise (145) : aucune mesure (corpus sans `Q:`).""",
        accomp="""Basse sur le premier temps, accords sur les temps 2 et 3 (« boum-tchic-tchic »), guitare et basse de la valse québécoise, marches de basse
limitées au dernier quart de phrase ; **pas de pieds** (table vide). Un accord par mesure (1 710 mesures sur 1 823 chiffrées).""",
        hetero="""**Aucune seconde voix** : pas de source d'ensemble pour ce répertoire ; l'accordéon de la valse québécoise est un anachronisme déclaré
pour 1837 et n'est pas repris.""",
        mix="""Salle de danse courte des profils britanniques (taille 0,40, amorti 0,55, niveau 0,28), bus 12 dB comme la valse québécoise
(cinq graines : RMS moyen −19,5 dBFS en neutre, pic le pire aux émotions maximales −4,5 dBFS).""",
        limits="""Seulement 60 airs, 58 du même recueil ; 52 positionnables, 27 paires de fins à huit mesures (B : 7 secondes fins). Les poids de
`second_ending` de B et les taux par position sont des petits effectifs. Aucune articulation écrite sauf quelques liaisons et ties
(13 liaisons, 87 ties sur 5 126 notes).""",
        sources_note="Nottingham Music Database 58, Cahusac 2"),
}


def readme(name, built):
    import math
    style = STYLES[name]
    n = NARRATIVE[name]
    real, stats = built['real'], built['stats']
    profile = tomllib.loads(built['profile'])
    ref_name = style['ref']
    orn = ornament_ref(ref_name)
    types = orn['engine_proposal']['types']
    articulations = orn['articulations']['by_type']
    rules = [r for r in profile['performance']['tracks']['Melody']['rules'] if 'context' in r]
    ends = {}
    endings = run_endings()
    for part in ('A', 'B'):
        counts = endings[ref_name]['counts'].get(f'eight_{part}:second_written', {})
        ends[part] = (counts.get('tonic', 0), counts.get('fifth', 0), counts.get('other', 0))
    lines = [f'# {n["title"]}', '']
    lines += ['## Tradition, corpus et choix du style (Constitution V et VII)', '',
              f'- **Tradition** : {style["tradition_text"]}.',
              f'- **Corpus de référence** : `harmonium_lab`, `corpus/{style["corpus"]}/tunes/{style["kind"]}/`, version {corpus_version(style["corpus"])}.',
              f'  **{stats["airs"]} airs** ({n["sources_note"]}) ; {stats["tunes_measured"]} airs mesurés, {stats["notes"]} notes écrites, '
              f'{real["positional_tunes"]} airs aux parties de huit mesures positionnables. Provenance et droits : `corpus/sources.md`.',
              '- Aucune musique autochtone.', '', '### Pourquoi ce style', '', n['why'], '',
              '### Métrique, swing et tempo', '', n['meter'], '']
    patterns = ' ; '.join(f'« {p["pattern"]} » {p["share"]:.1%} ({p["tune_count"]} airs)' for p in real['structures'][:4])
    lines += ['### Formes observées', '', f'Parties notées (A à F) : ' + ', '.join(f'{k} {v}' for k, v in real['parts'].items() if k != 'all') + f' ({real["parts"]["all"]} au total). Structures de paires de mesures (part des parties) : {patterns}.', '']
    lines += ['## Phrases probabilistes — DS-T9 / P3-T1', '',
              f'Source : `harmonium_lab/references/structure/{ref_name}.json`, champ `real` (formules de [profiles/README.md](../README.md#source-des-réglages-distribués-p3-t1-contrat-11) ;',
              'reproduites par `specs/013-moteur-dans-le-jeu/r1_profils.py selftest` sur quatre profils distribués).', '',
              '| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |', '|---|---:|---:|---:|---:|']
    for part in ('A', 'B'):
        copy5 = exact_rate(real['transformations'][f'{part}:m5/m1'], 'exact')
        copy6 = exact_rate(real['transformations'][f'{part}:m6/m2'], 'exact')
        cad = real['phrases']['rest_by_position'][f'{part}:m8']['tonic_cadence_rate']
        b7 = exact_rate(real['transformations']['B:m7/A:m7'], 'exact') if part == 'B' else 0.0
        lines.append(f'| {part} | {copy5:.2%} | {copy6:.2%} | {cad:.2%} | {b7:.2%} |')
    rhythm_rows = rhythm_patterns(real, style['cell_units'])
    lines += ['', 'Poids arrondis sur 10 000 ; les mesures se tirent indépendamment ; les taux conjoints m.5–6 et les structures de paires restent des',
              'mesures de sortie, pas des garanties du profil. `melody_center_offset` de B = ' + str(built['b_offset'])
              + ' demi-tons (moyenne `register.b_minus_a_mean_pitch` arrondie).', '',
              f'Rythmes de cadence m.4/m.8 : {len(rhythm_rows)} motifs de mesure pleine à durées contrastées, {sum(c for _, c in rhythm_rows)} mesures '
              '(`real.rhythm.cells`, effectifs sans arrondi, même distribution aux deux positions).', '',
              '## Secondes fins — P3-T2', '',
              'Effectifs de dernière note **écrite** des fins finales de parties de huit mesures (tonique / quinte / autre), commande '
              '[`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py) :', '',
              f'- A : {ends["A"][0]} / {ends["A"][1]} / {ends["A"][2]} ({sum(ends["A"])} occasions)',
              f'- B : {ends["B"][0]} / {ends["B"][1]} / {ends["B"][2]} ({sum(ends["B"])} occasions)', '',
              '## Cellules mélodiques', '',
              f'`cells.toml` : `harmonium-lab mine-cells --type {style["kind"]} --corpus corpus/{style["corpus"]}` ({built["cells_airs"]} airs, top 30 par rôle) :',
              ', '.join(f'{role} {count}' for role, count in built['cells_by_role'].items()) + f' cellules, poids = effectifs. '
              'Validées par le chargeur du moteur (`--check-cells`).', '']
    km = ', '.join(f'{["Do", "Do♯", "Ré", "Mi♭", "Mi", "Fa", "Fa♯", "Sol", "La♭", "La", "Si♭", "Si"][k]} {m} ({c} airs)' for k, m, _, c in built['key_modes'])
    lines += ['Tonalités et modes (`key_modes`, cinq plus fréquents du corpus, poids de somme 20 par plus grand reste) : ' + km
              + f' sur {built["tunes_total"]} airs analysés.', '',
              '## Harmonie et accompagnement', '', n['accomp'], '']
    lines += ['## Espace et mixage', '', n['mix'], '', 'Choix d\'espace proposés, pas des mesures historiques ; à valider à l\'oreille. Schéma : [Mixage](../README.md#mixage).', '']
    by = orn['articulations']['by_type']
    lines += ['## Interprétation V2', '',
              f'Source : `harmonium_lab/references/ornaments/{ref_name}.json` (DS-T10b). {stats["tunes_measured"]} airs, {stats["notes"]} attaques porteuses écrites.', '',
              '| Marque écrite | Occurrences | Par 100 notes |', '|---|---:|---:|']
    for label, keys in (('Liaison `(…)`', ['slur']), ('Tenue liée `-`', ['tie']), ('Détaché `.`', ['staccato']), ('Accent explicite', ['accent', 'marcato']), ('Tenuto `L`', ['tenuto'])):
        count = sum(by[k]['count'] for k in keys)
        lines.append(f'| {label} | {count} | {count * 100 / stats["notes"]:.6f} |')
    lines += ['', f'{len(rules)} règles de conditionnelles **conjointes** position × durée × contexte (cellules d\'au moins 30 porteuses), poids '
              '`100 × marques / (porteuses − toutes marques)` face à `Normal = 100`, comme les sept profils distribués ; chaque règle porte son effectif en commentaire. '
              'Micro-timing, jitter, courbe de phrase, respiration et émotions : **valeurs de proposition des profils voisins** (même mètre), à valider à l\'oreille ; '
              'le corpus ne les mesure pas. Schéma : [Interprétation](../README.md#interprétation).', '',
              '## Ornements V3b', '',
              f'Source : `harmonium_lab/references/ornaments/{ref_name}.json` (DS-T10, `engine_proposal`) ; `measure_ornaments.py corpus` contrôle que les taux du profil '
              'sont le recalcul du corpus. Taux écrits = **planchers** (les transcriptions sous-notent), `rate_scale = 4` pour tous les types, multiplicateurs d\'arousal neutres.', '',
              '| Type | Occurrences | /100 notes (plancher) | Taux par mesure | Durée porteuse min. (croches) |', '|---|---:|---:|---|---:|']
    for kind in KINDS:
        spec = types[kind]
        if kind in profile['ornaments']['types']:
            rule = profile['ornaments']['types'][kind]
            lines.append(f'| `{kind}` | {spec["occurrences"]} | {spec["written_per_100_notes"]:.3f} | {"oui" if "per_100_notes_by_bar" in rule else "non"} | {rule["min_carrier_eighths"]:g} |')
        else:
            lines.append(f'| `{kind}` | {spec["occurrences"]} | {spec["written_per_100_notes"]:.3f} | — | aucune occurrence : désactivé |')
    lines += ['', '`passing` : note de passage de densité émotionnelle, courbe `[[0,0],[0.5,0],[1,1]]`, tenue ≥ 1,5 croche, sans tirage. '
              'Durées porteuses : minimum observé sous 30 occurrences ; sinon plus petite durée (classes de ≥ 5 occurrences) dont le taux par note atteint 10 % du meilleur ; '
              'triolet = durée totale remplacée. Schéma : [Ornements](../README.md#ornements).', '',
              '## Forme longue V7b', '']
    set_data = tomllib.loads((PROFILES / name / 'set.toml').read_text()) if (PROFILES / name / 'set.toml').exists() else None
    template_set = style['set_template']
    kind_label = {'jig': 'jig → jig', 'reel': 'reel → reel', 'waltz': 'waltz → waltz'}[style['kind']]
    kind_set = tomllib.loads((PROFILES / template_set / 'set.toml').read_text())['key']
    lines += ['`set.toml` : sets de 2 à 5 airs (45·44·7·4, The Session), intro complète et entrées de pistes au premier air de chaque set. '
              f'Facteurs de tonalité = rapports observé / indépendance **{kind_label}** de The Session, repris de {template_set} : ' +
              ' · '.join(f'`{k}` {v:.2f}' for k, v in kind_set.items()) + '. Aucune mesure par tradition ni de pratique de set acadienne/britannique coloniale : '
              'valeurs de répertoire, à régler à l\'écoute. Rampe : +' + f'{style["ramp"][0]:g} bpm / +{style["ramp"][1]:g} d\'intensité, '
              f'plafond {style["meter"]["hi"]:g} (proposition, valeur du type voisin ; borne proxy cap-bretonnaise, **pas une mesure**). '
              'Raccords : ' + ', '.join(f'{k} {w}' for k, w, _ in style['transitions']) + '. Schéma : [Forme longue](../README.md#forme-longue-set).', '',
              '## Hétérophonie V4', '', n['hetero'], '', '## Limites', '', n['limits'], '']
    return '\n'.join(lines)


def satellites(name, built):
    """feet / arrangement / dynamics / set : modèles distribués adaptés ; seules les palettes de couleur et le set viennent du corpus."""
    style = STYLES[name]
    models = {'gigue_acadienne': ('gigue_quebecoise', 'gigue_quebecoise', 'gigue_quebecoise'),
              'reel_britannique_coloniale': ('hornpipe_britannique_coloniale', 'hornpipe_britannique_coloniale', 'gigue_britannique_coloniale'),
              'valse_britannique_coloniale': ('hornpipe_britannique_coloniale', 'hornpipe_britannique_coloniale', 'hornpipe_britannique_coloniale')}
    feet_model, arrangement_model, dynamics_model = models[name]
    feet = (PROFILES / feet_model / 'feet.toml').read_text()
    if name == 'gigue_acadienne':
        feet = feet.replace('la gigue québécoise', 'la gigue acadienne', 1)
    else:
        feet = f'# Table percussive (désactivée : le corpus britannique colonial ne décrit aucune podorythmie ; {style["name"].lower()} sans pieds).\n' \
               + feet.split('\n', 1)[1]
    arrangement = retitle((PROFILES / arrangement_model / 'arrangement.toml').read_text(),
                          f'Arrangement {style["de"]} : tours d\'un air, entrées progressives et intros idiomatiques.'
                          if name == 'gigue_acadienne' else
                          f'Arrangement {style["de"]} : violon et accompagnement léger, sans percussion.')
    dynamics = (PROFILES / dynamics_model / 'dynamics.toml').read_text()
    dynamics = retitle(dynamics.split('[[color]]')[0].rstrip('\n') + '\n\n' + color_toml(built['key_modes']),
                       f'Pilotage continu {style["de"]} par les émotions (Arousal, Tension, Valence, Density).')
    if not dynamics.endswith('\n'):
        dynamics += '\n'
    key_source = {'gigue_acadienne': 'Rapports observé / indépendance, jig → jig (The Session) : mêmes valeurs que la gigue québécoise,',
                  'reel_britannique_coloniale': 'Rapports observé / indépendance, reel → reel (The Session) : mêmes valeurs que le reel québécois,',
                  'valse_britannique_coloniale': 'Rapports observé / indépendance, waltz → waltz (The Session) : mêmes valeurs que la valse québécoise,'}[name]
    key_source += ' aucune mesure par tradition.'
    comment = {'gigue_acadienne': 'Raccords comme la gigue québécoise (appel de percussion possible).',
               'reel_britannique_coloniale': '# Pas d\'appel : ce profil n\'a aucun matériel de percussion. Raccords à valider à l\'oreille.',
               'valse_britannique_coloniale': '# Raccord direct seul, comme la valse québécoise ; aucune percussion à appeler.'}[name]
    if name == 'gigue_acadienne':
        comment = '# Raccords comme la gigue québécoise (appel de percussion d\'une mesure) : à valider à l\'oreille.'
    ramp_bpm, ramp_lift = style['ramp']
    set_text = set_toml(style['set_template'], comment, style['transitions'], ramp_bpm, ramp_lift, style['meter']['hi'], key_source)
    destination = PROFILES / name
    for filename, text in (('feet.toml', feet), ('arrangement.toml', arrangement), ('dynamics.toml', dynamics), ('set.toml', set_text)):
        (destination / filename).write_text(text)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('mode', choices=['selftest', 'build'])
    parser.add_argument('--only', choices=list(STYLES))
    args = parser.parse_args()
    if args.mode == 'selftest':
        selftest()
    else:
        endings = run_endings()
        for name in ([args.only] if args.only else STYLES):
            built = build_style(name, endings)
            destination = PROFILES / name
            destination.mkdir(exist_ok=True)
            (destination / 'profile.toml').write_text(built['profile'])
            (destination / 'cells.toml').write_text(built['cells'])
            satellites(name, built)
            (destination / 'README.md').write_text(readme(name, built))
            print(name, 'profile.toml', len(built['profile'].splitlines()), 'lignes ; cellules', built['cells_by_role'],
                  '; tonalités', built['key_modes'])
