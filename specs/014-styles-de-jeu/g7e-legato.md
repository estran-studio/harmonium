# G7e — Legato monophonique de la basse (fluide, sans réattaque détachée)

Retour du propriétaire : la basse de `nt_desert` est **incapable de changer de
note fluidement** ; la relance chromatique (mes. 4/8) et le riff D/D♯ sont
**détachés** — chaque hauteur attaque, la précédente se coupe avant, la ligne
est découpée. Priorité (choix b) : la fluidité au niveau des notes et du
lecteur SoundFont, **sans toucher au backend Odin** (G6d s'en charge, puis un
suivi branchera le glide).

## Décisions

1. **`Articulation::Legato`** dans le pipeline de notes (timeline/report), à
   côté de `Normal`/`Tied`. Aucun nom de style en code : le legato est une
   articulation de note lue depuis les données du profil
   (`BassMotifNote.legato`), un seul chemin (`accomp.rs::bass_motif_bar`).
   Interdit aux règles d'interprétation comme `Tied` : il vient des coups,
   jamais du profil d'interprétation.
2. **Playhead** (`timeline/pointers.rs`) : pour une note legato qui enchaîne
   une note tenue **d'une autre hauteur** sur le même canal, l'attaque nouvelle
   est émise puis la fin de la note précédente est **repoussée d'un léger
   chevauchement** (`LEGATO_OVERLAP_MILLI_STEPS = 200` millièmes de pas, ≈ 0,2
   pas ≈ 15–50 ms selon le tempo) au lieu de se couper avant. Une seule attaque
   par note, `NoteOn == NoteOff`, y compris à la barre et après invalidation
   (fin forcée `force_off`, état des voix remis à zéro en `seek_to_bar`).
   La même hauteur répétée (D → D au bord du motif) garde sa réattaque normale :
   le legato ne s'applique qu'au changement de hauteur.
3. **SoundFont** : le legato est réalisé par le chevauchement de notes du
   playhead ; `oxisynth` (dérivé de FluidSynth) **stocke** les contrôleurs de
   mode mono (CC 126) et de portamento (CC 5/65) mais **ne les interprète
   pas** — aucun mode mono ni glissando de hauteur n'est produit (limite XML,
   documentée dans `voice_manager.rs`). On n'émet donc pas de CC morts
   (principe IV). Voir « Limites ».
4. **Données `nt_desert`** : relance chromatique d'A1 (D–E–F♯–G–G♯, mes. 4/8)
   et riff D/D♯ d'A2 marqués `legato = true` ; **la pédale G♯ reste `Tied`**.
   Aucun octave ni harmonie ne change.
5. **Principe VIII** : la fixture `valse_basse_motivique` (valse 3/4) emploie
   le même « legato » sur sa marche de basse **sans changer ses hauteurs,
   durées ni vélocités** — seule l'articulation passe de `Normal` à `Legato`.
6. Vers les supports : voir « Écoute ».
7. **TODO documenté** : brancher le vrai glissando (portamento) du nouveau
   moteur Odin après la migration G6d — pas d'implémentation Odin ici.

## Fichiers touchés

- `harmonium_core/src/timeline/mod.rs` : variante `Articulation::Legato`.
- `harmonium_core/src/timeline/pointers.rs` : `PreparedEvent { legato,
  force_off }`, rang de tri, `prepare_playback`, `Playhead::tick` +
  `legato_attack`, constante `LEGATO_OVERLAP_MILLI_STEPS`, tri stable des
  événements.
- `harmonium_core/src/report.rs` : `Legato` non neutre (sérialisé dans les
  snapshots).
- `harmonium_core/src/performance.rs` : `Legato` refusé aux règles
  d'interprétation.
- `harmonium_core/src/trad/accomp_profile.rs` : `BassMotifNote.legato`.
- `harmonium_core/src/trad/accomp.rs` : articulation `Tied | Legato | Normal`.
- `harmonium_audio/src/voice_manager.rs` : commentaire de limite oxisynth.
- `profiles/nt_desert/profile.toml` : `legato = true` sur la relance et le riff.
- `tests/fixtures/profiles/valse_basse_motivique/profile.toml` : `legato = true`
  sur la marche (principe VIII).
- `harmonium_core/tests/legato_contract_tests.rs` : nouveau contrat public.
- `harmonium_core/tests/{part_tracks_off_tests,bass_motif_loading_tests}.rs` :
  littéraux `BassMotifNote` complétés du champ `legato`.

## Preuves de test

- `legato_contract_tests` (3 verts) :
  - **relance** : barre 4 → `(0,26)(0/200,32)(5,28)(5/200,26)(6,30)(6/200,28)
    (7,31)(7/200,30)(11,32)(11/200,31)` — une attaque par note, fin précédente
    chevauchée (offset 200), aucune orpheline ;
  - **riff D/D♯** : en A2, tout changement de hauteur lié (NoteOff.offset >
    NoteOn.offset), aucune orpheline ;
  - **principe VIII** : valse 3/4 → même marche `1–2–3 / 5–4–2` (hauteurs,
    durées 4), articulation `Legato`, lecture sans orpheline avec chevauchement.
- `pedale_une_seule_attaque…` (`bass_motif_tests`, 11 verts) et
  `pedale_liee_une_attaque_sans_orpheline` (`nt_desert_profile_tests`, 6
  verts) : toujours verts — la pédale garde une seule attaque, aucune orpheline
  avec le legato ; la valse legato ne change ni hauteurs ni invalidation/rejeu.
- **10 profils trad identiques** : `style_profile_tests` (goldens
  `main_golden_*`) sans régénération ; `TUNING_VERSION` inchangé (aucune donnée
  trad touchée, aucun `harmonium_*/src` altérant le flux trad).
- **`make quality` vert** : fmt nightly, les deux clippy (aucun nouvel
  avertissement dans les fichiers touchés), workspace sans studio, studio, et
  les six contrats Odin du host.

## Limites

- **SoundFont** : `oxisynth` ne fait ni mode mono (CC 126) ni portamento
  (CC 5/65). Le rendu SoundFont n'a donc **pas de glissando de hauteur** ; la
  fluidité vient du seul chevauchement (la note précédente relâche 0,2 pas après
  l'attaque, ce qui masque la coupure). Le vrai glide est renvoyé au moteur
  Odin (TODO ci-dessous).
- Le chevauchement est une **fraction de pas** (constante), donc sa durée en
  millisecondes suit le tempo du profil ; 200 millièmes de pas ≈ 15 ms au
  désert (pas ≈ 71 ms), davantage à tempo lent. Cohérent, mais pas réglé par
  profil.
- Le legato est **monophonique** par construction : il ne s'applique qu'à la
  note tenue d'une autre hauteur sur le canal. Une répétition de la même
  hauteur n'est pas un glissando.

## TODO — glide Odin (après G6d)

Une fois la migration de l'API Odin terminée (G6d, branche parallèle), brancher
le glissando réel du moteur Odin sur les notes `Articulation::Legato` : au
déclenchement d'une note legato sur un canal Odin, faire glisser la voix
tenue vers la nouvelle hauteur sans réattaquer l'enveloppe (portamento), au
lieu du chevauchement SoundFont. Aucun code Odin modifié ici ; G6d s'en charge.

## Écoute

Dossier hors dépôt :
`/Users/william.quintal/Project/bascanada/harmonium_lab/corpus/nuclear_throne/ecoute_g7/`.

Rendus **48 mesures**, graine 1, automation neutre (`g7f_neutre.toml` :
0,5 / 0,3 / 0,5 / 0,5), master +9 dB (profil), banque de pieds
`harmonium_trad.sf2`, aucun presets Odin (avertissements attendus) :

```sh
cargo run -q -p harmonium-cli --release -- --export --bars 48 \
  --profile profiles/nt_desert --seed 1 --automation "$ECOUTE/g7f_neutre.toml" \
  --soundfont "$BANQUE" --feet-soundfont "$BANKS/harmonium_trad.sf2" \
  --record-wav "$SORTIE.wav"
```

- `g7e_sf2.wav` (28 796 996 octets, 81,62 s) — banque `nt_desert.sf2`.
- `g7e_generaluser.wav` (28 796 996 octets, 81,62 s) — banque `GeneralUser-GS.sf2`.

Niveaux (volumedetect) : `g7e_sf2` crête **−2,7 dBFS** (moyenne −21,9),
`g7e_generaluser` crête **−6,7 dBFS** (moyenne −26,4) — aucun écrêtage ; la
crête du sf2 est un peu plus haute que G7f (−3,8) car le chevauchement legato
fait brièvement coexister deux voix de basse, sans dépasser 0 dBFS.

À comparer aux `g7f_sf2.wav` / `g7f_generaluser.wav` du même dossier (96
mesures) : mêmes sections, mêmes repères `g7f_sections.txt` pour le premier
tour (A1 / A2 / Pont / B / A1′). Écouter la relance de basse (mes. 4/8) et le
riff D/D♯ (A2) : plus de coupure entre les hauteurs, une seule attaque par
note, les fins chevauchent légèrement l'attaque suivante.