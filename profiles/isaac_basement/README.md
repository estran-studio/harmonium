# `isaac_basement` — essai Basement de *The Binding of Isaac*

Essai d'écoute de fin de phase 014 : peut-on approcher le Basement de
Ridiculon (*Diptera Sonata*, *Periculum*) avec les seuls mécanismes déjà en
place pour `nt_desert` ? Oui en données, sans ajout moteur. Références :
labo `references/isaac-basement/README.md` (G9-G0), dont les propositions
§7 sont reprises ici avec les corrections ci-dessous.

## Écriture

| Partie | Mesures | Harmonie (D mineur harmonique) | Basse | Chœur | Batterie |
|---|---:|---|---|---|---|
| A1 | 8 | pédale D | D1 réattaqué chaque mesure, relance C#2→D2 liée toutes les 2 mesures | tu | stomp |
| A2 | 4 | Dm Gm Dm Dm | montée F2 G2 A2 Bb2, une note par mesure, liée | tu | stomp |
| D | 4 | Bb A Gm A (V majeur, sensible C#) | descente Bb2 A2 G2 F2→E2, liée | plainte lente | stomp |
| Pont | 8 | G#m Gm Fm C#m, 2 mesures chacun (pont de *Periculum*) | fondamentale tenue + voisine chromatique | oui | tue |

Forme : A1 A2 D A2 D Pont A1' (40 mesures), deux tours. Les cellules du
chœur (`cells.toml`) sont composées, non transcrites : notes longues, plainte
Bb→A et sensible C#→D, aucun solo.

Écarts aux propositions du labo :

- Descente en accords **majeurs** Bb et A (VI et V du mineur harmonique) au
  lieu des Bbm/Am du détecteur, qui ne connaît que majeur/mineur ; à juger à
  l'oreille.
- Descente ramenée à 4 mesures (un motif de basse couvre 1 à 4 mesures) et
  reprise A2+D comme dans *Diptera Sonata*.
- Montée et descente à l'octave D2–Bb2 (leçon G7h) ; seule la pédale reste
  en D1.
- Valence haute : bascule vers D mixolydien (le validateur exige un mode
  clair parallèle à chaque mode sombre atteignable).

## Timbres

- SoundFont (GeneralUser GS) : basse doigts 33, nappe chœur 91, chœur
  Aahs 52, kit 128/0.
- Odin (`--features odin`, presets d'usine fournis par l'hôte) : Bass
  « Bass Crusher [RM] », Chords « Haunt [RM] ».

## Rendus

Hors dépôt, `harmonium_lab/corpus/isaac/ecoute/` :

```bash
./target/release/harmonium-cli --export --bars 80 --profile profiles/isaac_basement \
  --seed 1 --soundfont assets/GeneralUser-GS.sf2 --record-wav isaac_basement_generaluser.wav
# Odin : build --features odin, --odin-presets <dossier plat de .odin d'usine>
```
