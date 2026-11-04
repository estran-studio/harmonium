# B3 — Profils latins : bossa nova, samba, afro-cubain, baião

Date : 2026-11-03. Branche `wquintal/b3-latin`, partie de `main` à `eecbd3f` (`TUNING_VERSION` 26, **inchangé**). Données
seulement : aucun fichier de `src/` d'un crate, aucun mécanisme ajouté ; quatre dossiers de profil **neufs**, quatre tests de
contrat, scripts de reproduction, un script de mesure partagé. Les profils existants ne sont pas touchés.

Un compte rendu détaillé par profil (choix, preuves avec commandes, mutations, manques, limites) est dans `b3/` ; celui-ci
en est la synthèse et porte les vérifications d'acceptation.

## Ce qui est livré

| Profil | Source du kit (GMD, CC BY 4.0) | Fiabilité | Tempo (plage) | Comping / basse / kit par défaut | Particularité |
|---|---|---|---|---|---|
| `gmd_latin_brazilian_bossa` | `latin/brazilian-bossa` : 2 fichiers, 1 batteur, 631 mesures | **trop peu** (écrit en tête du README) | 127 (104–180) | guitare nylon 24, basse acoustique 32, kit jazz 128/32 | comping syncopé sur deux mesures, basse fondamentale-quinte anticipée, cross-stick et charleston/ride |
| `gmd_latin_brazilian_samba` | `latin/brazilian-samba` : 5 fichiers, 1 batteur, 322 mesures ; relances : `latin/samba` `fill` (12 fichiers retenus) | juste | 89 (85–120) | guitare nylon 24, basse acoustique 32 (deux appuis), kit standard 128/0 | surdo = tom 43 aux pas 4 et 12, tamborim (wood block), agogô, chocalho, pandeiro |
| `gmd_afrocuban` | `afrocuban` droit (1 fichier, 98 mesures) et `afrocuban/rhumba` (1 fichier droit, 103 mesures ; 18 fills) | trop peu par groupe | 105 (105–128) | piano 0 (montuno), basse acoustique 32 (tumbao), kit standard 128/0 | **seul à `[clave]`** : son 3-2 / 2-3 tirée par air, phrases de comping et de batterie alignées |
| `gmd_latin_brazilian_baiao` | `latin/brazilian-baiao` : 3 fichiers, 2 batteurs, 200 mesures ; 29 fills | juste (deux batteurs) | 95 (95–110) | accordéon 21, basse aux doigts 33, kit standard 128/0 | zabumba, triangle 80/81, agogô ; mixolydien |

Chaque dossier : `profile.toml`, `percussion.toml` (généré), `arc.toml`, `dynamics.toml`, `set.toml`, `arrangement.toml`, `cells.toml`,
**`session.toml`** (28 à 33 variantes en 9 groupes : tempo, tonalité, comping, basse, kit, voicings, registre, énergie, arc), `README.md`.
Sans Odin (D1) : GeneralUser GS, programmes et touches vérifiés dans la banque (`/tmp/harmonium-019/b3/sf2keys.py` : kits 0, 8, 16, 32, 40
contiennent les touches latines 54 à 82 ; 37 = Jazz Rim Shot). Mentions de licence (GMD CC BY 4.0, WJazzD ODbL §4.3) dans `[meta] corpus` et chaque README.

**Songo non fait** : `latin/brazilian-songo` (5 fichiers, « juste ») n'est pas le songo cubain du pack Salsa d'iReal (`l2-ingestion.md` § 5) ;
nommer un profil d'après lui serait une étiquette trompeuse (principe VII).

## Ce qui est mesuré, ce qui est de métier (principe VII, D5)

- **Mesuré** : la batterie du kit (GMD : groupes de mesures, patterns par densité, phrases de 2 et 4 mesures, relances quand des
  fichiers `fill` existent), les tempos (GMD et plage de session), le registre de la basse, le rythme harmonique et les approches
  (WJazzD `LATIN`, « assez », 23 solos, 1 639 mesures valides : **hard bop sur rythme latin**, il ne distingue ni bossa, ni samba, ni
  afro-cubain ; **les croches de basse n'y sont pas observables**).
- **Métier, dit comme tel** : clave, montuno, tumbao, congas, cloche, surdo, tamborim, triangle, comping, motifs de basse, grilles,
  arc, mixage. Le GMD ne contient **aucune percussion latine** (kit électronique Roland TD-11, dix batteurs). Le tumbao n'est donc pas
  « mesuré » (réponse à la clause « si mesuré » du brief : non). Le statut juridique d'un idiome rythmique traditionnel codé en
  règles (clave, montuno) n'est pas vérifié : **décision du propriétaire** (déjà notée par L2 § 5), écrite dans le README de `gmd_afrocuban`.
- BRID (samba, audio CC BY 4.0) n'est pas ingéré : une chaîne de détection d'attaques en plus, fiabilité à mesurer.

## Habillage par graine (D6)

`mesures_latin.py habillages --seeds 16 --bars 48` (binaire de la branche, lu dans `MeasureSnapshot.session`, `instruments`,
`composition_bpm`, `position`) ; « grappes de tempo » : tempos à ± 8 BPM.

| Profil | Habillages distincts / 16 graines | Grappes de tempo (± 8 BPM) | Tonalités | Triplets de programmes (comping, basse, kit) | Kits (banque, programme) |
|---|---:|---:|---:|---:|---|
| `gmd_latin_brazilian_bossa` | 16 | 5 | 6 | 15 | 128/0, 128/8, 128/32, 128/40 |
| `gmd_latin_brazilian_samba` | 14 (le test Rust, autre correspondance graine → tirage, en lit 16) | 3 | 6 | 10 | 128/0, 128/8, 128/40 |
| `gmd_afrocuban` | 16 | 2 | 6 | 11 | 128/0, 128/8, 128/16, 128/32, 128/40 |
| `gmd_latin_brazilian_baiao` | 16 | 2 | 6 | 11 | 128/0, 128/8, 128/32, 128/40 |

Mesures refaites sur les profils livrés (dynamique de tempo comprise). Limite de la source : les tempos mesurés ne couvrent que 89–116 (samba), 105–128
(afro-cubain), 95–110 (baião) BPM : trois grappes à ± 8 BPM ne tiennent pas sans étendre arbitrairement ; la plage des variantes reste dans la source (bossa :
WJazzD `LATIN` donne 104 à 228 ; les 5 grappes de la bossa viennent du décalage d'arousal de ± 4 BPM ajouté aux plages de variantes). Le tempo suit l'arousal de ± 4 BPM
(samba, bossa, afro-cubain, baião) : le contrat générique `time_plan_playback_tests` exige qu'une commande d'émotion déplace le tempo (il a échoué sur `gmd_afrocuban`, puis sur `gmd_latin_brazilian_bossa`, à courbe
plate avant correction ; la courbe n'est pas écrêtée aux bornes du profil).

## Variété (`seed-variety --chorus 4`, 16 graines, même binaire pour V0)

Chaque cellule : mesures changées d'un chorus au suivant · suites distinctes sur 16 graines · distance inter-graines (détails et autres
couches dans les fragments).

| Couche | bossa livré / sans `session` / V0 | samba livré / sans `session` / V0 | afro-cubain livré / sans `session` / V0 |
|---|---|---|---|
| comping, voicings | distance inter-graines 98 % / 90 % / 46 % | 92 % · 16 · 95 % / 90 % · 16 · 88 % / 74 % · 16 · 46 % | 87 % · 16 · 99 % / 82 % · 16 · 95 % / 74 % · 16 · 46 % |
| basse | — | 20 % · 16 · 89 % / 19 % · 16 · 91 % / 92 % · 16 · 59 % | 49 % · 16 · 91 % / 46 % · 16 · 90 % / 92 % · 16 · 59 % |
| batterie | changées d'un chorus au suivant 100 % / 100 % / 0 % | 100 % · 16 · 13 % / 100 % · 16 · 13 % / 0 % · 11 · 19 % | 100 % · 16 · 45 % / 100 % · 16 · 44 % / 0 % · 11 · 19 % |
| accompagnement | inter-graines 65 % / 63 % / 38 % | 100 % · 16 · 60 % / 100 % · 16 · 60 % / 99 % · 16 · 38 % | 100 % · 16 · 75 % / 100 % · 16 · 74 % / 99 % · 16 · 38 % |
| orchestration | **11 suites / 16 · inter-graines 44 %** / 1 / 1 | 1 programme par piste sans habillage | — |

Baião : batterie 81 / 88 / 0 % (paires de chorus identiques 5 / 17 / 100 %), basse 43 / 44 / 92 % (détail : fragment).

Lecture : l'habillage change surtout ce qui s'entend d'une graine à l'autre (tempo, tonalité, programmes, kit, voicings : orchestration de
1 à 11 suites distinctes pour la bossa) et peu les couches de notes déjà variées par S0, l'arc et les phrases. **La basse change peu d'un chorus à
l'autre** (20 à 49 %, V0 : 92 %) : plan de motifs fixé par air, limite du moteur déjà relevée par B4 + B7 et B8.

## Basse contre WJazzD `LATIN` (`mesures_latin.py basse`, ce que la source mesure, échantillonné au temps)

| Mesure | WJazzD `LATIN` | bossa | samba | afro-cubain | baião |
|---|---|---|---|---|---|
| registre p1 / p50 / p99 | 31 / 40 / 50 | 34 / 39 / 45 | 31 / 41 / 46 | 36 / 41 / 46 | 34 / 39 / 44 |
| arrivée sur la basse de l'accord au changement (arrivées comparées) | 57 % [46–68] | 100 % (385) | 100 % (351) | 100 % (158) | 100 % (89) |
| approche à un demi-ton avant un changement à la barre (temps 4 → arrivée) | chromatique 21,6 % | 23,3 % | 18,6 % | 14,3 % | 12,0 % |
| approche à deux demi-tons (WJazzD : diatonique 18,5 %, non équivalent) | 18,5 % | 44,2 % | 44,1 % | 13,5 % | 0 % |
| changement au temps 3 / mesures à un accord | 15,1 % / 86,9 % | 10,6 % / 89,3 % | 9,4 % / 91,4 % | 0 % / 100 % | 0 % / 100 % |

Mesuré par `mesures_latin.py basse --seeds 8 --bars 48` sur les profils livrés (le tableau des fragments, mesuré par chaque agent sur d'autres graines, donne des valeurs voisines).
L'arrivée à 100 % est le choix des motifs (ils arrivent toujours sur la basse publiée : contrat de test, D4), pas une reproduction de la source : dans 43 % des changements
WJazzD arrive sur une autre note que la basse de l'accord. Aucun score agrégé : le validateur du labo (`backing.validate`) ne lit que des mesures à une note par temps et ne s'applique pas à des riffs ou des gabarits.

## Distance de la batterie du kit au GMD (Bray-Curtis, 0 = identique, voix latines exclues)

| Profil | Livré | Sans arc | Copie non calibrée |
|---|---:|---:|---:|
| bossa | 0,203 | — | 0,720 |
| samba | 0,354 | 0,367 | 0,863 |
| afro-cubain | 0,301 | 0,300 | 0,786 |
| baião | 0,157 | 0,074 | 0,844 |

Le point faible commun est la caisse claire (coups fantômes du batteur du GMD sur presque tous les pas). Baïão : l'arc éloigne la batterie du
corpus (le charleston seul en exposition et en retombée) de 8 points par rapport au profil sans arc : choix de service.

## Mixage (LUFS intégrés en solo, graines 1 et 2, 48 mesures)

| Profil | accords | basse | batterie | mix | crête vraie |
|---|---:|---:|---:|---:|---:|
| bossa | −25,5 | −24,8 | −26,0 | −20,9 | −2,7 dBFS (−1,3 sur les graines 3 à 8) |
| samba | −26,6 | −25,6 | −26,4 | −22,0 | −1,3 dBFS (16 graines) |
| afro-cubain | −26,1 | −25,4 | −25,5 | −21,3 | −4,3 dBFS |
| baião | −25,9 | −25,1 | −25,4 | −21,0 | −3,6 dBFS |

Le niveau varie avec le timbre tiré (bossa : accords de −30,7 à −22,8 selon la graine) : les niveaux sont calés sur la moyenne. Un seul niveau
par piste pour plusieurs kits (baião : 2,5 dB mesurés entre deux graines). `PercussionLow` et `PercussionHigh` partagent le canal 9.

## Contrat

| Fichier | Tests | Mutations vérifiées (chacune fait échouer ≥ 1 test, puis annulée) |
|---|---:|---:|
| `gmd_latin_brazilian_bossa_profile_tests.rs` | 12 | 17 |
| `gmd_latin_brazilian_samba_profile_tests.rs` | 11 | 18 |
| `gmd_afrocuban_profile_tests.rs` | 16 | 29 |
| `gmd_latin_brazilian_baiao_profile_tests.rs` | 11 | 12 |

Contrats communs : grille par défaut = une boucle écrite, basse sur les changements (fenêtre du profil), comping dans l'accord publié (celui de la mesure
suivante pour une attaque anticipée), batterie dans le kit et voix latines sur leurs pistes, feel droit, tempo dans la plage de chaque variante
(± 4 BPM d'arousal), arc (exposition et retombée plus légères que le sommet, stop-time), même graine = même sortie, habillages distincts. Afro-cubain : **clave
respectée mesure par mesure** aux trois densités et trois intensités, comping et batterie du même côté, sens 3-2 et 2-3 vus sur 16 graines. Défauts attrapés
avant livraison : un gabarit de bossa jouait une quinte au temps 3 d'un accord qui change à la blanche (corrigé : `fifth_if_held`) ; deux mutations
d'afro-cubain n'échouaient d'abord rien (clave lue à une seule densité, kits comptés avec les balais de l'arc) et ont durci les tests ; le contrat
générique `time_plan_playback_tests` a révélé la courbe de tempo plate de quatre profils.

## Manques du schéma et du moteur (tâche A7 ; rien n'est contourné)

1. **Basse sans étiquette d'arc** et plan de motifs fixé par air : 20 à 49 % de mesures de basse changées d'un chorus au suivant.
2. **`passing` ne se restreint pas** à un changement d'accord à la barre : plus d'approches que WJazzD n'en mesure.
3. **Une boucle de grille ne dépend pas de la tonalité tirée** : le mode d'une variante de session n'est qu'une étiquette (les accords s'écrivent relatifs à la tonique).
4. **`two_feel` n'existe que sous `walking`** : la basse à deux appuis de la samba est écrite en gabarits.
5. **`[[fills]]` n'a pas de côté de clave** : les relances d'afro-cubain sont limitées à la queue des fills du rhumba (pas 13 à 15).
6. **Étiquettes de l'arc plus fortes que celles de la session** : une variante de clave rumba (troisième coup au pas 14) n'est pas tirable.
7. **`keep_steps` du stop-time ne garde pas l'anticipation du pas 14** de la mesure précédente (le tumbao joue le un ; le tumbao strict le tait).
8. **Un gabarit de basse ne suit pas un changement d'accord au temps 3** : les grilles de l'afro-cubain changent à la barre seulement.
9. **Pas de `tie` sur l'anticipation du pas 14** (bossa) : le moteur réattaque la note au temps 1.
10. **Micro-timing de voix borné à ±0,2 pas, sans dispersion** (pédale hors temps du GMD à +0,225).
11. **Une relance remplace toute la mesure** (l'ensemble se tait) ; pas d'héritage de `[arc]` dans l'arc nommé de session ; champ `intensity` publié sans l'arc.
12. **Groupes de session indépendants** : balais à 180 BPM possibles ; un `level_db` par piste pour plusieurs kits ; phrases, breaks et relances comptés en mesures de 4/4 (une mesure de samba = deux de 2/4).
13. **Rôles GM** : pas de surdo ni de tamborim dans le kit ; le triangle muet (80) est presque inaudible dans GeneralUser GS (−49,9 LUFS à vélocité 100, joué à 127).
14. **Courbe de tempo** : une courbe plate est refusée par le contrat générique de playback ; `tempo_offset` non écrêté aux bornes du profil.

## Limites

- Sources : un batteur pour la bossa et la samba, deux pour l'afro-cubain (un par fichier) et le baião ; kit électronique ; groupes « trop peu » pour la bossa et l'afro-cubain ;
  la moitié ternaire de l'afro-cubain n'est pas représentée.
- Clave, montuno, tumbao, percussions latines, comping, grilles, arc, habillage, mixage : règles de métier, **non écoutées** (D5).
- Tempos : trois plages séparées de plus de 8 BPM ne sont pas tenables avec les étendues mesurées de la samba, de l'afro-cubain et du baião.
- Basse : arrivée toujours sur la basse publiée (WJazzD : 57 %).

## Liens d'écoute (à ouvrir par l'orchestrateur ; deux graines contrastées par profil)

| Lien | Ce qui change |
|---|---|
| `harmonium://open?profile=gmd_latin_brazilian_bossa&seed=1` | 125 BPM, ré, guitare nylon, contrebasse, balais, voicing sans fondamentale, comping une octave plus bas, énergie douce, arc « orchestre » (balais, piano électrique au sommet, balais et nylon à la retombée) |
| `harmonium://open?profile=gmd_latin_brazilian_bossa&seed=2` | 110 BPM, do, piano électrique, fretless, kit room, notes guides, énergie pleine, arc standard |
| `harmonium://open?profile=gmd_latin_brazilian_samba&seed=7` | 91 BPM, sol, guitare nylon, balais, voicings *shell*, comping à l'octave inférieure, énergie douce |
| `harmonium://open?profile=gmd_latin_brazilian_samba&seed=6` | 114 BPM, do, piano Tine, kit standard, notes guides, arc « orchestre » (8 groupes sur 9 diffèrent de la graine 7) |
| `harmonium://open?profile=gmd_afrocuban&seed=5` | clave 3-2, 105 BPM, si bémol, I–IV–V7–I, piano électrique, basse fretless, kit room, sans fondamentale, doux |
| `harmonium://open?profile=gmd_afrocuban&seed=6` | clave 2-3, 128 BPM, do, vamp Im7–IV7, guitare nylon, contrebasse, balais puis kit room, voicings serrés, soutenu, arc « orchestre » (cuivres au sommet : `&bar=49`) |
| `harmonium://open?profile=gmd_latin_brazilian_baiao&seed=11` | 96 BPM, la mixolydien, piano en voicings ouverts une octave plus bas, basse aux doigts, kit room, arc « orchestre » (balais au chorus 1, accordéon au chorus 4) |
| `harmonium://open?profile=gmd_latin_brazilian_baiao&seed=13` | 110 BPM, si bémol mixolydien, accordéon en coquilles, contrebasse, kit jazz, arc continu |

Les graines viennent du CLI (`mesures_latin.py habillages`) ; les profils de ces liens sont distribués (`StyleProfile::distributed()`).

## Vérifications d'acceptation

| Vérification | Résultat |
|---|---|
| `cargo test --workspace -q --no-fail-fast` | **122 binaires, 1 044 réussis, 0 échec, 2 ignorés** (dont les quatre de contrat latin : 16 + 11 + 12 + 11 tests). Une exécution antérieure à la correction du tempo avait montré deux échecs (bornes de tempo de la bossa et du baião) |
| `time_plan_playback_tests` (contrat générique, tous les profils distribués) | passe après le décalage d'arousal de ± 4 BPM ; avant : `gmd_afrocuban` et `gmd_latin_brazilian_bossa` n'avaient que 2 tempos sous automation |
| `cargo test -p harmonium --features odin` ; `cargo test -p harmonium_bevy` | verts (0 échec) ; 11 tests Bevy |
| `cargo check -p harmonium_studio` | passe |
| `WASM_BINDGEN=/tmp/harmonium-019/wb/bin/wasm-bindgen harmonium_determinism/scripts/run.sh` | natif = wasm : **177 / 177** fichiers identiques (152 de `main` + 24 pour les quatre profils + la sonde `swing_fixtures.txt`) |
| profils existants | `git diff eecbd3f` hors des quatre profils, de leurs tests et de `b3/` : `profiles/README.md` et `plan.md` seulement ; aucun fichier de `src/`, `TUNING_VERSION` 26, goldens inchangés |
| `cargo +nightly fmt --all --check` | propre |
| clippy (`harmonium_core --tests`) | aucun avertissement dans les fichiers des quatre profils |
| `seek_to_bar_64_under_50ms` | passe aux trois essais ; code moteur inchangé, seuil inchangé (durée non imprimée par le test) |

Correction faite à l'intégration : les courbes de tempo des profils bossa, afro-cubain et baião étaient plates (`tempo_offset` nul). Le contrat générique exige qu'une commande d'émotion déplace
le tempo ; les trois profils portent maintenant `tempo_offset = [[0, -4], [0.5, 0], [1, 4]]` (comme la samba) et leurs tests de contrat attendent la plage de la variante ± 4 BPM, sans encadrer le
tempo par `min_bpm` / `max_bpm` (le moteur ne l'écrête pas). Les chiffres de tempo des fragments d'agents datent d'avant cette correction ; les tableaux de ce compte rendu sont refaits.

## Reproduire

```sh
cd ~/Project/bascanada/harmonium       # pour assets/
CLI=~/orca/workspaces/harmonium/b3-latin/target/release/harmonium-cli
python3 ~/orca/workspaces/harmonium/b3-latin/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py habillages --cli $CLI --profile <dossier> --seeds 16 --bars 48
python3 …/b3/mesures_latin.py basse --cli $CLI --profile <dossier> --seeds 8 --bars 48 \
    --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json
~/Project/bascanada/harmonium_lab/.venv/bin/harmonium-lab seed-variety --profile <dossier> --seeds 16 --chorus 4 \
    --engine-dir ~/Project/bascanada/harmonium --engine-bin $CLI --label B3 --work-dir /tmp/b3/var -o v.json
```

Générateurs de batterie, distance, mixage et variété de chaque profil : `specs/019-bibliotheque-accompagnements/b3/<id>_*.py` (commande en en-tête).
