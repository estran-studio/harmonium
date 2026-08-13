# Phase 1 — Modèle de données : position de transport

Normatif pour les cas limites.

## La valeur empaquetée

Un `AtomicU64` unique remplace `playhead_bar: Arc<AtomicUsize>`.

```
bits 63..32 : mesure (1-indexée, comme aujourd'hui)
bits 31..0  : step dans la mesure (0-indexé, grille ticks_per_beat)
```

Empaquetage et dépaquetage sont deux fonctions pures, testées isolément :

```
pack(bar, step)   = (bar as u64) << 32 | (step as u64 & 0xFFFF_FFFF)
unpack(v)         = ((v >> 32) as usize, (v & 0xFFFF_FFFF) as usize)
```

Invariant : `unpack(pack(b, s)) == (b, s)` pour tout couple représentable.

Valeur initiale : `pack(1, 0)` — mesure 1, début de mesure. C'est ce que
`AtomicUsize::new(1)` signifiait déjà (`audio.rs:73`, `audio.rs:174`).

**Un seul chargement rend toujours une paire cohérente.** C'est la raison
d'être de l'empaquetage : deux atomiques séparés se lisent en deux temps, et
un franchissement de barre entre les deux produit une position qui n'a
jamais existé — précisément au moment qui compte.

## La position exposée

L'accesseur public rend une position musicale, pas des bits :

```
TransportPosition {
    bar: usize,       // 1-indexée
    step: usize,      // 0-indexé dans la mesure
    beat: f32,        // 1.0 = premier temps ; step / ticks_per_beat + 1.0
}
```

`beat` est dérivé, jamais stocké : c'est la même formule que les deux
consommateurs actuels calculent chacun de leur côté
(`harmonium_practice/src/engine.rs:1339`, `harmonium_host/src/lib.rs:713`).
La centraliser ici supprime la duplication au passage.

## Points d'écriture

Tous les points qui écrivaient `playhead_bar` écrivent désormais la position
empaquetée (research R7) :

| Ligne | Contexte | Valeur |
|---|---|---|
| `playback.rs:242` | démarrage à `start_bar` | `pack(start_bar, 0)` |
| `playback.rs:293` | chaque buffer | `pack(bar courant, step courant)` |
| `playback.rs:375` | seek | `pack(target_bar, 0)` |
| `playback.rs:384` | seek (seconde voie) | `pack(target_bar, 0)` |

Un point d'écriture oublié ne casse rien de visible : la position se fige ou
recule silencieusement. D'où l'exigence de test sur le seek et le démarrage,
pas seulement sur la lecture continue.

## Lecture

- **Thread principal** (composer, génération) : inchangé dans l'esprit —
  `playhead_bar()` rend toujours la mesure, dérivée du champ empaqueté. Le
  `MusicComposer` n'a pas besoin du step.
- **Callback MIDI** : clone l'`Arc<AtomicU64>` une fois au démarrage de
  l'entrée MIDI, puis `load(Relaxed)` à chaque note. Aucun verrou.
- **UI** : continue de lire les rapports à 100 ms. La résolution au temps y
  suffit et rien ne change.

## Cas limites

- **En pause** — le thread audio n'avance plus la position, donc deux
  lectures successives rendent la même valeur. C'est le comportement
  attendu : deux notes jouées à l'arrêt portent la même position.
- **Avant la première lecture** — la valeur initiale `pack(1, 0)` est lue.
  Elle est indiscernable d'une vraie position au premier temps de la mesure
  1, ce qui est correct : c'est bien là qu'est le transport.
- **Changement de signature rythmique** — le step est compté dans la mesure
  courante ; le nombre de steps par mesure change avec la signature, et
  `beat` reste juste puisqu'il divise par `ticks_per_beat`, qui ne change
  pas.
- **Débordement** — 32 bits de mesure valent plus de quatre milliards de
  mesures. À 120 bpm en 4/4, c'est plus de soixante-cinq mille ans de
  lecture continue. Non traité délibérément.
