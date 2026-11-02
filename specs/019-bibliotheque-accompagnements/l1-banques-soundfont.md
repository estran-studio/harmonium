# L1 — Banques SoundFont pour training (phase 019, vague 1)

**État : terminée. Choix du propriétaire (2026-11-02) : option C, GeneralUser GS conservée** (aucune
différence entendue entre A, B et C ; sons de GeneralUser GS préférés) ; rien des §9 n'est appliqué. Toutes les licences ont été relues le
**2026-11-02** (date de consultation de chaque lien ci-dessous, sauf mention contraire). Branche
`wquintal/l1-banques-sf2`, documents seulement : ni code, ni profil, ni banque dans le diff.

Écoute : <http://192.168.8.171:8080/#phase-019-l1> (25 MP3 320 k, `harmonium_lab/corpus/ecoute_019/l1/`,
gitignoré). Outils et commandes : section 9 et `l1-annexe-outils.md`.

**Ce que cette étude ne dit pas.** L'agent n'écoute pas : les rendus, les moments à écouter et les
chiffres de mixage sont livrés pour **ton** oreille. Les mesures ci-dessous (échantillons, tailles, niveaux,
chargement, provenance) disent ce qui existe, ce qui sonne et ce qui coûte ; elles ne disent pas quelle banque
sonne le mieux.

## 1. Recommandation (à trancher après écoute)

| Option | Contenu | Licence | SF3 | Quand la choisir |
|---|---|---|---|---|
| **A (recommandée)** | **MS Basic 2.0.0** seule (piano de concert 144 échantillons, kits jazz et balais, toutes les touches latines GM) | MIT | 40 Mo (qualité 8) | Si le piano et la provenance documentée comptent plus que le poids : c'est la seule banque GM complète où le piano est nettement plus riche que celui de GeneralUser GS (§4). |
| B | A + modules **CC0** de FreePats fusionnés hors ligne (orgues, guitares nylon et jazz, basse YR, sax ténor, piano droit, kit « World Percussion ») | MIT + CC0 | 65 Mo (q. 8) / 41 Mo (q. 4) | Si l'écoute de L1-c et L1-d justifie ces timbres ; coûte une étape de fusion dans la construction des ressources (§8). |
| C | Statu quo : **GeneralUser GS 2.0.3** | GeneralUser GS License v2.0 | 10,6 Mo (q. 8) / 7,6 Mo (q. 4, actuel) | Si l'écoute ne montre pas de gain : zéro travail, 4 à 5 fois plus léger, mais piano à 17 échantillons et licence qui reconnaît une provenance non garantie (§3). |

Sont **déconseillés** : FluidR3 seul (même contenu que MS Basic, moins le piano, notice interne contradictoire :
§3), Salamander (CC BY 3.0, 99 Mo en SF3 q. 4, chargement 16 à 25 s : §6), toute banque de la section 5.

Constat qui pèse sur le choix : **hors piano, MS Basic = FluidR3** (100 % des échantillons identiques pour
orgue, guitares, basses, cuivres, sax, kits jazz et balais ; mesure SHA-1, §7). GeneralUser GS partage 0 %
de ses échantillons avec FluidR3 sur les 25 programmes comparés. Passer à MS Basic ne change donc pas les timbres hors piano de façon
« plus riche » ; cela change le piano, le poids, le mixage (§8) et la chaîne de droits.

## 2. Ce qui a été fait

1. Recensement : banques complètes GM (GeneralUser GS, MuseScore General, MS Basic, FluidR3, Timbres of Heaven,
   SGM, Arachno, TimGM6mb, Sonatina), banques par instrument (FreePats : piano, orgue, guitares, basse, sax,
   percussion, kit ; Salamander ; VSCO 2 CE, VCSL, Karoryfer), et ce qui traîne sur Musical Artifacts.
2. Licence relue au texte pour chaque banque (§3, §5) ; provenance des échantillons quand la licence est
   ambiguë.
3. Chargement dans le synthé du moteur (fork oxisynth) : `harmonium-cli --export --soundfont` depuis
   `~/Project/bascanada/harmonium`, SF2 et SF3 (§6).
4. Conversion SF3 avec `harmonium_studio/scripts/sf2_to_sf3.py` (qualité 4 du studio et qualité 8).
5. Balayage de 34 programmes ou kits par banque avec le moteur (palette, §7).
6. Rendus d'écoute : 25 mix (5 styles × banques) et 105 solos par piste, LUFS, MP3 (§8).

Moteur utilisé : `harmonium-cli 0.1.0` (`target/release`, compilé le 2026-11-02 à 15:14 dans
`~/Project/bascanada/harmonium`, dépôt à `fa9c630`). Graine 7.

## 3. Banques retenues pour l'écoute : licence, taille, chargement

« Fermé » = produit distribué fermé (training et jeux) qui embarque ou télécharge la banque.
Chargement : `harmonium-cli --export --bars 8 --style wjazzd_swing --soundfont <banque>`, code retour 0,
aucun avertissement hors banques mono-instrument ; durées de la machine au repos (ordre de grandeur,
la machine était partagée avec d'autres agents).

| Banque | URL | Licence (lien, 2026-11-02) | SF2 (octets) | SF3 q. 4 / q. 8 (octets) | Usage fermé | Chargement oxisynth |
|---|---|---|---|---|---|---|
| **GeneralUser GS 2.0.3** (réf.) | <https://www.schristiancollins.com/generaluser.php> ; dépôt <https://github.com/mrbumpy409/GeneralUser-GS> | [GeneralUser GS License v2.0](https://raw.githubusercontent.com/mrbumpy409/GeneralUser-GS/main/documentation/LICENSE.txt) : « You may use GeneralUser GS without restriction for your own music creation, private or commercial » ; « feel free to use it in your software projects » | 32 319 396 (SHA-256 `9575028c…`, identique à `assets/`) | 7 620 750 (= `web-resources.json`) / 10 581 204 | **Oui, sans condition**. Risque résiduel écrit par l'auteur : provenance de certains échantillons non garantie (« I cannot be 100% sure where all of the samples originated », aucun ne vient de banques ou CD du commerce) | SF2 0,4 s ; SF3 1,7 s ; LUFS 8 mesures −20,9 |
| **MuseScore_General 0.2** | <https://ftp.osuosl.org/pub/musescore/soundfont/MuseScore_General/> | [MIT](https://ftp.osuosl.org/pub/musescore/soundfont/MuseScore_General/MuseScore_General_License.md) (Frank Wen 2000-2002, M. Cowgill 2014-2017, S. C. Collins 2018-2019) ; provenance par preset : [Sample_Sources.csv](https://ftp.osuosl.org/pub/musescore/soundfont/MuseScore_General/MuseScore_General_Sample_Sources.csv) | 215 614 036 (`ee51d2c4…`) | officiel 39 900 972 (`5b85b6c2…`) ; q. 4 26 817 104 | **Oui** : MIT ; condition : garder la notice de copyright et le texte de licence avec toute copie, et les crédits (Ethan Winer, Michael Schorsch) | SF2 0,4 s ; SF3 4,2 à 4,9 s |
| **MS Basic 2.0.0** | binaire : <https://github.com/Meowchestra/SoundFonts/releases/tag/v2.0.0> (miroir tiers) ; source de l'auteur : <https://github.com/mrbumpy409/MS_Basic> (« for sfutils demo; not a official repository ») | [MIT](https://github.com/Meowchestra/SoundFonts/blob/main/LICENSE) (même chaîne que ci-dessus ; changelog 2.0.0 : échantillons d'origine restaurés, retrait des cordes VSCO, 4 février 2026) | 215 641 790 (`f4ed62be…`) | miroir 39 926 686 (`3e7c855b…`) ; q. 4 26 826 196 ; q. 8 40 073 962 | **Oui** (MIT, mêmes conditions). Réserve : aucun binaire 2.0.0 officiel de MuseScore, seulement un miroir et une source « non officielle » de l'auteur ; à figer par SHA-256 | SF2 0,3 s ; SF3 4,3 à 4,8 s ; LUFS −15,0 |
| **MS Basic de MuseScore (`main`)** | <https://github.com/musescore/MuseScore/tree/main/share/sound> | [MS Basic_License.md](https://raw.githubusercontent.com/musescore/MuseScore/main/share/sound/MS%20Basic_License.md) : MIT (texte de MuseScore_General 0.2) | — | 51 278 610 (`5ea2375e…`, fichier interne `MuseScore_General_HQ v0.2`, avec cordes VSCO 2) | Oui (MIT) | chargé sans erreur |
| **FluidR3 GM 2-2** | <https://ftp.osuosl.org/pub/musescore/soundfont/fluid-soundfont.zip> | MIT selon README/COPYING (repris dans la [licence MuseScore_General](https://ftp.osuosl.org/pub/musescore/soundfont/MuseScore_General/MuseScore_General_License.md)) | 148 345 256 (`2ae766ab…`) | 21 734 588 | **Conditionnel** : le champ commentaire du fichier SF2 contient « DO NOT REDISTRIBUTE ANY OF THESE SAMPLES », en contradiction avec le MIT du README. Les fichiers MuseScore_General/MS Basic ont remplacé ce commentaire par « Released under the MIT license ». À ne pas livrer tel quel | SF2 0,3 s ; SF3 4,8 s |

### Modules par instrument (FreePats et Salamander), assemblés pour la combinaison B

Licences : page des licences de FreePats <https://freepats.zenvoid.org/licenses.html> et fichier de licence
de chaque archive. Tous les fichiers SF2 chargent sans erreur (banque mono-instrument : le moteur avertit
« non compatible General MIDI », attendu).

| Module | Page (2026-11-02) | Licence | SF2 → SF3 q. 4 (octets) | Plage de touches | Remarque |
|---|---|---|---|---|---|
| Upright Piano KW | <https://freepats.zenvoid.org/Piano/acoustic-grand-piano.html> | CC0 1.0 | 57 377 848 → 3 763 684 | 21–108 | 132 échantillons, 2 couches de vélocité, piano droit de salon |
| Yamaha Disklavier Pro (« YDP-GrandPiano ») | idem | [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/) (Zenph Studios / OLPC ; crédit requis) | 118 398 836 → 10 798 640 | 20–109 | 121 échantillons ; non rendu à l'écoute |
| Salamander Grand Piano V3 | idem ; [dépôt](https://github.com/sfzinstruments/SalamanderGrandPiano) | [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/) (Alexander Holm ; crédit requis) | 1 266 462 334 → 98 818 508 | 21–108 | 960 échantillons, 16 couches ; chargement SF3 16 à 25 s |
| Drawbar / Percussive / Rock Organ | <https://freepats.zenvoid.org/Organ/electric-organ.html> | CC0 1.0 | 6 505 320 / 14 545 902 / 13 625 288 → 404 416 / 771 924 / 1 301 222 | 31–108 | enregistrés depuis setBfree (logiciel libre GPL ; ce sont des enregistrements de sa sortie, pas de son code) |
| Spanish Classical Guitar (nylon) | <https://freepats.zenvoid.org/Guitar/acoustic-guitar.html> | CC0 1.0 | 19 822 546 → 1 391 926 | 29–88 | 48 échantillons |
| FSBS Electric Guitar Jazz / Clean | <https://freepats.zenvoid.org/ElectricGuitar/clean-electric-guitar.html> | CC0 1.0 | 7 768 134 → 451 672 / 10 854 982 → 747 812 (variantes « bridge small ») | 35–86 | Fender sampled direct puis ampli et effets |
| Finger Bass YR / Picked Bass YR | <https://freepats.zenvoid.org/ElectricGuitar/clean-electric-bass.html> | CC0 1.0 | 4 220 300 → 279 568 / 3 527 750 → 278 164 | 26–45 / 26–46 | **muettes au-dessus de la touche 45/46** : mesuré, 14 % des notes de basse d'Isaac et 9 % de celles du swing (> 45) seraient perdues ; corrigé dans la fusion par `--extend-keys` (transposition des derniers échantillons) |
| Tenor Saxophone | <https://freepats.zenvoid.org/Reed/saxophone.html> | CC0 1.0 (échantillons VCSL CC0) | 28 117 404 → 3 279 936 | 43–89 | boucles infinies, attaque rapide |
| World Percussion | <https://freepats.zenvoid.org/Percussion/world-and-rare-percussion.html> | CC0 1.0 | 9 256 408 → 1 729 992 | 48–71 | cajón 48–50, bongos 51–53, shakers 54–56, tambourin 57–58, clap 59, claves 60, castagnettes 61, congas 62–66, maracas 67–68, darbuka 69–71 ; **pas** de timbales, cloche, cuíca, guiro, agogo ; mapping propre à la banque (pas GM) |
| MuldjordKit | <https://freepats.zenvoid.org/Percussion/acoustic-drum-kit.html> | CC BY 4.0 (Lars Muldjord) | 219 362 048 → 20 591 654 | 48–66 | kit rock/metal, **pas de balais** ; non retenu |

### Combinaisons construites pour l'écoute

Le moteur ne charge qu'**une banque principale** (plus une banque de percussion à décalage de banque 1) :
combiner demande une fusion hors ligne (`sf2merge.py`, annexe). Base : MS Basic 2.0.0 SF2.
Remplacés : 0:0 piano, 0:16/17/18 orgues, 0:24 nylon, 0:26 guitare jazz, 0:27 guitare claire, 0:33/34
basses YR, 0:66 sax ténor ; ajouté : 128:64 World Percussion. Plages de touches ouvertes (0–127) sur les
presets ajoutés hors kits.

| Combinaison | Piano | SF2 (octets) | SF3 q. 4 / q. 8 (octets) | Chargement |
|---|---|---|---|---|
| `fp-up` | Upright Piano KW (CC0) | 391 256 560 | 41 219 396 / 64 697 118 | SF2 0,5 s ; SF3 6,8 s |
| `fp-sal` | Salamander (CC BY 3.0) | 1 600 341 152 | 136 274 328 / — | SF2 3,9 s ; SF3 22 s |

## 4. Qualité des programmes utiles, par famille

Mesure objective seulement : nombre d'échantillons et Mo de PCM 16 bits du preset (couches de vélocité,
échantillonnage serré), et « sonne » = LUFS du solo moteur > −50 (palette §7). **Tous** les programmes
balayés sonnent dans les cinq banques (aucun silence, aucun preset absent). La taille n'est pas la qualité :
GeneralUser GS est réglé pour une faible empreinte et ses synthés à base d'ondes tiennent en quelques Ko.

| Famille (programme GM 0-based) | GeneralUser GS | FluidR3 | MS Basic 2.0.0 | Modules libres |
|---|---|---|---|---|
| Piano de concert 0 | 17 éch., 2,2 Mo | 40 éch., 7,8 Mo | **144 éch., 100 Mo** (Splendid Grand, domaine public confirmé par le représentant d'AKAI, selon l'auteur) | Upright KW 132 / 57 Mo ; Disklavier 121 / 118 Mo ; Salamander 960 / 1 266 Mo |
| Rhodes (4, « Tine EP ») | 31 éch., 0,4 Mo | « Rhodes EP » 26 éch., 3,2 Mo | 13 éch., 1,6 Mo (= échantillons de FluidR3) | **aucun SF2 propre trouvé** (§5) |
| Orgue 16 / 17 / 18 | 11 / 10 / 15 éch., < 0,3 Mo | 2 / 20 / 17 éch., 0,2 / 4,1 / 3,1 Mo | identique à FluidR3 | FreePats : 16 / 32 / 32 éch., 6,5 / 14,5 / 13,6 Mo |
| Guitares nylon 24 / jazz 26 / claire 27 | 12 / 15 / 9 éch., 0,3–0,5 Mo | 10 / 12 / 10 éch., 1,7 / 0,8 / 0,7 Mo | identique à FluidR3 | nylon 48 éch., 19,8 Mo ; FSBS jazz et clean 13 éch., 7,8 et 10,9 Mo |
| Basse acoustique 32 | 4 éch., 0,1 Mo | 9 éch., 0,9 Mo | identique à FluidR3 | **aucune contrebasse pizzicato en SF2 propre** (Sneakybass CC0 en SFZ, §5) |
| Basses électriques 33 / 34 | 20 / 13 éch., 0,1–0,2 Mo | 9 / 9 éch., 1,3 / 1,7 Mo | identique à FluidR3 | YR : 12 / 13 éch., 4,2 / 3,5 Mo (plage 26–46) |
| Contrebasse arco 43 | 13 éch., 0,6 Mo | 16 éch., 1,5 Mo | 18 éch., 2,0 Mo | — |
| Trompette 56 / trombone 57 / sourdine 59 / cors 60 / section 61 | 7 / 8 / 12 / 19 / 6 éch., 0,1–0,9 Mo | 6 / 13 / 6 / 14 / 10 éch., 0,2–3,0 Mo | identique à FluidR3 (cors : 7 éch.) | VSCO 2 CE (CC0) : trompette, cor, trombone, tuba d'orchestre en WAV/SFZ, pas de SF2 ni de cuivres de jazz |
| Sax alto 65 / ténor 66 | 13 / 16 éch., 0,5 / 0,4 Mo | 32 / 30 éch., 1,6 / 4,3 Mo | 16 / 15 éch., 0,8 / 2,2 Mo | Tenor Sax FreePats 46 éch., 28 Mo |
| Kit standard 128:0 / jazz 128:32 / balais 128:40 | 65 / 65 / 64 éch., 2,7–2,8 Mo | 105 / 105 / 107 éch., 11 Mo | 60 / 60 / 61 éch., 6,4–6,5 Mo | MuldjordKit (rock, sans balais) ; Swirly Drums (§5) |
| Percussions latines GM (24 touches : tambourin 54, cloche 56, vibraslap 58, bongos 60-61, congas 62-64, timbales 65-66, agogo 67-68, cabasa 69, maracas 70, guiro 73-74, claves 75, woodblocks 76-77, cuíca 78-79, triangles 80-81, shaker 82) | 24/24 sonnent | 24/24 | 24/24 | World Percussion : bongos, congas, claves, maracas, tambourin, cajón ; ni timbales, ni cloche, ni cuíca, ni guiro |

Surdo et pandeiro : aucune touche GM, absents partout.

## 5. Banques rejetées, et pourquoi

Une banque à licence floue ou incompatible avec un produit fermé est rejetée, quelle que soit sa qualité.

| Banque | Lien (consulté 2026-11-02) | Raison |
|---|---|---|
| **Timbres of Heaven 4.0** (Don Allen) | <https://www.midkar.com/SoundFonts> | « may not be distributed on any site without the express written permission of Don Allen … All Rights Reserved » : pas de redistribution sans accord écrit. |
| **Arachno 1.0** | <https://www.arachnosoft.com/main/soundfont.php?documentation> | « primarily distributed for private, non-commercial purposes only … obtain a written consent from the original authors » : non commercial. |
| **SGM V2.01** (Shan) | <https://musical-artifacts.com/artifacts/3506> | Licence « by » sans version sur Musical Artifacts ; l'étude SK3 du labo avait relevé GPL-3.0 sur un autre miroir (non reconsulté aujourd'hui). Pas d'autorisation attribuable à l'auteur : flou. |
| **Sonatina Symphonic Orchestra** | [LICENSE](https://raw.githubusercontent.com/peastman/sso/master/LICENSE) ; [CC Sampling Plus 1.0](https://creativecommons.org/licenses/sampling+/1.0/legalcode) | Le droit commercial couvre des œuvres dérivées « highly transformative » et partielles ; la copie de l'œuvre entière n'est permise qu'en partage non commercial (§3). Une SF2 embarquée est une copie commerciale de l'œuvre. Orchestre seulement, de toute façon. |
| **FluidR3 GM** seule | §3 | Notice interne contradictoire ; MS Basic la remplace avec le même contenu. |
| **TimGM6mb** | [manuel MuseScore](https://musescore.org/en/print/book/export/html/278624) | GPL v2 : incompatible avec une distribution fermée sans analyse juridique, et banque de 6 Mo. |
| **FreePats GM, FSS Steel Guitar, Kawai Upright Piano 2017** (variantes GPL) | [licences](https://freepats.zenvoid.org/licenses.html), [guitare acier](https://freepats.zenvoid.org/Guitar/steel-acoustic-guitar.html), [manuel MuseScore 3](https://musescore.org/en/print/book/export/html/278624) | GPL v3+ « avec exception spéciale » : exception non analysée pour un jeu ; l'alternative CC0 existe pour la plupart (la version 2022 d'Upright Piano KW retenue est CC0). Le GM de FreePats est de plus incomplet (45 programmes). |
| **jRhodes3d** (Rhodes Mark I, Jeff Learman) | [LICENSE](https://github.com/sfzinstruments/jlearman.jRhodes3d/blob/master/LICENSE) | Échantillons CC BY-NC 4.0 ; usage commercial « contact me ». |
| **Carlos Vaquero, Double Bass Pizzicato** (SF2 de Greg Kennedy) | [archive.org](https://archive.org/details/SF2_CarlosVaqueroDoubleBassPizzicato), [Freesound](https://freesound.org/people/Carlos_Vaquero/sounds/153802/) | CC BY-NC (3.0 sur archive.org, 4.0 sur Freesound) : non commercial. |
| **« Rhodes Electric Piano »** (Musical Artifacts 5001) | <https://musical-artifacts.com/artifacts/5001> | Échantillonné depuis Pianoteq 8 (logiciel commercial) : chaîne de droits non établie, pas de texte de licence du logiciel vérifié. |
| **« Studio Rhodes LA »** (Musical Artifacts 7017) | <https://musical-artifacts.com/artifacts/7017> | Licence « cc-sample » : non standard. |
| **Banques « ripées » de claviers et expandeurs** (Yamaha PSR-SX700, Roland SRX, Korg AG-10, Tyros…) | p. ex. <https://musical-artifacts.com/artifacts/9553> (by-nc-sa), 9181, 9739, 10097 (« copyright ») | Échantillons du commerce : licence copyright, NC ou grise. Exemple : les kits balais trouvés (9181, 9612, 9740, 9791) sont tous de ce type. |
| **Compilations de Mike77154** (Baby Bass, Latin Trumpet, Mini Cheapy Orchestra, Real Brass Deal…) | p. ex. <https://musical-artifacts.com/artifacts/2264>, 3546 | Licence « various » ou by-3 sur des assemblages d'auteurs multiples : provenance non vérifiable. |
| **Philharmonia Orchestra samples** | <https://philharmonia.co.uk/resources/sound-samples/> | Usage commercial libre mais « must not be sold or made available ‘as is’ (i.e. as samples or as a sampler instrument) » : une SF2 livrée est un instrument d'échantillons. |
| **University of Iowa MIS** (contrebasse pizzicato, vibraphone, marimba…) | <https://theremin.music.uiowa.edu/MIS.html> | « may be downloaded and used for any projects, without restrictions » : permission très large mais sans texte de licence ni mention de redistribution comme instrument ; pas de SF2 officielle. **Flou, rejeté ; à reconsidérer si le propriétaire accepte ce texte.** |
| **Aegean Symphonic Orchestra, Salamander C5 Light, Magic Sound Font** | liste du [manuel MuseScore 3](https://musescore.org/en/print/book/export/html/278624) | Licences non vérifiées (orchestre ; dérivé de Salamander ; compilation Soundfonts4U) : écartées faute de chaîne de droits. Le Salamander complet FreePats (CC BY 3.0) couvre le besoin piano. |

### Sources libres sans SF2 (pistes de construction, hors périmètre de L1)

Le labo sait construire des SF2 à partir d'échantillons CC0 (`make fiddle-sf2`, `harmonium_lab/bank/build.py`).
Candidats **CC0** à convertir, tous en WAV + SFZ :

| Besoin | Source | Licence | Taille / remarque |
|---|---|---|---|
| Contrebasse pizzicato | [Sneakybass](https://github.com/sfzinstruments/karoryfer.sneakybass) (Otto Rubner 1958, cordes Spirocore, 4 alternances) | CC0 | 1 contrebasse jouée très doux ; le meilleur candidat pour la marche de basse |
| Kit à balais | [Swirly Drums](https://github.com/sfzinstruments/karoryfer.swirly-drums) (caisse claire, charleston, toms, **cloche, bongos, darbouka, djembé**) | CC0-1.0 (dépôt) | 869 Mo de WAV ; kit « indie », frappes aux balais |
| Kit baguettes de jazz | [Virtuosity Drums](https://github.com/sfzinstruments/virtuosity_drums) | CC0 | pas de balais |
| Percussions latines, sax | [VCSL](https://github.com/sgossner/VCSL) | CC0 | cloches, agogo, claves, guiro, cabasa, congas, bongos, shakers, tambourins, cajón, sax ténor |
| Cuivres, bois, orgue, piano droit | [VSCO 2 CE](https://versilian-studios.com/vsco-community/) | CC0 | trompette, cor, trombone, tuba, clarinette, flûte… d'orchestre |

## 6. Chargement oxisynth et SF3

* **Toutes** les banques de §3 (SF2 et SF3) chargent sans erreur dans le moteur ; l'oxisynth du fork lit le SF3.
* Mesure sur machine partagée (ordres de grandeur) : le SF2 se charge en 0,1 à 4 s ; le SF3 décode le Vorbis au
  chargement : 1,7 s (GeneralUser), 4 à 5 s (MS Basic), 7 s (`fp-up`), **16 à 25 s (Salamander et `fp-sal`)**.
  Training et le studio web téléchargent **et** décodent la banque : le temps de décodage compte autant que
  la taille.
* Déterminisme de la chaîne SF3 : la conversion de `GeneralUser-GS.sf2` redonne **exactement** le SHA-256
  de `web-resources.json` (`290b0081…`).
* **Qualité Vorbis.** Le studio convertit en `oggenc -q 4`. L'auteur de MS Basic écrit que « it is not
  recommended to use higher OGG compression (lower values), as this has been shown to create buzzing
  loops and other audible anomalies » et distribue à `-q 0,8` (échelle de sf3convert, proche de `-q 8`
  d'oggenc) : 40 Mo. Je n'ai pas écouté si le q. 4 produit ces défauts sur MS Basic ; les MP3 d'écoute sont
  faits à partir des SF3 q. 4. **Si MS Basic est retenu, livrer en q. 8 (40 Mo)**, ou écouter d'abord.
* Réserve sur MuseScore_General **0.2** (celui de l'OSUOSL) : le changelog 0.2.2 corrige l'orgue 16 (« 8va too
  low ») et d'autres défauts ; MS Basic 2.0.0 les contient.

## 7. Palette (moteur) et provenance

`palette.py` : pour chaque banque, 12 mesures de `wjazzd_swing` avec un profil jetable portant le
programme, la piste en solo, LUFS intégré. 34 balayages par banque, 170 au total : **aucun silence, aucun
preset absent** (colonne `preset` du JSON). Extraits (LUFS du solo) :

| Programme | GeneralUser GS | FluidR3 | MS Basic 2.0.0 | `fp-up` | `fp-sal` |
|---|---|---|---|---|---|
| Piano 0 | −24,9 | −25,8 | −26,3 | −19,2 | −29,4 |
| Rhodes 4 | −24,7 | −18,8 | −20,4 | −20,4 | −20,4 |
| Orgue drawbar 16 | −20,5 | −25,8 | −23,1 | −20,7 | −20,7 |
| Guitare jazz 26 | −23,9 | −29,1 | −24,8 | −18,4 | −18,4 |
| Basse acoustique 32 | −23,5 | −23,5 | −22,1 | −22,1 | −22,1 |
| Contrebasse (arco) 43 | −28,0 | −27,5 | −28,5 | −28,5 | −28,5 |
| Trompette 56 | −22,8 | −28,7 | −19,7 | −19,7 | −19,7 |
| Sax ténor 66 | −25,5 | −21,5 | −20,3 | −17,5 | −17,5 |
| Kit jazz 32, grosse caisse / ride (solos des pistes percussion, densité 2) | −39,4 / −30,4 | −40,4 / −20,9 | −31,2 / −16,0 | −31,2 / −16,0 | −31,2 / −16,0 |

Le niveau d'un programme varie de plus de 10 LU d'une banque à l'autre (piano : de −19 à −29). Les banques
ne sont pas normalisées entre elles.

**Provenance mesurée** (`provenance.py`, part des échantillons du preset de MS Basic 2.0.0 présents à
l'identique, SHA-1, dans FluidR3) : 100 % pour les programmes 4, 16, 18, 24, 26, 27, 32, 33, 34, 56, 57,
59, 60, 61, 65, 66 et les kits 0, 16, 32, 40 ; 44 % pour la contrebasse 43 ; **0 %** pour le piano 0, l'EP FM 5
(partagé avec GeneralUser), la guitare acier 25 et la clarinette 71. Entre GeneralUser GS et FluidR3 : 0 % sur
ces 25 programmes.

## 8. Écoute comparative et mixage

Même graine 7 ; les notes MIDI sont **identiques entre banques** (SHA-256 du MIDI de chaque style et piste :
0 divergence sur 130 rendus, 25 mix et 105 solos). Cinq banques : GeneralUser GS (`gu`), FluidR3 (`fluid`), MS Basic 2.0.0 (`msb2`),
`fp-up`, `fp-sal` ; SF3 q. 4 ; `latin_gm` ne joue que `gu`, `fluid`, `msb2`, `latin_fp` que `fp-*`.

Styles et justification :

| Style | Instruments | Pourquoi |
|---|---|---|
| `wjazzd_swing` (blues puis AABA, 44 mesures, 150 BPM) | piano 0, basse acoustique 32, kit jazz 128:32 | le profil de la phase ; couvre piano, basse, batterie |
| `swing_dense`, `swing_balais` (variantes jetables : densité de batterie 2 constante ; kit 40 pour le second) | idem, avec grosse caisse, caisse claire, toms, relances | **les kits n'étaient pas différenciés** au tempérament neutre (ride + pédale : octet pour octet les mêmes échantillons dans les kits 0, 32 et 40 de GeneralUser) ; sans cette variante les balais ne s'entendent pas |
| `isaac_burning` (32 mesures, 140 BPM, sans Odin : les pistes jouent leur programme SoundFont déclaré) | orgue rock 18, basse au médiator 34, chœur 52, kit rock 128:0 | timbres absents du jazz : orgue, basse électrique, kit rock ; couvre B4 et B8 |
| `latin_gm`, `latin_fp` (profil jetable, 32 mesures, 120 BPM droit) | guitare nylon 24, basse acoustique 32, claves, cloche, maracas, congas, bongos | percussions latines et nylon (B3), aucun profil latin n'existe encore |

### LUFS par piste en solo (`--solo-track`) et écarts à GeneralUser GS

`wjazzd_swing` (kit jazz, densité 1 : pas de grosse caisse, d'où « — ») :

| Banque | mix | Chords | Bass | ride + pédale | crête mix (dBTP) |
|---|---|---|---|---|---|
| GeneralUser GS (réf.) | −20,6 | −23,6 | −23,4 | −31,7 | −6,1 |
| FluidR3 | −18,6 | −25,3 | −23,5 | −21,7 | −4,9 |
| MS Basic 2.0.0 | −15,2 | −26,1 | −22,1 | −16,6 | −1,6 |
| `fp-up` | −14,2 | −18,9 | −22,1 | −16,6 | **−0,4** |
| `fp-sal` | −15,4 | −28,9 | −22,1 | −16,6 | −1,7 |

`swing_dense` (kit jazz, densité 2) : mêmes colonnes, plus la grosse caisse (PercussionLow) : GeneralUser −37,8 ;
FluidR3 −38,4 ; MS Basic et `fp-*` −29,5 ; crête de `fp-up` **+0,1 dBTP** (écrêtage possible).

`isaac_burning` (mix / chœur / orgue / basse / charleston-caisse-toms / grosse caisse) : GeneralUser
−21,4 / −40,7 / −24,2 / −24,7 / −32,7 / −35,1 ; FluidR3 −19,8 / −35,7 / −23,8 / −22,1 / −34,2 / −36,3 ; MS Basic
−19,5 / −39,3 / −25,5 / −21,1 / −39,1 / −30,5 ; `fp-*` −18,7 / −39,3 / −21,6 / −22,3 / −39,1 / −30,5.

`latin_gm` (mix / accords / basse / percussion haute / basse percussion) : GeneralUser −19,8 / −24,7 / −24,2 /
−24,7 / −28,9 ; FluidR3 −19,1 / −23,2 / −24,1 / −24,4 / −26,7 ; MS Basic −19,2 / −24,4 / −22,7 / −25,2 / −27,7.
`latin_fp` : −18,2 / −20,9 / −22,7 / −25,5 / −27,9 pour les deux combinaisons.

**Écarts à signaler (le mixage du profil est calibré sur GeneralUser GS).** Avec la même graine, le même
réglage de mixage et le master à +25 dB de `wjazzd_swing` :

| Piste (swing) | FluidR3 | MS Basic 2.0.0 | `fp-up` | `fp-sal` |
|---|---|---|---|---|
| Accords | −1,7 LU | −2,5 | **+4,7** | −5,3 |
| Basse | −0,1 | +1,3 | +1,3 | +1,3 |
| Ride + pédale | **+10,0** | **+15,1** | +15,1 | +15,1 |
| Grosse caisse (`swing_dense`) | −0,6 | +8,3 | +8,3 | +8,3 |
| Mix complet | +2,0 | +5,4 | +6,4 | +5,2 |

Le profil `wjazzd_swing` fixe la batterie à −9 dB pour que le ride soit 8 LU sous la basse et les accords
(« la batterie à 0 dB couvrait tout à l'écoute du propriétaire », README du profil). Avec MS Basic ou FluidR3 le
ride repasse **au-dessus** des accords (+9,5 LU pour MS Basic) : **tout changement de banque impose de
recalibrer le mixage de chaque profil** (batterie environ −14 à −15 dB par rapport à l'actuel pour MS Basic,
environ −10 dB pour FluidR3), ou de normaliser la banque une fois (gain par preset dans la construction de
la banque, plus économe que recalibrer chaque profil). Les MP3 d'écoute sont **ramenés à −21 LUFS par fichier**
(gains de −6,9 à +0,4 dB, `mp3_gains.json` dans le cache) : le volume ne doit pas décider ; ne juger que les timbres.

### Ce qui est à écouter (page LAN, minutes:secondes)

`wjazzd_swing` : 0:00–0:19 comping de piano et marche de basse (blues) ; 0:12,8 ii-V ; 0:19,2 AABA ; 0:44,8
pont. `swing_dense` et `swing_balais` : relances de fin de phrase à 0:17,6, 0:30,4, 0:43,2, 0:56, 1:08,8 (caisse
claire, toms) ; comparer jazz et balais au même instant. `isaac_burning` : pédale d'orgue 0:00–0:13,7, entrée
du chœur 0:13,7, relance 0:25,7. `latin_*` : motif complet dès 0:00 ; ii-V 0:16, AABA 0:24, pont 0:56.

## 9. Conséquences si l'option A ou B est retenue (non appliquées)

* **`harmonium_studio/web-resources.json`** : remplacer l'entrée `main-bank` (fichier, `bytes`, `sha256`), la
  `license` (`GeneralUser-GS-LICENSE.txt` → texte MIT de MS Basic + notices ; garder la licence GeneralUser GS
  seulement si elle est encore livrée), nouvelle release **`studio-resources-3`** (`web_resources.py stage`,
  `gh release create`, puis commit du JSON, sinon `check-profiles` échoue). Poids de téléchargement : 7,6 → 40 Mo
  (A, q. 8) ou 65 Mo (B, q. 8).
* **Mentions** (`assets/README.md`, mentions du jeu et du studio) : copyright MIT (Frank Wen, Michael Cowgill, S.
  Christian Collins, Ethan Winer pour les blocs temple, Michael Schorsch pour les cymbales de marche) et texte de
  licence livré avec la banque ; si Salamander ou Disklavier : crédit CC BY 3.0 (Alexander Holm ; Zenph Studios /
  OLPC) ; si MuldjordKit : CC BY 4.0 (Lars Muldjord). Les modules CC0 n'imposent rien.
* **Profils à revoir** : mixage de chaque profil jouant en SoundFont (`mix.tracks.*.level_db`) ; commentaire de
  `profiles/wjazzd_swing/percussion.toml` et ligne « kit jazz de GeneralUser GS » de son README ; programmes
  GM : basse 32 (4 échantillons dans GeneralUser, 9 dans MS Basic) contre 33 ou 43 ; Rhodes 4 = « Tine EP » de
  FluidR3 dans MS Basic ; Isaac sans Odin (les zones Odin retombent sur le programme SoundFont déclaré).
* **Construction** : l'option B demande une étape de fusion avant `sf2_to_sf3.py` (le moteur charge une banque) ;
  garder la plage de touches ouverte (basse YR) et normaliser les niveaux (piano droit +7 LU au-dessus du piano
  de MS Basic). Cette étape n'existe pas dans le dépôt.
* **Manques à traiter hors L1** : contrebasse pizzicato (Sneakybass CC0), kit à balais réel (Swirly Drums CC0),
  Rhodes (aucune source propre), cuivres de jazz (aucune source propre : VSCO CE est orchestral), pandeiro, surdo.

## 10. Reproduction

Cache : `/tmp/harmonium-019/l1/` (banques dans `banks/`). Outils : `l1-annexe-outils.md` (recopier les blocs dans
`/tmp/harmonium-019/l1/work/`). Binaire : `cargo build --release -p harmonium-cli` dans
`~/Project/bascanada/harmonium`.

```sh
L1=/tmp/harmonium-019/l1; mkdir -p $L1/{banks/dl,banks/sf3,banks/sf2,banks/merged,render,work,profiles}
cd $L1/banks
# banques en cache du labo (hors dépôt) : GeneralUser GS 2.0.3, FluidR3, MuseScore_General 0.2 (make fiddle-samples)
for f in generaluser fluidr3 musescore; do ln -sf ~/Project/bascanada/harmonium_lab/samples/fiddles/raw/$f.sf2 $f.sf2; done
cd dl
curl -sSLO https://ftp.osuosl.org/pub/musescore/soundfont/MuseScore_General/MuseScore_General.sf3
curl -sSLO https://github.com/Meowchestra/SoundFonts/releases/download/v2.0.0/MS_Basic-v2.0.0.sf2
curl -sSLO https://github.com/Meowchestra/SoundFonts/releases/download/v2.0.0/MS_Basic-v2.0.0.sf3
curl -sSL -o MS_Basic_HQ_0.2.sf3 "https://raw.githubusercontent.com/musescore/MuseScore/main/share/sound/MS%20Basic.sf3"
# FreePats (CC0 sauf Salamander, Disklavier : CC BY 3.0) : voir les URLs du §3, bsdtar -xf pour .7z/.tar.xz/.tar.bz2
# puis copier chaque .sf2 dans banks/sf2/ (Salamander.sf2, UprightPianoKW-20220221.sf2, …)
cd $L1/banks
# fusion (outil de l'annexe) : base MS Basic 2.0.0, préfixer --extend-keys, une ligne --add par module (§3)
python3 $L1/work/sf2merge.py merged/fp-up.sf2 dl/MS_Basic-v2.0.0.sf2 --extend-keys \
  --add sf2/UprightPianoKW-20220221.sf2 '0:0=>0:0' --add sf2/DrawbarOrganEmulation-20190712.sf2 '0:0=>0:16' \
  --add sf2/PercussiveOrganEmulation-20190715.sf2 '0:0=>0:17' --add sf2/RockOrganEmulation-20190715.sf2 '0:0=>0:18' \
  --add sf2/SpanishClassicalGuitar-20190618.sf2 '0:0=>0:24' --add sf2/EGuitarFSBS-jazz_bridge_small_20260807.sf2 '0:0=>0:26' \
  --add sf2/EGuitarFSBS-clean_bridge_small_20260807.sf2 '0:0=>0:27' --add sf2/FingerBassYR_20190930.sf2 '0:0=>0:33' \
  --add sf2/PickedBassYR_20190930.sf2 '0:0=>0:34' --add sf2/TenorSaxophone-20200717.sf2 '0:0=>0:66' \
  --add sf2/WorldPercussion_20200905.sf2 '0:0=>128:64'
# (`fp-sal` : même commande avec sf2/Salamander.sf2 à la place du piano droit)
# conversion SF3 (qualité 4 du studio ; ajouter « 8 » en dernier argument pour la qualité 8)
S3=~/orca/workspaces/harmonium/l1-banques-sf2/harmonium_studio/scripts/sf2_to_sf3.py
for n in generaluser fluidr3; do python3 $S3 $n.sf2 sf3/$n.sf3; done
python3 $S3 dl/MS_Basic-v2.0.0.sf2 sf3/MS_Basic-v2.0.0-q4.sf3 ; python3 $S3 dl/MS_Basic-v2.0.0.sf2 sf3/MS_Basic-v2.0.0-q8.sf3 8
python3 $S3 merged/fp-up.sf2 sf3/fp-up.sf3 ; python3 $S3 merged/fp-sal.sf2 sf3/fp-sal.sf3
# profils jetables, chargement, rendus + LUFS, balayage de palette
python3 $L1/work/make_profiles.py ~/orca/workspaces/harmonium/l1-banques-sf2/profiles $L1/profiles
python3 $L1/work/loadtest.py
cd ~/Project/bascanada/harmonium && python3 $L1/work/render_l1.py   # 25 mix + 105 solos, graine 7 ; écrit $L1/render/results.json
python3 $L1/work/palette.py                                          # 34 programmes × 5 banques
~/Project/bascanada/harmonium_lab/.venv/bin/python $L1/work/latinkeys.py
python3 $L1/work/families.py ; python3 $L1/work/provenance.py
```

Un rendu unique (exemple : GeneralUser, swing, graine 7) :

```sh
cd ~/Project/bascanada/harmonium
./target/release/harmonium-cli --export --bars 44 --style wjazzd_swing --seed 7 \
  --soundfont /tmp/harmonium-019/l1/banks/sf3/generaluser.sf3 \
  --record-wav /tmp/x.wav --record-midi /tmp/x.mid [--solo-track Chords|Bass|PercussionLow|PercussionHigh]
ffmpeg -nostats -i /tmp/x.wav -af ebur128=peak=true -f null -          # LUFS intégré et crête
ffmpeg -y -i /tmp/x.wav -af volume=<gain>dB -codec:a libmp3lame -b:a 320k out.mp3   # gain = −21 − LUFS du mix
```

Les 25 MP3 (un par mix de `results.json`, nom `<style>_<banque>.mp3`, dans
`~/Project/bascanada/harmonium_lab/corpus/ecoute_019/l1/`) :

```sh
python3 - <<'E'
import json, os, subprocess
out = os.path.expanduser('~/Project/bascanada/harmonium_lab/corpus/ecoute_019/l1'); os.makedirs(out, exist_ok=True)
for x in json.load(open('/tmp/harmonium-019/l1/render/results.json')):
    if x['track'] != 'mix': continue
    g = round(-21.0 - x['lufs'], 1)
    subprocess.run(['ffmpeg', '-y', '-v', 'error', '-i', x['wav'], '-af', f'volume={g}dB', '-codec:a', 'libmp3lame',
                    '-b:a', '320k', f"{out}/{x['style']}_{x['bank']}.mp3"], check=True)
E
```

Profils jetables (`make_profiles.py`) : `swing_dense` = `wjazzd_swing` avec `percussion_intensity = [[0.0, 1.0], [1.0, 1.0]]`
dans `dynamics.toml` ; `swing_balais` = idem avec `percussion_program = 40` ; `latin_gm` et `latin_fp` = idem
avec `chords_program = 24`, `percussion.toml` remplacé (motif d'une mesure, voix GM ou World Percussion),
120 BPM, swing 0,5.

## 11. Limites

* Aucune écoute par l'agent : le jugement de qualité sonore (piano, balais, contrebasse, Rhodes, cuivres) reste
  à faire sur la page LAN.
* Les temps de chargement viennent d'une machine partagée avec d'autres agents.
* « Sonne » = niveau non nul au solo moteur, pas « est beau » ; la taille d'un preset n'est pas sa qualité.
* La provenance des échantillons de FluidR3 (« samples found in the public domain », crédits listés) repose sur
  la déclaration de l'auteur ; celle de GeneralUser GS aussi. Aucune des deux n'est vérifiable de l'extérieur.
* Le binaire MS Basic 2.0.0 vient d'un miroir tiers ; le SHA-256 est à figer avant toute release.
* Aucun avis juridique : lecture des textes de licence, pas analyse de contrat.
