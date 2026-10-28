# `nt_desert` — Désert de *Nuclear Throne*

Références d'analyse : *Fläshyn* (Desert A) et *A Way* (Desert B),
Jukio Kallio, Vlambeer, 2015. Corpus : `harmonium_lab`,
`corpus/nuclear_throne/manifest.jsonl`, pistes 02 et 19, droits réservés.
Audio, stems et transcriptions restent hors dépôt. Les grilles de basse,
batterie et harmonie proviennent de `references/nt-desert/README.md`, §6.

**G7f est une proposition composée, pas une transcription.** La présence
supposée d'un sifflement traversant *Fläshyn* n'est pas établie. L'extraction
`19_a_way_lead_motif` (~7 % de voix utile) ne justifie pas de la présenter
comme un motif fiable. Aucun programme 78 n'est employé.

## Écriture

12/8, 24 pas par mesure ; 210 noires/min (= 140 noires pointées/min).
Deux tours de 48 mesures par air, un air par set ; raccord direct sans outro.
Le deuxième tour garde les pistes déjà entrées : il n'est pas une nouvelle
introduction. Les entrées se réappliquent au prochain air.

| Section | Mesures du tour | Fonction | Couches au premier tour |
|---|---|---|---|
| A1 | 1–8 | Pédale **G♯**, relance chromatique de basse toutes les quatre mesures | Basse 81 et nappe 89 seules |
| A2 | 9–16 | Riff **D/D♯**, harmonie D mineur | Entrée batterie et motif de dessus doux 88 |
| Pont | 17–32 | C C E E F B C C / F C E E F B C♯ C♯ | Nappes seules ; `Bass`, `Heel`, `Toe`, `Melody`, `Countermelody` coupées |
| B | 33–40 | G/G♯ par blocs de deux mesures, basse ponctuelle | Retour du motif dans un registre abaissé de cinq demi-tons avant réalisation harmonique |
| A1' | 41–48 | Retour de la pédale et de sa relance | Dessus et batterie restent présents ; doublure seulement aux mesures 89–96 |

Le dessus est un **pad mélodique 88**, non un sifflet ni un solo rapide.
Une seule cellule d'appel : tenue de deux pulses, puis deux notes descendantes
(`8 — 6 — 5`). En A2, graine 1 neutre : **A5–F5–D5**. La réponse plus
espacée repart de `5` vers `8`, avec une variante intermédiaire rare.
Les rôles alternent appel et réponse ; les variations `varied_end` et
`repeated_notes` restent parcimonieuses (reprise 0,15, tour 0,20), sans
transposition d'octave de cellule. L'hétérophonie double le dernier retour
une octave plus bas, à −14 dB, sans mélodie indépendante.

Mètre 12/8, 24 pas par mesure, 210 noires/min (= 140 noires pointées/min).
Batterie en groove (G7c/G7d) : grosse caisse aux pas 0/8/20, caisse claire
aux pas 2/10/14 ; le charleston (seul son de bruit du kit employé) est
réduit aux accents de temps — pas 0 et 12 à la densité neutre, pulsations
6/18 seulement à la densité 2 — et sonne sous la caisse claire ; temps fort
plein, autres temps ×0,8 dans les motifs, accent du temps fort et
micro-timing ±0,02 pas en `[performance.tracks.Heel/Toe]`. Instruments :
nappe chaude 89 (vélocité ×0,8, tessiture médium, jamais sous MIDI 48),
scie grave 81 sur la basse seule (deux octaves plus bas, seule voix grave),
kit 128/0 (touches 36, 38, 42).
Le gain master (9 dB, SoundFont seulement) ne touche pas les voix Odin.
Basse et nappe restent à 0 dB au niveau de piste ; les presets d'usine
`Sausage Bassener` (Bass) et `Darken Nature [RM]` (Chords) conservent leur
master, sans boost de compensation de l'ancien moteur. Le morphing pilote
leurs filtres et enveloppes en unités physiques. Batterie à −5/−6 dB avec
peu de salle (envois 0,06) ; les autres réglages de mix ne changent pas.

`CopyOf`/`CopyOfPart` copieraient aussi les mesures de basse : ils ne sont pas
employés pour ne pas effacer la relance d'A1 ni importer le riff d'A2 en B.
La répétition vient de la cellule d'appel unique et des rôles écrits.

Les accords du pont, d'A2 et de B gardent des triades complètes, doublées à
l'octave (`strings = 4`) et liées. **La pédale d'A1 reste une seule note**,
pas une triade ajoutée au bourdon. Les fills de toms 48–45–41 remplacent le
dernier temps aux frontières `phrase_ends = [8]` d'A1/A2/B ; ils ne jouent
ni dans l'ouverture sans batterie, ni au pont. À haute densité, quelques
réponses de caisse claire s'intercalent ; à basse densité, un seul tom grave.
Le groove courant et son micro-timing ±0,02 pas restent ceux de G7c/G7d.

## Timbres et émotions

Tous les programmes nécessaires sont présents dans `assets/nt_desert.sf2`
et GeneralUser GS : dessus/doublure 88, nappe 89, basse 81, kit 128/0
(36, 38, 42, 41, 45, 48). Aucun preset de clavier absent de la banque du jeu
n'est requis, donc aucune variante muette de secours. Banque procédurale CC0 :
`harmonium_lab/references/nt-desert/banque.md`.

Mix G7d conservé : master SoundFont +9 dB, basse/nappe 0 dB, batterie
−5/−6 dB. Nouveau dessus à −8 dB ; doublure à −14 dB. Les configurations
Odin existantes de basse/nappe restent inchangées ; G7f ne touche pas leur API.

- **Arousal** : tempo et vélocités, densité de batterie et fills.
- **Density** : basse ≥ 0,25, pieds ≥ 0,30/0,35, dessus ≥ 0,40,
  doublure ≥ 0,50 après son entrée ; à 0,20, nappe seule.
- **Tension** : accents de contretemps et notes plus sèches ; pas de dominante
  substituée à l'harmonie déclarée.
- **Valence** : G♯ phrygien au neutre, A phrygien possible au sombre,
  mixolydien au clair ; couleur et registre suivent les courbes existantes.

## Limites et écoute

Le modèle `loop` rapproche les porteuses des sons de l'accord : sur la
pédale pure, le dessus reste sur G♯ (avec possibles octaves), pas une gamme
libre au-dessus du bourdon. A2 reste D mineur plutôt que la superposition
D–D♯–A de la référence. Le programme est unique par piste, pas par section.
Aucune de ces limites n'est contournée par du code moteur ; pas de travail
G7e legato ni G6d Odin dans cette branche.

Rendus avant/après, mesures par section et repères d'écoute :
[`g7f-nt-desert-riche.md`](../../specs/014-styles-de-jeu/g7f-nt-desert-riche.md).
Les mesures ne constituent pas une validation d'écoute humaine.
