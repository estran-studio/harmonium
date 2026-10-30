# Z1 — Zones de timbre Odin sur tous les profils de jeu

Données seulement : `[[odin.Bass.zones]]` et `[[odin.Chords.zones]]` de `nt_desert`,
`isaac_basement`, `isaac_cellar`, `isaac_burning`, `isaac_downpour`. Aucun code moteur,
aucune section musicale. Demande du propriétaire : « le plus possible » de zones, après la
bascule A → B d'`isaac_burning` jugée « très bien ».

## Décisions

- **Trois ambiances, même ordre partout** (première zone qui contient l'émotion gagne) :
  1. **étrange** `tension = [0.6, 1.0]` : plus psychédélique (celle d'`isaac_burning`, validée,
     est conservée telle quelle) ;
  2. **combat** `arousal = [0.7, 1.0]` : plus agressif ;
  3. **menace** `valence = [-1.0, -0.3]` : plus sombre ;
  4. **défaut** : le son actuel, inchangé (validé à l'écoute).
  Aucune zone ne capture les émotions par défaut (`arousal 0.5, valence 0.3, density 0.2,
  tension 0.4`) : rendu par défaut identique octet pour octet à `main` (ci-dessous).
- **Choix des presets** parmi les 260 presets d'usine : analyse du `.odin` (matrice de
  modulation : LFO vers hauteur = vibrato, vers volume = trémolo ; arpégiateur) pour écarter
  les presets qui vibrent, puis audition mesurée sur le solo de piste d'`isaac_basement`
  (niveau, brillance, modulation d'amplitude 4–12 Hz). Goûts du propriétaire : sons aimés
  réemployés (Bass Low, Bass Crusher, Moonshade, Haunt, Darken Nature, Growly FM Bass) ; Phazzy
  Organ et Sausage Bassener jamais ajoutés (Sausage Bassener reste le défaut validé de
  `nt_desert`).
- **Niveaux** : `level_db` de zone = RMS du solo de piste du défaut moins celui du preset de
  zone (16 mesures, graine 1, mêmes règles de morphing), arrondi à 0,5 dB, borné à −24..+12.
- Les règles de morphing de chaque piste sont reprises à l'identique dans toutes ses zones.
- `nt_desert` Chords « combat » : Synthwave Pad écrêtait (+2,0 dBFS de crête pour −24,9 dB RMS)
  → remplacé par PAD Open. Le lead (`Melody`) de `nt_desert` reste SoundFont.
- `isaac_downpour` « étrange » : Bass Crusher et Eerie Pad restaient 2–3 dB sous le défaut même
  à +12 dB (notes de deux mesures, attaque douce) → BS PWM Bass XY et Haunt.

| Profil | Piste | Zone | Preset | `level_db` |
|---|---|---|---|---:|
| `nt_desert` | Bass | étrange | Bass Low [RM] | −4,5 |
| `nt_desert` | Bass | combat | Growly FM Bass [XR] | −2,5 |
| `nt_desert` | Bass | menace | DeepBass [Photonic] | +4,5 |
| `nt_desert` | Chords | étrange | Moonshade | −5,5 |
| `nt_desert` | Chords | combat | PAD Open | −2,5 |
| `nt_desert` | Chords | menace | Alone in the Subway | −0,5 |
| `isaac_basement` | Bass | étrange | Bass Low [RM] | −14,5 |
| `isaac_basement` | Bass | combat | Growly FM Bass [XR] | −13,0 |
| `isaac_basement` | Bass | menace | DeepBass [Photonic] | −7,0 |
| `isaac_basement` | Chords | étrange | Moonshade | −7,5 |
| `isaac_basement` | Chords | combat | PAD Open | −4,0 |
| `isaac_basement` | Chords | menace | Church Organ | −4,5 |
| `isaac_cellar` | Bass | étrange | Bass Low [RM] | −3,0 |
| `isaac_cellar` | Bass | combat | Bass Crusher [RM] | +3,5 |
| `isaac_cellar` | Bass | menace | DeepBass [Photonic] | +1,5 |
| `isaac_cellar` | Chords | étrange | Moonshade | −11,0 |
| `isaac_cellar` | Chords | combat | Darken Nature [RM] | −6,5 |
| `isaac_cellar` | Chords | menace | Alone in the Subway | −6,0 |
| `isaac_burning` | Bass | étrange | Bass Low [RM] (inchangé) | −5,0 |
| `isaac_burning` | Bass | combat | Growly FM Bass [XR] | −5,0 |
| `isaac_burning` | Bass | menace | DeepBass [Photonic] | −3,0 |
| `isaac_burning` | Chords | étrange | Moonshade (inchangé) | −7,0 |
| `isaac_burning` | Chords | combat | Prophet | −3,0 |
| `isaac_burning` | Chords | menace | Clouded [RM] | 0,0 |
| `isaac_downpour` | Bass | étrange | BS PWM Bass XY | +9,5 |
| `isaac_downpour` | Bass | combat | Growly FM Bass [XR] | +1,0 |
| `isaac_downpour` | Bass | menace | BS Decay Bass MW | +6,0 |
| `isaac_downpour` | Chords | étrange | Haunt [RM] | +5,0 |
| `isaac_downpour` | Chords | combat | Darken Nature [RM] | +3,5 |
| `isaac_downpour` | Chords | menace | Drifter | +3,0 |

## Preuves

- Rendu `--features odin`, émotions par défaut, graine 1, 16 mesures : **identique octet pour
  octet à `main`** pour les cinq profils (`cmp`).
- Rendus d'écoute, graine 1, 80 mesures, `--soundfont assets/GeneralUser-GS.sf2
  --odin-presets ~/Project/bascanada/odin-presets` ; automation
  [`z1-zones-automation.toml`](z1-zones-automation.toml) (Isaac) et
  [`z1-zones-automation-nt.toml`](z1-zones-automation-nt.toml) (`nt_desert` : la menace entre
  dès A2, car le pont tait basse et batterie). Temps de bascule effectifs tirés de la vérité
  exportée (tempo variable avec l'arousal), crête et RMS par segment :

| Rendu | menace | combat | étrange | retour au défaut | crête max |
|---|---|---|---|---|---:|
| `nuclear_throne/ecoute_g7/z1_nt_desert_zones.wav` | 0:13,7 | 0:54,2 | 1:19,4 | 1:46,3 | −2,2 dBFS |
| `isaac/ecoute/z1_isaac_basement_zones.wav` | 0:27,3 | 0:54,2 | 1:19,4 | 1:46,3 | −5,4 dBFS |
| `isaac/ecoute/z1_isaac_cellar_zones.wav` | 0:27,3 | 0:54,2 | 1:19,4 | 1:46,3 | −5,6 dBFS |
| `isaac/ecoute/z1_isaac_burning_zones.wav` | 0:27,3 | 0:54,2 | 1:19,4 | 1:46,3 | −3,1 dBFS |
| `isaac/ecoute/z1_isaac_downpour_zones.wav` | 0:26,6 | 0:52,7 | 1:17,3 | 1:43,4 | −5,0 dBFS |

- RMS des segments à ±1,5 dB du défaut sur chaque profil (le combat change aussi tempo et
  batterie par l'arousal).

## Limites

- Choix et niveaux mesurés, pas écoutés : le jugement appartient au propriétaire (studio : curseurs
  valence, arousal, tension ; la bascule attend la frontière de partie).
- Niveaux appariés en RMS sur 16 mesures : un preset à attaque lente ou à release long peut
  sonner plus ou moins présent que son chiffre.
- Le passage de `nt_desert` au pont (17–32) tait basse et batterie : la menace y n'est audible
  que sur les accords.
