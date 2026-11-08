# B1 — banque de sons plus riche (phase 020, vague 1)

**État : terminée (2026-11-07, Mac). Trois dossiers de banques prêts pour l'écoute de William ; aucun choix fait.** Branche
`wquintal/b1-banque-sons`. Ni profil ni code du moteur ne changent ; outils hors moteur dans [`b1-outils/`](b1-outils/) ;
mesures détaillées dans [`b1-mesures.md`](b1-mesures.md). Aucune banque, aucun échantillon dans le diff.

**Ce que cette étude ne dit pas.** L'agent n'écoute pas. Les chiffres disent ce qui se charge, ce que ça pèse, à quel niveau
ça joue ; ils ne disent pas si c'est meilleur. L1 n'avait fait entendre aucune différence entre banques entières à William ;
ici seuls les programmes faibles des 13 profils d'E1 sont remplacés, le reste de GeneralUser GS est conservé.

## 0. Trouvailles qui dépassent B1

1. **oxisynth retrouve l'échantillon d'une zone par son nom, premier trouvé** (`oxisynth/src/core/soundfont/instrument.rs`
   l. 119-127, fork `a8643ca`). Une banque assemblée dont deux échantillons de sources différentes portent le même nom joue,
   pour le second preset, les échantillons du premier. Cas mesuré dans l'assemblage de B1 : les guitares FreePats jazz (26)
   et claire (27) ont toutes deux un échantillon `C2_s1_01` ; avant correction, 26 et 27 rendaient **le même WAV octet pour
   octet**. Les LUFS ne le montrent pas (valeurs plausibles). `sf2build.py` rend maintenant les noms uniques
   (`Out._unique_name`) ; 67 noms dupliqués dans `b1-gu`, 74 dans `b1-msb` avant, 0 après. GeneralUser GS et MS Basic n'ont
   aucun doublon.
2. **`assets/harmonium_trad.sf2` est touché par le même défaut** (non corrigé : la banque trad doit rester identique,
   hors périmètre). Ses 8 échantillons s'appellent `01`..`04` pour la touche 36 et `01`..`04` pour la touche 37 ; ceux de la touche 37
   (`02` : 3 376 points, `03` : 6 611) retombent sur ceux de la touche 36 (11 025 points chacun), joués un demi-ton plus haut
   [INFERENCE depuis le code et l'en-tête ; non écouté]. À vérifier à l'oreille par l'orchestrateur ; correction possible : renommer
   les échantillons de la touche 37 dans la source de la banque trad (hors dépôt).
3. **MS Basic 2.0.0 brute écrête au mix** : crête du mix à +8,3 dBFS (bossa), +8,0 (baião), +7,2 (afro-cubain) avec les mixages
   actuels, calibrés sur GeneralUser GS (§ 4). Une banque différente impose de recalibrer le mixage.

## 1. Candidats

Critère : redistribution du **fichier d'échantillons** dans un jeu fermé et sur le web ; attribution permise ; pas de NC ni
d'« usage personnel » ; SA signalée. Licences lues dans le fichier de licence ou le README de **l'archive téléchargée**
(colonne « lu à la source ») ou, à défaut, sur la page du dépôt (« dépôt ») ; le catalogue
<https://sfzinstruments.github.io/> (consulté le 2026-11-07) n'est qu'un index. Les banques GM complètes écartées ont été relues
au texte en L1 (`l1-banques-soundfont.md` §§ 3 et 5, 2026-11-02) ; on ne les relit pas ici.

### Retenues (dans les banques d'écoute)

| Source | Licence (lu à la source) | Sert pour | Poids (PCM → SF3 q4) | Provenance et réserves |
|---|---|---|---|---|
| **GeneralUser GS 2.0.3** (réf.) | [GeneralUser GS License v2.0](https://raw.githubusercontent.com/mrbumpy409/GeneralUser-GS/main/documentation/LICENSE.txt) (L1) | base de `b1-gu`, kits non remplacés | 32 Mo → 7,6 Mo | Redistribution permise ; l'auteur ne garantit pas la provenance de tous les échantillons (L1 § 3). |
| **MS Basic 2.0.0** | MIT (L1 § 3) | `ms-basic`, base de `b1-msb` | 216 Mo → 26,8 Mo (q4), 40,1 Mo (q8) | Piano Splendid (domaine public selon l'auteur), le reste = échantillons FluidR3 (L1 § 7). Notices de copyright à livrer avec la banque ; binaire = miroir tiers, à figer par SHA-256. |
| **FreePats**, 10 SF2 : orgues Drawbar / Percussive / Rock (16-18), guitare nylon (24), FSBS jazz (26), clean (27), dist2 (29-30), basses Finger / Picked YR (33-34), sax ténor (66) | **CC0** dans le `readme`/`LICENSE.txt` de chaque archive | programmes 16-18, 24, 26, 27, 29, 30, 33, 34, 66 | 7 Mo à 393 Mo (dist2) → 0,3 à 30,2 Mo | Orgues : enregistrements de la sortie de setBfree (logiciel GPL ; pas son code). Sax : échantillons VCSL CC0. Plage de touches élargie à 0-127 (extrémités transposées). |
| **Greg Sullivan E-Pianos** : Wurlitzer EP200 (→ prog. 4), Hohner Pianet T (→ prog. 7) | **CC BY 3.0**, attribution requise (`README.md` de l'arbre, rév. `8c3e581`) | 4 et 7 | 21,5 Mo source → 0,65 et 1,41 Mo | **Pas un Rhodes, pas un clavinet.** Le Pianet est un piano à anches (timbre clair, attaque nette) : substitut pour le funk, à juger. Plages de touches : 33-96 et 29-88 (les accords ne descendent pas sous la touche 48). |
| **Karoryfer Sneakybass** (→ prog. 32) | **CC0** (`LICENSE`, rév. `cc1a67e`) | contrebasse pizzicato 32 | 419 Mo source → 0,83 Mo | Contrebasse Otto Rubner 1958, cordes Spirocore, jouée « très doucement » : crête à 0,94 pour −5,3 LU sous GeneralUser, non montable sans écrêter (§ 4). |
| **Karoryfer Swirly Drums** (→ kits 0, 8, 16, 32, 40) | **CC0** (`license`, rév. `c40dafe`) | kick, caisse claire, charleston, toms ; balais (« stir » sur 38/40 du kit 40) | 1,83 Go source → 4,6 Mo pour les 5 kits | Un seul tour de round-robin ; articulations et keyswitches abandonnés (non représentables en SF2) ; échantillons normalisés. Les 5 kits partagent les mêmes échantillons principaux. |
| **VCSL** (`sgossner/VCSL`, rév. `c1ea7bc`) : cowbell, bongos, congas, agogo, shakers, claves, guiro | **CC0** (`LICENSE` à la racine) | touches latines GM 56, 60-64, 67-68, 70, 73-75, 82 des 5 kits | 27 Mo sélectionnés | Pas de timbales, cuíca, surdo (touches GM de la base conservées). |

### Examinées et écartées ou non intégrées

| Source | Licence | Décision |
|---|---|---|
| FluidR3 GM seule | MIT au README, mais « DO NOT REDISTRIBUTE » dans le champ commentaire du SF2 (L1 § 3) | Écartée : MS Basic a le même contenu hors piano. |
| MuseScore General 0.2 | MIT (L1) | Même contenu qu'MS Basic hors piano ; sans bénéfice à l'écoute en L1. |
| Salamander Grand Piano | CC BY 3.0 | Piano non faible ; 99 Mo en SF3, chargement 16-25 s (L1 § 6). |
| Arachno ; Timbres of Heaven ; SGM ; Musyng Kite | non commercial ; permission écrite exigée ; droits contradictoires ; provenance non vérifiable (L1 § 5) | Écartées. |
| Sonivox EAS | Apache-2.0 pour le code | Table d'ondes embarquée, pas une banque SF2/SF3 chargeable dans oxisynth. |
| VSCO 2 CE (CC0) | CC0 | Orchestral (cuivres classiques), ~3 Go SFZ ; pas de stabs de jazz ni de section funk. Cuivres de la base conservés. |
| **jRhodes3d / jRhodes3c** | **CC BY-NC(-SA)** (dépôt, catalogue) | Écartées : NC. **Aucun vrai Rhodes libre de redistribution trouvé.** |
| MuldjordKit ; DRS Kit ; Naked Drums | CC BY 4.0 (catalogue) | Pas de kit à balais ; MuldjordKit hors placement GM (touches 48-66), 209 Mo. Non intégrés. |
| Salamander Drumkit ; AVL Drumkits ; Sam's Sonor | CC BY-SA 3.0 / 3.0 / 4.0 | Écartées : SA sur une banque dérivée livrée fermée. |
| **SM Drums** | « Public Domain », 2,2 Go+ (catalogue ; licence non relue à l'archive) | Non intégré : poids ; à relire si Swirly déçoit. |
| Virtuosity Drums ; Big Rusty ; Unruly ; Frankensnare | CC0 (dépôt, GitHub API) | Libres, 0,3 à 2,3 Go ; pas de balais ; non intégrés. Piste si le kit de Swirly déçoit. |
| **Karoryfer Black and Green Guitars, Shinyguitar, Emilyguitar** | CC0 (dépôt, GitHub API) | **Piste non explorée pour la guitare de jazz 26** : deux guitares hollowbody (Black and Green) et une archtop (Shinyguitar), 98 à 443 Mo de SFZ. FreePats FSBS « jazz » est une Fender de base enregistrée en direct puis passée dans un ampli et un rack d'effets (README de l'archive) : pas une guitare creuse. À monter si William trouve la 26 de `b1-gu` insuffisante. |
| Karoryfer Fashionbass, Pastabass, Growlybass, Swagbass, Black and Blue Basses | CC0 (dépôt, GitHub API) | Basses électriques ; Black and Blue testée en B1-ovh puis retirée (niveau disproportionné). Non intégrées. |
| Karoryfer Bear Sax, Weresax (CC0) ; MTG Solo Sax (CC BY 4.0) | catalogue / dépôt | Alternatives au sax FreePats ; non intégrées. |
| Accurate-Salamander ; Headroom Piano ; LivingRoom Upright | CC BY ; CC BY 4.0 ; CC BY-SA 4.0 (catalogue) | Piano non faible ; non intégrés. |
| Kits ripés de claviers (Yamaha, Roland, Korg), Mike77154, Studio Rhodes LA, « Rhodes Electric Piano » (Pianoteq) | copyright, NC, licences floues (L1 § 5) | Écartés. |

**Pas de solution libre pour** : Rhodes réel, clavinet réel, guitare muted (28), slap (36), fretless (35), cuivres de section
(61), timbales, cuíca, surdo, pandeiro. Ces programmes restent ceux de la banque de base : **ne pas les présenter comme résolus**.

## 2. Chargement et taille

Toutes les banques se chargent dans oxisynth (SF2 et SF3 q4), **rendus hors ligne sans erreur** (`harmonium-cli --export`, code 0,
aucune ligne d'avertissement) : 13 profils × (mix + 4 pistes en solo) en SF2 pour chaque candidate, 13 mix en SF3 q4 pour
chacune ; les notes MIDI sont identiques entre banques sur les 65 rendus (SHA-256). Le mix SF3 q4 s'écarte du mix SF2 de 0,1 LU au plus
(0,2 pour `ms-basic`). Conversion : `harmonium_studio/scripts/sf2_to_sf3.py` (qualité 4 du studio ; qualité 8 en plus pour comparer) ;
celle de GeneralUser GS redonne le SHA-256 de `web-resources.json` (`290b0081…`).

| Banque | SF2 (octets) | SF3 q4 (octets) | SF3 q8 (octets) | SF3 q4 + trad (octets) | Décodage SF3 q4 (rendu 2 mesures, Mac) |
|---|---:|---:|---:|---:|---:|
| GeneralUser GS (réf.) | 32 319 396 | 7 620 750 | — | 7 774 936 | 1,9 s |
| `ms-basic` | 215 641 790 | 26 826 196 | 40 073 962 | 26 980 382 | 5,6 s |
| `b1-gu` | 607 751 490 | **53 881 154** | 87 287 974 | 54 035 340 | **12,1 s** |
| `b1-msb` | 786 710 192 | **72 884 902** | 116 309 530 | 73 039 088 | **15,6 s** |
| (variante de mesure) `b1-gu` sans 29 et 30 | 215 062 760 | 23 602 178 | — | 23 756 364 | 6,4 s |

La banque trad (`harmonium_trad.sf2`, 154 186 octets) est téléchargée en plus. Le web télécharge **et décode** la banque : coûts
mesurés avec le CLI natif ; le décodage en wasm est plus lent [INFERENCE, non mesuré].

**Où va le poids de `b1-gu`** (SF3 q4, octets du flux Vorbis des échantillons du preset, mesurés dans l'en-tête) : **guitare saturée
dist2 (29 et 30, même échantillons) 30,2 Mo sur 53,9** ; sax 3,2 ; kits Swirly + VCSL 4,6 pour les cinq ; nylon 1,6 ; Pianet 1,4 ;
orgue rock 1,3 ; reste de chaque preset < 1 Mo. Le piano de MS Basic ajoute 10,1 Mo à `b1-msb`. **La question de poids est donc
celle de dist2** : sans 29 et 30, `b1-gu` pèse 23,6 Mo (décodage 6,4 s) ; avec, 53,9 Mo. Les programmes 29 et 30 ne jouent que
dans `gmd_rock` et `gmd_blues_shuffle` de la liste E1.

## 3. Construction des banques (reproductible)

Base + presets importés + kits recomposés, écrits par `b1-outils/assemble.py` (`sf2build.py` : fusionneur PCM 16 bits de l'annexe L1,
avec zones SFZ converties par `sfzlib.py`). Python avec `numpy scipy soundfile` (venv du labo), `ffmpeg`, `oggenc`.

| Programme (GM, 0-based) | Source | Niveau (gain appliqué pour ramener à GeneralUser GS) |
|---|---|---|
| 4 (EP) | Wurlitzer EP200 (SFZ, 48 zones) | −2,6 dB |
| 7 (clavinet) | Hohner Pianet T (33 zones) | 0 |
| 16, 17, 18 (orgues) | FreePats Drawbar / Percussive / Rock | 0 ; −2,1 ; −1,6 |
| 24 (nylon) ; 26 (jazz) ; 27 (claire) | FreePats Spanish Classical ; FSBS jazz ; FSBS clean (bridge small) | −2,3 ; −1,4 ; −6,1 |
| 29, 30 (overdrive, distorsion) | FreePats FSBS dist2 (un seul preset d'échantillons pour les deux) | −5,0 |
| 32 (contrebasse) | Sneakybass pizzicato (40 zones) | 0 (voir § 4) |
| 33, 34 (basse doigts, médiator) | FreePats Finger / Picked Bass YR | −1,6 ; −1,4 |
| 66 (sax ténor) | FreePats Tenor Saxophone | −8,9 dB |
| kits 0, 8, 16, 32, 40 (128:n) | Swirly (kick 35-36, caisse claire 38/40, charleston 42/44/46, toms 41-50, cymbales 49/51/53 ; balais « stir » 38/40 du kit 40) + VCSL (touches latines) sur le reste du kit de la base (GeneralUser même dans `b1-msb`) | voir § 4 |
| **inchangés** : 0, 3, 21, 25, 28, 35, 36, 38, 61, kits 24 et 25 | banque de base | — |

Les kits **électroniques 24 et 25 restent ceux de la banque de base** : les remplacer par un kit acoustique aurait supprimé le
timbre voulu de `gmd_funk`, `gmd_reggae` et `gmd_pop`. Les gains viennent de `palette.py` (un programme à la fois, mêmes notes,
brute puis finale ; `b1-mesures.md`) ; seuls les **abaissements** sont appliqués (monter un échantillon PCM 16 bits déjà à pleine
crête l'écrêterait). `b1-gu` = base GeneralUser GS ; `b1-msb` = base MS Basic (piano de concert 144 échantillons, accordéon,
acier, muted, fretless, slap, synth, section, kits 24 et 25 de MS Basic) avec les mêmes presets importés.

```sh
R=/tmp/h020-b1; mkdir -p $R/{dl,sf2,src,merged,sf3,render}   # sources hors dépôt ; ~2,5 Go
# FreePats : 10 archives SF2 (URLs : pages d'instruments du § 1), bsdtar -xf dans $R/sf2/<archive>/, un seul .sf2 par dossier
# GitHub, révisions figées : GregSullivan.E-Pianos 8c3e581, karoryfer.sneakybass cc1a67e, karoryfer.swirly-drums c40dafe
#   (https://codeload.github.com/sfzinstruments/<dépôt>/zip/<rév>) dans $R/src/<dépôt>/ ;
# VCSL c1ea7bc : git clone --filter=blob:none --no-checkout, puis `git checkout c1ea7bc -- <fichiers .wav des dossiers
#   Agogo Bells, Bongos, Conga, Cowbells, Guiro, Claves, Shaker Small/Large>` copiés dans $R/src/vcsl/ (+ LICENSE)
curl -sSL -o $R/dl/MS_Basic-v2.0.0.sf2 https://github.com/Meowchestra/SoundFonts/releases/download/v2.0.0/MS_Basic-v2.0.0.sf2
cd specs/020-qualite-des-styles/b1-outils; export H020_B1_ROOT=$R H020_B1_GU=$R/dl/generaluser.sf2 H020_B1_REPO=<worktree>
PY=~/Project/bascanada/harmonium_lab/.venv/bin/python
$PY assemble.py gu; $PY assemble.py msb                # $R/merged/b1-{gu,msb}.sf2 (déterministe : même SHA-256 à la reconstruction)
$PY seeds.py 40 && $PY palette.py … && $PY measure.py b1-gu $R/merged/b1-gu.sf2 && $PY report.py   # mesures et liens
python3 manifest.py                                    # empreintes
python3 harmonium_studio/scripts/sf2_to_sf3.py $R/merged/b1-gu.sf2 $R/sf3/b1-gu-q4.sf3 4
```

### Manifeste (principe VII)

Tous les fichiers sont hors dépôt ; ces empreintes valent pour **ces fichiers précis** (reconstruits ici, sur ce Mac ; elles diffèrent de
celles du premier essai sur `ovh-inner`, dont les banques ont été refaites). Arbres de fichiers : SHA-256 des couples
`chemin UTF-8 + NUL + SHA-256 hexadécimal du fichier`, chemins triés (authentifie aussi README et licences).

| Archive FreePats (CC0), SF2 extrait | SHA-256 |
|---|---|
| `DrawbarOrganEmulation-20190712.sf2` | `200b7d762fb1d4d7dbee34760a768319c7548e94b1a6ba76108f0702888b33ce` |
| `PercussiveOrganEmulation-20190715.sf2` | `468bd7a250849002afc28e28d11579e62b26a291e64724047d7880d3b149e9ab` |
| `RockOrganEmulation-20190715.sf2` | `994f79822699ca17174641864583ec8d32042c188ec85dcc90c28d29830eddf9` |
| `SpanishClassicalGuitar-20190618.sf2` | `0d4c08ea8c1c8924b3829084b0c8778f6d6b906f331e44fde51bc159f76b03b9` |
| `EGuitarFSBS-jazz bridge small 20260807.sf2` | `1baca0ebb66627bbd4c08d4137c41728253588ce85504ed5148f06ea7c000d22` |
| `EGuitarFSBS-clean bridge small 20260807.sf2` | `a58803acd8640a06a0a1a277a16ff4cd2610d9eab5fd6793741db4730c12f594` |
| `EGuitarFSBS-dist2 bridge 20220911.sf2` | `2859f619d97de590fe94bf57102b741eacefd2782de22a88b7e26a4de512d488` |
| `FingerBassYR 20190930.sf2` | `bd1332696f61c0cb254ce7f202fb6b945ff46ccb1dcdd62e345a50bde3afb165` |
| `PickedBassYR 20190930.sf2` | `b16f6666486f42c6c4bde19a62462f0cda8b0f1d0ce72ad8ec51a557118d03ae` |
| `TenorSaxophone-20200717.sf2` | `3b4f4cdabe5c9a9825cd0dd1469a35117b86876256a089bba5f97d4abf1e9b46` |

| Arbre GitHub (révision) | Fichiers ; octets | SHA-256 de l'arbre |
|---|---|---|
| `sfzinstruments/GregSullivan.E-Pianos` (`8c3e581acda3594b553948ff0222d4f84a698376`), CC BY 3.0 | 178 ; 21 484 396 | `d9b2f9a006067ab1959948b3be936a7d99ed4c4488aff34cf24457a72485e32d` |
| `sfzinstruments/karoryfer.sneakybass` (`cc1a67e5c913678b85eaad1a3b5cb180d3be3e2d`), CC0 | 715 ; 418 713 231 | `f8a60cd27c662f05d314803839b4542efa143d6e32bcab3a5c80e4fa18ba9dc3` |
| `sfzinstruments/karoryfer.swirly-drums` (`c40dafe0011cb2e54c0c220ff0fa308a11fc60f5`), CC0 | 5 103 ; 1 826 949 043 | `dd1c0ab519d4608bec527c8aba60c5e3d6d80a9abdcca38f1b6ecc55db00dac6` |
| `sgossner/VCSL` (`c1ea7bcc3c7309650ab0da9d15c9cd1fbc4a4c7e`), CC0, sous-arbre des 8 familles + `LICENSE` | 139 ; 27 438 088 | `fb5c5800c540ae6ba4867c35fb3083493f39dcbd9726cdb6d9afc1762cd082ac` |

| Fichier | Octets | SHA-256 |
|---|---:|---|
| GeneralUser GS 2.0.3 (= `assets/`) | 32 319 396 | `9575028c7a1f589f5770fccc8cff2734566af40cd26ed836944e9a5152688cfe` |
| `harmonium_trad.sf2` (identique dans chaque dossier) | 154 186 | `4dc8159837b44c754d3bac9eb645e6fb85e53983d9e52a551f8256b67ef28443` |
| MS Basic 2.0.0 SF2 (`ms-basic/GeneralUser-GS.sf2`) | 215 641 790 | `f4ed62be6986e70438c995c03fb457610c75d01976bdc3248da9d0cc43a5613c` |
| `b1-gu` SF2 | 607 751 490 | `6b0fc769308d81c1ef0d14845f954b14720a39fe64840a7073932a6e8d559b4c` |
| `b1-msb` SF2 | 786 710 192 | `79e0f2fc80c7d36928c875687a57f14277147a512cd2ad750c5116aebc86e5b4` |

SF3 q4 complets : `ms-basic` `59ef8e01fec5a5e27f3915014df0a4d5bb50f62c17d62bfaa91c7384c4bcb594` ; `b1-gu`
`b367c389c4c4f6f40c04c50616359e9cda0102948fe01e2e2147252ab1e725bf` ; `b1-msb`
`5cadf34e3a72c451a562050ce6b1fc9bdc8a0ea8cb0683e3876486990e36ec91`. Les SF3 ne sont pas dans les dossiers d'écoute (le
studio lit le SF2 local ; le SF3 chiffre le futur téléchargement).

**Licences à livrer avec une banque retenue** : texte MIT et notices de MS Basic (`ms-basic`, `b1-msb`) ou licence GeneralUser GS
(`b1-gu`) ; **crédit CC BY 3.0 : Greg Sullivan** (Wurlitzer, Pianet) ; les modules CC0 n'imposent rien. Aucun avis juridique :
lecture des textes de licence.

## 4. Niveaux : mesures et corrections de mix proposées (non appliquées)

Le mixage de chaque profil est calibré sur GeneralUser GS. LUFS par piste en solo et écarts à GeneralUser GS, mêmes notes
(MIDI identiques), 64 mesures à la graine du lien d'écoute, méthode de L1 § 8 : **tableaux complets par profil dans
[`b1-mesures.md`](b1-mesures.md)** (LUFS, écarts, crêtes, nouveau `level_db` proposé pour `Chords`, `Bass`, `PercussionLow`,
`PercussionHigh`, par banque).

Écart au mix complet (LU, candidate − GeneralUser GS) et crête, sur les 13 profils :

| Banque | mix | accords | basse | perc. grave | perc. aiguë | pire crête du mix |
|---|---|---|---|---|---|---|
| `ms-basic` | **+0,6 à +7,7** | −3,1 à +8,1 | −0,9 à +4,7 | **+2,4 à +8,4** | −3,9 à **+14,8** | **+8,3 dBFS** (bossa) ; 5 profils au-dessus de 0 dBFS, 9 au-dessus de −1 |
| `b1-gu` | −3,3 à +2,2 | −2,0 à +3,5 | **−5,0 à +1,2** | **−10,1 à 0** | −10,6 à +4,4 | −1,1 dBFS (bossa) |
| `b1-msb` | −3,5 à +2,2 | −2,4 à +3,5 | −5,0 à +1,2 | −10,1 à +2,7 | −10,6 à +4,4 | −1,1 dBFS (bossa) |

GeneralUser GS lui-même atteint +0,4 dBFS (funk, afro-cubain) avec ces mixages.

Lecture :
- **`ms-basic` brute n'est pas écoutable aux niveaux actuels** : ride et charleston jusqu'à +14,8 LU, grosse caisse jusqu'à
  +8,4, écrêtage (5 profils). Il faudrait rabaisser la batterie jusqu'à environ 13 dB (tableau « Correction de mix proposée »
  de l'annexe). C'est le contrôle « autre banque GM », pas un candidat de production.
- **`b1-gu` et `b1-msb`** : chaque preset importé est ramené à GeneralUser GS en palette (accords : écart ≤ 1,1 LU, clavinet −1,1 ;
  `palette.py`). **La palette n'est pas le profil** : au mix d'un profil les accords restent jusqu'à +3,5 LU (bossa) au-dessus de GeneralUser,
  car le registre et les voicings du profil ne sont pas ceux du balayage. Restent aussi : **contrebasse 32 : −5,3 LU** en palette
  (Sneakybass, non montable sans écrêter ; −1,7 à −5,0 au mix des profils où elle joue) ; **grosse caisse et caisse claire de Swirly :
  −4 à −10 LU** (0 pour `gmd_reggae`, dont le kit 24 est inchangé) ; ride et charleston de +4,4 (bossa, swing) à −10,6 (soul)
  selon le motif du profil. Proposition, profil par profil (annexe) : `Bass` +1,7 à +5 dB là où joue la contrebasse,
  `PercussionLow` +4 à +10 dB, `PercussionHigh` de −4 à +10 dB ; vérifier la crête après.
- `gmd_reggae` : `b1-gu` joue le kit 24 inchangé (écart 0,0) ; `b1-msb` utilise le kit 24 de MS Basic (+2,7 / +2,2).
- **La palette sur `wjazzd_swing` ne dit pas tout du mix par profil** : l'habillage tire des programmes différents selon la graine,
  donc un même profil joue plusieurs programmes ; les corrections par profil de l'annexe valent pour la graine du lien.

## 5. Prêt pour l'écoute de William (D7)

Dossiers (SF2 hors dépôt, nom `GeneralUser-GS.sf2`, plus le même `harmonium_trad.sf2`) :

| Dossier `~/orca/harmonium-020/b1/` | Contenu |
|---|---|
| `gu/` | **Référence** : GeneralUser GS d'origine (SHA-256 `9575028c…` = `assets/`). |
| `ms-basic/` | MS Basic 2.0.0 intact (contrôle). **Baisser la batterie** pour l'écouter (§ 4). |
| `b1-gu/` | GeneralUser GS + programmes FreePats / Wurlitzer / Pianet / Sneakybass + kits Swirly et percussions VCSL. **Candidate principale.** |
| `b1-msb/` | Idem sur base MS Basic : ajoute surtout le piano de concert, accordéon, acier, section. |

Ouvrir un dossier à la fois, **les mêmes liens dans le même ordre** pour `gu/` puis chaque candidate. Graine choisie par
`b1-outils/seeds.py` : celle (de 1 à 40) dont l'habillage expose le plus de couples (rôle, programme) concernés par B1 sur les
64 premières mesures ; « mesure n » = première mesure où le programme joue. Les graines dépendent de l'habillage de `main` à
`9937b3c` : si Q0 ou S1 le changent, **refaire `seeds.py`** avant d'écouter. Le studio affiche les mesures ; si le tempo
de la graine le permet, écouter d'abord la mesure 1 puis les mesures citées.

| Lien | Programmes exposés (mesure n) | Ce qu'il faut écouter |
|---|---|---|
| [wjazzd_swing](harmonium://open?profile=wjazzd_swing&seed=32) | orgue perc. 17, contrebasse 32, kit balais 40 (1) ; guitare jazz 26, kit jazz 32 (45) | Contrebasse de Sneakybass contre celle de GeneralUser (4 échantillons) : attaque, longueur de note, présence en bas ; caisse claire aux balais (« stir » de Swirly, mesure 1) puis ride et caisse claire du kit jazz (mesure 45) ; guitare 26 : timbre de comping. |
| [wjazzd_ballad](harmonium://open?profile=wjazzd_ballad&seed=1) | orgue 16, contrebasse 32, balais 40 (1) ; kit jazz 32 (33) | Orgue drawbar FreePats (setBfree) contre celui de GeneralUser ; balais sur tempo lent ; contrebasse. |
| [gmd_jazz_waltz](harmonium://open?profile=gmd_jazz_waltz&seed=2) | orgue 16, basse fretless 35 (inchangée), balais 40 (1) ; kit jazz 32 (33) | Orgue et balais ; la basse 35 n'est **pas** remplacée (témoin). |
| [gmd_latin_brazilian_bossa](harmonium://open?profile=gmd_latin_brazilian_bossa&seed=24) | guitare jazz 26, contrebasse 32, balais 40 (1) ; kit jazz 32 (17) ; **Wurlitzer 4 (49)** ; **nylon 24 (65)** | Nylon FreePats contre GeneralUser ; Wurlitzer ; percussions latines VCSL (cloche, bongo, conga, claves). |
| [gmd_latin_brazilian_samba](harmonium://open?profile=gmd_latin_brazilian_samba&seed=6) | nylon 24, contrebasse 32, balais 40 (1) ; Wurlitzer 4, kit standard 0 (17) ; acier 25 (inchangé), kit power 16 (49) | Énergie des accents (batterie Swirly −7 à −9 LU sous GeneralUser : le motif sera plus faible) ; manque de surdo et de cuíca (non résolu). |
| [gmd_latin_brazilian_baiao](harmonium://open?profile=gmd_latin_brazilian_baiao&seed=7) | nylon 24, contrebasse 32, balais 40 (1) ; kit room 8 (17) | Poids de la grosse caisse (−6 LU) ; cohérence avec les syncopes. |
| [gmd_afrocuban](harmonium://open?profile=gmd_afrocuban&seed=6) | nylon 24, contrebasse 32, balais 40 (1) ; kit room 8 (17) ; **section de cuivres 61 (49)** | Section 61 **non remplacée** (limite) ; claves, bongos, congas de VCSL sans masque. |
| [gmd_funk](harmonium://open?profile=gmd_funk&seed=1) | muted 28 (inchangé), basse doigts 33, kit power 16 (1) ; **Pianet 7, slap 36 (inchangé) (49)** ; **guitare claire 27 (65)** | **Pianet à la place du clavinet** : l'attaque funk y est-elle ? Basse doigts FreePats ; guitare claire FSBS contre GeneralUser ; kit power de Swirly. |
| [gmd_soul](harmonium://open?profile=gmd_soul&seed=1) | **Pianet 7**, basse doigts 33, kit room 8 (1) ; **Wurlitzer 4 (49)** | Wurlitzer EP200 contre « Tine EP » : le seul son proche d'un piano électrique de soul ; basse 33. |
| [gmd_reggae](harmonium://open?profile=gmd_reggae&seed=7) | muted 28 (inchangé), basse 33, kit électronique 24 (inchangé) (1) ; guitare claire 27 (17) | Contretemps secs 28 puis 27 ; basse 33 YR. Le kit 24 est celui de la base. |
| [gmd_rock](harmonium://open?profile=gmd_rock&seed=2) | guitare claire 27, basse médiator 34, kit standard 0 (1) ; **orgue rock 18, kit room 8 (17)** | Basse au médiator ; orgue rock ; kit Swirly standard puis room. |
| [gmd_pop](harmonium://open?profile=gmd_pop&seed=3) | guitare claire 27, basse 33, kit standard 0 (1) | Peu de programmes exposés (score 3 sur les 40 graines) : sert de témoin pour le changement de basse et de batterie. |
| [gmd_blues_shuffle](harmonium://open?profile=gmd_blues_shuffle&seed=7) | contrebasse 32, kit standard 0 (1) ; orgue 16 (13) ; guitare claire 27 (25) ; **overdrive 29, basse médiator 34 (37)** | Guitare saturée dist2 contre GeneralUser (celle qui pèse 30 Mo : vaut-elle son poids ?) ; orgue 16 ; contrebasse 32. |

La mesure 1 d'un lien est celle de l'habillage tiré ; les programmes « (n) » apparaissent à la mesure n du premier air
(`--record-truth`, 64 mesures) et peuvent revenir ensuite selon l'arc.

**À écouter dans chaque lien, dans cet ordre** : (1) la basse (32/33/34) ; (2) la batterie (kick, caisse claire, charleston) ;
(3) les accords (guitare, orgue, EP) ; (4) les percussions latines. Comparer `gu/` à `b1-gu/` d'abord ; `b1-msb/` ne diffère de `b1-gu/`
que par le piano (0), l'accordéon, l'acier, le muted, le slap, le fretless et la section ; `ms-basic/` d'abord avec la batterie
rabaissée.

## 6. Recommandation

1. **Écouter `b1-gu` contre `gu`**. C'est la seule des trois qui isole le changement utile (basse 32, guitares, orgues, sax,
   Wurlitzer, kits) sur une base GeneralUser GS qui avait la préférence de William en L1. Les points où un gain est attendu :
   contrebasse 32, nylon 24, balais 40, orgue 16-18 ; les points où il n'y en a **aucun** : cuivres 61, muted 28, slap, fretless,
   timbales, cuíca, surdo.
2. **`b1-msb`** : à n'écouter que si le piano de concert compte (L1 n'avait pas fait entendre de gain) ; coût +19 Mo SF3.
3. **`ms-basic`** : contrôle, pas de gain attendu hors piano (L1 : 100 % des échantillons de FluidR3). Ne pas retenir.
4. **Poids** : `b1-gu` coûte 53,9 Mo de SF3 et 12 s de décodage (contre 7,6 Mo et 1,9 s) **dont 30,2 Mo et environ 6 s pour la
   seule guitare saturée**. Si William retient `b1-gu` mais pas dist2 (rock et blues seulement), la variante sans 29 et 30 pèse
   23,6 Mo. Décision après l'écoute des liens `gmd_rock` et `gmd_blues_shuffle`.
5. **Avant tout changement de banque** : recalibrer `Bass` et `PercussionLow` des profils (tableau de l'annexe), ou ajouter un
   étalonnage de banque pour la contrebasse et les kits (impossible en PCM 16 bits sans compresseur ; piste : gain par preset
   avec `initialAttenuation` négatif si oxisynth le tolère, non testé).
6. **Pistes si le résultat déçoit** : guitare de jazz archtop (Shinyguitar, Black and Green Guitars, CC0), autre kit à balais
   (SM Drums, domaine public selon le catalogue), vrai clavinet (aucun libre trouvé).

## Reprise sur `ovh-inner`

Le premier essai (commit `wip(020/B1)`) avait les mêmes outils et les mêmes tailles de banque ; il est **refait ici** et corrigé sur
quatre points : noms d'échantillons uniques (§ 0), kits 24 et 25 conservés, gains de niveau par preset, graines choisies par
exposition (au lieu de la graine 7 fixe, 8 mesures). Ses empreintes de banque et le jeu de 354 fichiers VCSL (aujourd'hui 139) ne
sont pas reproductibles à l'identique.
