# Phase 0 — Recherche : W1, la voix d'accords

Relevé fait le 2026-08-13 sur `harmonium` @ `b5ddfcf`. Chaque décision est
adossée à une preuve dans le code, citée par fichier et ligne.

Format : Décision / Justification / Alternatives écartées.

---

## R0 — La prémisse de la spec est fausse (à corriger avant tout le reste)

La spec affirme : *« Voicing pitch selection already exists — this task is
about *when* voicings play, not *what* notes »*. Le code dit l'inverse.

**Preuves**

1. `harmonium_audio/src/voicing/` contient un pipeline complet — trait
   `Voicer`, `ShellVoicer` (guide tones), `BlockChordVoicer` (locked hands),
   `CompingPattern` euclidien, `apply_drop_two`, `get_guide_tones`. Un grep
   de `Voicer|VoicedNote|CompingPattern` sur l'ensemble des crates ne rend
   **aucune occurrence** en dehors de `harmonium_audio/src/voicing/`
   lui-même. Ce module est orphelin : personne ne l'appelle.
2. `enable_voicing` est câblé sur six couches — `params.rs:342,462`,
   `controller.rs:302`, CLI (`repl.rs:529`), VST GUI
   (`message_handler.rs:273`), `composer.rs:469`,
   `timeline_engine.rs:639` — et **n'est lu nulle part** dans le chemin de
   génération. Un grep de `enable_voicing` dans `harmonium_core/src`, hors
   déclarations et valeurs par défaut, ne rend rien. C'est un paramètre
   mort : l'utilisateur peut l'activer, il ne se passe rien.
3. `TrackId` (`timeline/mod.rs:45`) a quatre variantes — Bass, Lead, Snare,
   Hat. Aucune piste d'accords n'existe.

**Décision** : W1 n'est pas un raffinement rythmique posé sur des voicings
existants. C'est **le câblage de bout en bout d'une voix d'accords** :
génération de déclencheurs, choix des hauteurs, transport jusqu'à l'audio,
au snapshot, au mixer et au soundfont. La section « Proposed Implementation »
de la spec sous-estime le travail d'un facteur important, et sa section
« Assumptions » doit être corrigée.

**Ce que ça change pour le produit** : le constat de `STRATEGY_2026.md` —
« on n'entend pas l'harmonie » — n'est pas dû à un pattern rythmique qui
copie le lead. Il est dû à l'absence pure et simple de voix d'accords dans
le moteur. La cible du workstream est la bonne, sa description était douce.

---

## R1 — Où vit le choix des hauteurs

**Contrainte de dépendance** : `harmonium_audio` dépend de `harmonium_core`
(`harmonium_audio/Cargo.toml`), jamais l'inverse. Or le générateur de
timeline est dans `harmonium_core`. Le core **ne peut donc pas** appeler le
`Voicer` de `harmonium_audio`.

**Contrainte produit** : l'app de practice lit les notes depuis
`MeasureSnapshot` (`report.rs:37`, construit par
`MeasureSnapshot::from_measure` qui itère `TrackId::ALL`). Si les hauteurs
d'accords sont choisies en aval, dans la couche audio, elles n'apparaissent
jamais dans le snapshot — donc jamais à l'écran, jamais dans l'export MIDI
ou MusicXML, jamais dans le scoring de W2.

**Décision** : porter le choix des hauteurs dans
`harmonium_core/src/voicing/`, et **supprimer `harmonium_audio/src/voicing/`
dans le même changement**. Le code existant (`get_guide_tones`,
`apply_drop_two`, la logique shell et block-chord) est la semence du
portage, pas un module à garder en parallèle.

**Justification** : constitution II — quand l'architecture est mauvaise on
migre et on supprime l'ancien chemin dans le même changement. Garder les
deux créerait exactement le `_legacy` que la constitution interdit. La
suppression touche la surface publique (`harmonium_host/src/lib.rs:20`
réexporte `voicing`) mais aucun appelant n'existe.

**Alternatives écartées**
- Laisser le choix des hauteurs dans l'audio, ne faire transiter que des
  déclencheurs — écarté : les notes d'accords n'atteindraient ni le
  snapshot, ni l'écran, ni les exports.
- Inverser la dépendance des crates — écarté : refonte massive pour un
  bénéfice nul.

---

## R2 — `TrackId::Chord` : nouvelle variante ou réutilisation

*(ferme le NEEDS CLARIFICATION de la spec sur ce point)*

**Décision** : nouvelle variante `TrackId::Chord`, canal MIDI 4.

**Justification** : réutiliser `Lead` est exclu — en mode impro le Lead est
volontairement muet, et c'est précisément la situation où l'on a le plus
besoin d'entendre les accords. `muted_channels` est un `Vec<bool>` de 16
(`params.rs:622`, `ai.rs:131`), le canal 4 est libre.

**Le coût réel, énuméré** — ce que la nouvelle variante traverse :

| Site | Fichier | Ce qui casse ou doit suivre |
|---|---|---|
| Définition + `channel()` + `ALL` | `timeline/mod.rs:45-70` | `ALL` passe de 4 à 5 |
| Curseurs du playhead | `timeline/pointers.rs:87` | `track_cursors: [usize; 4]` → `[usize; 5]` |
| Émission d'événements | `timeline/pointers.rs:157` | `match track_id` exhaustif — bras à ajouter |
| Génération | `timeline/generator.rs` | 20 occurrences de `TrackId` |
| Export MusicXML | `timeline/export.rs` | 12 occurrences, portées/instruments |
| Export MIDI | `timeline/midi_export.rs` | 13 occurrences, pistes |
| Snapshot | `report.rs:72` | itère `TrackId::ALL` — suit tout seul |
| Mixer | `params.rs:622` | `muted_channels` indexé canal — suit tout seul |
| Soundfont | `harmonium_host` `set_channel_program` | routage programme GM du canal 4 |

Bonne nouvelle : les `match` exhaustifs font que **le compilateur énumère
lui-même** tous les sites à traiter. Le risque n'est pas d'en oublier un,
il est dans les deux tableaux de taille fixe.

---

## R3 — La liaison Charleston n'est pas représentable aujourd'hui

*(ferme le NEEDS CLARIFICATION sur la représentation de la liaison)*

C'est la découverte qui change le plan.

**Preuve** : `Playhead::tick()` (`timeline/pointers.rs:133-200`) **ne lit
jamais `duration_steps`**. Les NoteOff sont émis par remplacement :

- `TrackId::Bass` — la note active précédente est coupée quand la suivante
  arrive ;
- `TrackId::Lead` — `AudioEvent::AllNotesOff` sur le canal, puis la nouvelle
  note ;
- `Snare | Hat` — aucun NoteOff du tout.

Et `load_measure()` (ligne 114) remet `track_cursors = [0; 4]` à chaque
mesure, tandis que `advance_position()` pose `current_measure = None` en fin
de mesure. Rien ne survit à la barre.

`duration_steps` existe pourtant sur `TimelineNote` (`mod.rs:143`) et est
honoré par les exports MIDI et MusicXML — mais pas par la lecture audio.

**Conséquence** : FR-006 (« l'anticipation Charleston doit se prolonger
par-dessus la barre jusqu'au temps 1 ») est **inatteignable sans nouveau
travail**. Ce n'est pas un détail d'encodage, c'est un ordonnanceur de
NoteOff qui manque dans le playhead.

**Décision** : la liaison s'encode comme un `duration_steps` unique sur le
déclencheur de la mesure où il commence, autorisé à dépasser la longueur de
la mesure ; et le playhead gagne, **pour la piste d'accords uniquement**, un
ordonnancement des NoteOff par durée, en échéances absolues qui survivent au
changement de mesure. Les pistes existantes gardent leur sémantique de
remplacement — on ne change pas leur son en passant.

**Justification** : un `duration_steps` sur une note unique reste lisible
par les exports MIDI/MusicXML qui l'honorent déjà. Découper la liaison en
deux notes sur deux mesures produirait une double attaque à l'oreille et un
faux rythme dans la partition.

**Alternatives écartées**
- Généraliser tout de suite l'ordonnanceur de durée à toutes les pistes —
  écarté pour ce workstream : ça changerait le son du lead et de la basse,
  effet de bord non désiré dans un changement qui doit rester audible mais
  additif. À poser comme spec séparée si on veut assainir.
- Représenter la liaison comme une note dupliquée sur la mesure suivante —
  écarté : double attaque.

---

## R4 — `hits_per_bar` au-delà de la résolution de grille

*(ferme le NEEDS CLARIFICATION correspondant)*

**Décision** : clamp au nombre de steps de la mesure. Jamais de rejet,
jamais d'erreur.

**Justification** : ces paramètres arrivent de deux sources continues — le
morphing (`morph_factor = 0.03`, valeurs interpolées en continu) et le
mapping émotionnel de `harmonium_ai` (`mapper.rs:158`). Une valeur
transitoire hors bornes est normale en cours de morphing ; refuser
provoquerait une erreur dans le chemin audio, ce qui est inacceptable. Le
reste du host clampe déjà systématiquement (`clamp(0.0, 1.0)` dans
`timeline_engine.rs:721,725`) — on suit la convention en place.

`hits_per_bar = 0.0` doit produire une piste silencieuse sans division par
zéro, et non une erreur.

---

## R5 — Déterminisme : un flux d'aléa dédié au comping

**Le piège** : le générateur tire son aléa d'un `&mut dyn RngCore` unique
issu du `ChaCha8Rng` de session. Insérer des tirages de comping **au milieu**
de ce flux décale tous les tirages suivants : à graine identique, la mélodie
et la batterie changeraient. FR-011 (« les tests existants doivent passer »)
tomberait, et pire, toute session sauvegardée se rejouerait différemment.

**Décision** : le comping tire dans un RNG enfant, dérivé de façon
déterministe de `(session_seed, index de mesure)`, et jamais dans le flux
principal. Le flux existant reste bit-à-bit identique quand le comping est
actif comme quand il ne l'est pas.

**Justification** : constitution I — tout ce qui peut être semé doit l'être,
et un test doit pouvoir rejouer la sortie à l'identique. La dérivation par
mesure donne en prime la reprise en cours de morceau (`SeekPlayhead`,
`reset_to_initial()`) sans rejouer depuis la mesure 1.

**Vérification à inscrire dans les tests** : rendre 100 mesures avec comping
désactivé puis activé, à graine identique, et affirmer que les notes de
Bass/Lead/Snare/Hat sont strictement inchangées.

---

## R6 — `CompingParams` et les profils de style

**Constat** : `TuningParams` existe (`tuning.rs:434`) avec neuf
sous-structures et une méthode `validate()` qui vérifie les bornes champ par
champ. La spec évoquait « CORELIB-23 » comme un préalable ; il est livré.

**Décision** : `CompingParams` devient la dixième sous-structure de
`TuningParams`, avec sa couverture dans `validate()`. Le champ porte un
`serde(default)` pour que les 15 profils de style existants
(`harmonium_training/static/profiles/`, autre repo) continuent de se charger
sans modification.

**Frontière inter-repo** (constitution V) : le moteur livre la structure et
ses valeurs par défaut. Le peuplement des 15 profils avec les valeurs du
tableau de la spec est une tâche du repo `harmonium_training`, à référencer
depuis ce spec, pas à faire ici.

---

## R7 — SC-005 n'est pas mesurable en l'état

*(ferme le dernier NEEDS CLARIFICATION)*

**Preuve** : `harmonium_lab` expose `GlobalMetrics` (`dna_types.rs:255`) avec
sept champs — effort de conduite des voix, variance de tension, balance
tension/détente, pourcentage diatonique, rythme harmonique, durée, nombre de
changements d'accord. **Aucune métrique de comping.** Le « score composite
d'authenticité du comping » que SC-005 invoque n'existe pas.

**Décision** : retirer SC-005 de cette spec et le remplacer par des critères
mesurables dans le moteur lui-même (indépendance des déclencheurs, moyenne
de frappes par mesure, taux de mesures muettes, déterminisme). Construire
une métrique d'authenticité du comping dans `harmonium_lab` est une spec de
ce repo-là, à écrire séparément.

**Justification** : un critère de succès qui dépend d'un outil inexistant
n'est pas un critère, c'est un vœu. Mieux vaut l'admettre que de laisser une
case impossible à cocher.

---

## R8 — Le critère d'arrêt est musical, pas statistique

Les seuils de la spec (±20 % de frappes par mesure, ~15 % de mesures
muettes) sont nécessaires mais ne disent pas si le résultat *sonne*. Le
critère qui compte, hérité de `STRATEGY_2026.md` : sur un blues en Si♭, on
entend piano + basse + batterie, et on peut suivre les changements à
l'oreille sans regarder l'écran.

**Décision** : ce jugement d'écoute est une porte de recette explicite, pas
un test automatisé. Il est inscrit dans `quickstart.md` et dans la phase de
finition. Les tests statistiques prouvent que le générateur fait ce qu'on lui
demande ; l'écoute prouve qu'on lui a demandé la bonne chose.
