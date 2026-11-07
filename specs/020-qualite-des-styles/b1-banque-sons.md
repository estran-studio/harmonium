# B1 — banques de sons pour la phase 020

**Statut : trois banques prêtes pour l'écoute de William ; aucune substitution définitive décidée.** Les profils et le moteur ne changent pas. Comparaison sur les 13 identifiants d'E1, graine 7, huit mesures, sans Odin et sans écouter les fichiers. La banque trad reste strictement identique. Le choix est à faire à l'oreille : L1 n'avait pas permis à William de distinguer les banques entières.

## Sources examinées et droit de distribution

« Libre » signifie ici redistribution du **fichier de samples** avec un jeu fermé et depuis un site web, pas seulement utilisation de sa sortie audio. Les tailles sont des ordres de grandeur des téléchargements/sources examinés ; les tailles exactes des trois banques assemblées sont plus bas. Une licence du code du synthé ne s'étend pas automatiquement aux samples.

| Source ; preuve de licence à la source | Couverture et provenance ; taille | Décision |
|---|---|---|
| [GeneralUser GS](https://github.com/mrbumpy409/GeneralUser-GS/blob/main/documentation/LICENSE.txt), autorisation de redistribution de la banque, réserves sur l'origine de certains samples | GM complet, SF2 de référence déjà distribué ; hash en manifeste | Référence et base de `b1-gu`. Ne pas prétendre que chaque échantillon a une provenance individuelle garantie. |
| [MuseScore General 0.2 / MS Basic 2.0.0](https://github.com/musescore/MuseScore/blob/main/share/sound/MS%20Basic_License.md), MIT pour l'assemblage et les samples autorisés ; notices de composants à conserver | GM entier, ~216 Mo SF2 / ~40 Mo SF3, piano Splendid public domain, autres emprunts à FluidR3 sous les autorisations indiquées | `ms-basic` et base mélodique de `b1-msb`. MS Basic est la version complète disponible ; MuseScore General 0.2 déjà essayé en L1, sans bénéfice général à l'écoute. Conserver les notices d'amont avant toute distribution. |
| [FluidR3](https://github.com/musescore/MuseScore/blob/main/share/sound/MS%20Basic_License.md) et notice embarquée du SF2, ~150 Mo | GM, samples d'origines multiples ; certaines notices embarquées portent « DO NOT REDISTRIBUTE ANY OF THESE SAMPLES » | Pas de distribution directe d'une nouvelle banque FluidR3 sans résolution de cette contradiction ; MS Basic sous sa notice propre est l'alternative examinée. |
| [FreePats](https://freepats.zenvoid.org/licenses.html), vérifier chaque instrument, **CC0** pour ceux importés | 10 SF2 isolées, ~4–393 Mo suivant l'instrument ; prises FreePats de sax ténor, orgues, guitare nylon/jazz/clean/distorsion, basses doigts/médiator | Retenues dans les deux assemblages, détail et empreintes ci-dessous. Le catalogue GM entier mélange d'autres licences (GPL+exception, etc.) et ne couvre pas tout : pas de substitution globale. |
| [Salamander Grand Piano](https://raw.githubusercontent.com/sfzinstruments/SalamanderGrandPiano/master/README.md), CC BY 3.0 | Piano acoustique seul ; ~99 Mo même en SF3 q4 selon L1, chargement 16–25 s | Écarté : gonfle le web/chargement pour un programme qui n'est pas la faiblesse principale d'E1 ; piano de la base conservé. |
| [Arachno](https://www.arachnosoft.com/main/soundfont.php?documentation), attribution/permissions de tiers à négocier | GM complet, provenance hétérogène | Écarté : redistribution commerciale non acquise pour tous les samples. |
| [Timbres of Heaven](https://www.midkar.com/SoundFonts/), permission écrite du créateur exigée | GM complet, ensemble de fontes tierces | Écarté : pas de permission de redistribution dans un jeu fermé. |
| [SGM V2.01](https://github.com/open-soundfonts/SGM_V2_01_soundfonts/blob/master/LICENSE), miroir GPL-3.0 | GM complet, autre description « attribution only » sur Musical Artifacts | Écarté : droits contradictoires, GPL du miroir incompatible avec le besoin sans permission complémentaire. |
| [Musyng Kite](https://www.kvraudio.com/forum/viewtopic.php?t=351893), fil de l'auteur | GM d'environ 990 Mo ; provenance et droit de redistribution des samples non vérifiables | Écarté ; ne pas inférer le droit d'embarquer une banque à partir d'un téléchargement gratuit. |
| [Sonivox EAS](https://github.com/EmbeddedSynth/sonivox/blob/master/README.md), Apache-2.0 pour le projet | Synthèse/table d'ondes et samples embarqués dans du code, pas banque GM SF2/SF3 prête à charger | Écarté pour l'essai oxisynth : changer de moteur ou extraire la table ne répond pas au brief. |
| [VSCO 2 Community Edition](https://versilian-studios.com/vsco-community/), CC0 | Orchestre SFZ (~3 Go), sections de cuivres et solistes classiques, pas stabs de big band/section funk | Examiné ; écarté du paquet pour poids et pertinence. Les cuivres GM restent de la base, limite d'écoute signalée. |
| [VCSL — Versilian Community Sample Library](https://github.com/sgossner/VCSL/blob/master/LICENSE), CC0 | Percussions isolées ; sélection locale ~163 Mo, bongos/congas, cloche, agogo, shaker, clave, guiro | Retenue pour les touches GM disponibles des kits assemblés. Ne contient pas de surdo GM ni de cuica/timbales isolées vérifiées : touches correspondantes de la base conservées. |
| [jRhodes3d](https://raw.githubusercontent.com/sfzinstruments/jlearman.jRhodes3d/master/LICENSE), CC BY-NC | Vrai Rhodes SFZ, environ quelques centaines de Mo | Écarté : clause NC interdit l'usage commercial demandé. |
| [Greg Sullivan E-Pianos](https://raw.githubusercontent.com/sfzinstruments/GregSullivan.E-Pianos/master/README.md), CC BY 3.0, attribution obligatoire | Wurlitzer EP200 et Pianet T, ~21 Mo de source | Retenus sur programmes GM 4 (EP1) et 7 (clavinet), **substituts de timbre**, non pas un Rhodes ou un Clavinet D6. Aucun vrai Rhodes libre et redistribuable trouvé. |
| [FreePats guitares](https://freepats.zenvoid.org/ElectricGuitar/clean-electric-guitar.html) ; [distorsion](https://freepats.zenvoid.org/ElectricGuitar/distorted-electric-guitar.html), CC0 sur les instruments retenus | Jazz 26, clean 27, distorsion 29/30 ; nylon 24 ; jusqu'à ~393 Mo SF2 distorsion | Retenues. Le programme muted 28 reste celui de la base (autres SFZ testés, trop fort au niveau de référence). Overdrive 29 et distorsion 30 partagent la même prise : limite de variété. |
| [Karoryfer Sneakybass](https://raw.githubusercontent.com/sfzinstruments/karoryfer.sneakybass/main/readme.txt), CC0 | ~419 Mo source, contrebasse pizzicato 32 | Retenue. Les basses électriques FreePats 33/34 ; slap 35 et fretless 36 demeurent ceux de la base. Black and Blue Basses CC0 testé puis retiré : niveau disproportionné sur les rendus E1. |
| [Karoryfer Swirly Drums](https://raw.githubusercontent.com/sfzinstruments/karoryfer.swirly-drums/main/license), CC0 | ~1,83 Go source, batterie acoustique SFZ : caisse, grosse caisse, cymbales, charley, toms, balais « stir » | Retenue, échantillons sélectionnés dans les kits GM 0/8/16/24/25/32/40 ; autres touches des kits de base. Une seule variante de round-robin pour éviter des articulations non représentables dans SF2. |
| [MuldjordKit](https://freepats.zenvoid.org/Percussion/acoustic-drum-kit.html), CC BY 4.0, attribution | SF2 ~209 Mo, touches de batterie hors placement GM attendu (48–66), ~53 Mo d'archive | Écarté : exige remappage substantiel et ajoute du poids sans kit jazz/latin distinct. |
| [Salamander Drumkit](https://sfzinstruments.github.io/drums/salamander), CC BY-SA 3.0 | Batterie acoustique SFZ | Écarté : banque recomposée dérivée soumise au partage à l'identique ; à signaler dans un jeu fermé. |
| [AVL Drumkits](https://github.com/x42/avldrums.lv2/blob/master/sf2/README), CC BY-SA 3.0, exception sur les enregistrements de musique produite | Kits SF2 et LV2 | Écarté : exception pour l'audio généré ≠ autorisation de redistribuer une banque dérivée fermée. |
| [Virtuosity Drums](https://github.com/sfzinstruments/virtuosity_drums), CC0 ; [Big Rusty Drums](https://github.com/sfzinstruments/karoryfer.big-rusty-drums), CC0 | SFZ batteries d'environ 1,2 et 2,3 Go | Libres mais écartées pour le coût de conversion/téléchargement et le faible gain démontré face au kit acoustique déjà choisi. |

### Construction reproductible et manifeste

Outils **hors moteur** dans [`b1-outils/`](b1-outils/) : `assemble.py`, `sfzlib.py`, `sf2build.py`, `measure.py` ; Python avec `numpy scipy soundfile`, `ffmpeg`, `oggenc`. `sf2build.py` adapte le fusionneur PCM 16 bits de [l'annexe L1](../019-bibliotheque-accompagnements/l1-annexe-outils.md) : conserve tous les presets non remplacés, copie les banques sources FreePats, convertit les zones SFZ ; kits de la base avec les seules touches sélectionnées remplacées. Les touches hors zone FreePats sont étendues aux extrêmes, sans nouveau programme côté moteur. Les SFZ perdent les round-robins, certaines nuances/CC, et les effets qui n'ont pas d'équivalent SF2 ; l'interprétation est donc une **conversion d'essai**, pas l'instrument original. Swirly est normalisé par couche de vélocité avec au plus +18 dB (+36 dB au plus pour les grosses caisses de faible vélocité, crête ciblée à 0,85), VCSL au plus +24 dB ; guitares 27 à -5 dB, 29/30 à -8 dB. Ce sont des changements de sample/preset dans la banque, **aucun** `mix` de profil modifié. Les kits `b1-msb` réemploient le socle de kits GU pour isoler la comparaison des instruments mélodiques ; `ms-basic` est entièrement d'origine.

Pour reconstruire : déposer les SF2 FreePats extraits dans `<ROOT>/sf2/<nom-archive>/*.sf2`, MS Basic dans `<ROOT>/dl/MS_Basic-v2.0.0.sf2`, les répertoires zip GitHub dans `<ROOT>/src/<dépôt>/<dépôt>-HEAD/` et les WAV sélectionnés de VCSL dans `<ROOT>/src/vcsl/`, puis, depuis le répertoire des scripts, lancer `H020_B1_ROOT=<ROOT> H020_B1_GU=<GENERALUSER> python assemble.py gu` et `… python assemble.py msb`. Résultats `<ROOT>/merged/b1-{gu,msb}.sf2`. Stocker les sources elles-mêmes **hors dépôt** ; vérifier leur empreinte avant toute distribution. Le nom de banque final est `GeneralUser-GS.sf2` pour le studio. Conversion du SF3 q4 (qualité du studio) par `python harmonium_studio/scripts/sf2_to_sf3.py <sf2> <sf3> 4`.

Empreinte de l'archive SF2 telle qu'utilisée, SHA256 ; toutes les fontes FreePats ci-dessous sont CC0 sous la notice de leur page d'instrument, pas sous une licence supposée pour l'ensemble du catalogue :

| Source FreePats | SHA256 du SF2 |
|---|---|
| EGuitarFSBS clean bridge small | `bcea7d4c92a1cc6a26e545a218b34e59037d4bab6cf0fce24396428e6934d69c` |
| EGuitarFSBS dist2 | `0ed99135d955940c1200e09ca9e36e261dfebbe8767ae58da6f298fe29fdd146` |
| EGuitarFSBS jazz bridge small | `d2090931a0339ba6b60c6713bc6d701393d3a420c20e56ad69a8d50de645135d` |
| FingerBassYR | `f5b5cfe04435a9e9037d1f97d515dc596535acf26637db9579d724cd757f179e` |
| PickedBassYR | `e6bff767c352cd27de2de8cfff326d0849a76bb38d876075eecd5ffd2dc5afd0` |
| SpanishClassicalGuitar | `8c192f1fc640a553199a4a43a17ae12354a6a05e3d6aad79bd43b5d29314ff51` |
| DrawbarOrgan | `9b41fa8418f6f1a14924381bc6ccbddbb665bbec2b5f46d4ab549e8d88a2e178` |
| PercussiveOrgan | `42c4e000444c0cef2264392e8ef00818784a1ec2fb49936c394bebbe4720062c` |
| RockOrgan | `5be252317cc511d0d92e94b07e2289540b57e4381ec34b26392e685f9b1d505d` |
| TenorSaxophone | `f7640cf2039662b15b4521bb5615e1ddc426731559b4c31079cb5e9f50e16fa0` |

Pour les arbres SFZ, SHA256 calculé en triant les chemins relatifs et en ajoutant pour chaque fichier `chemin UTF-8 + NUL + SHA256(octets du fichier)` dans le hachage final. Cela authentifie aussi README/licences présents dans l'arbre, sans stocker les WAV sous Git :

| Arbre GitHub ; révision | Nombre de fichiers, octets ; SHA256 de l'arbre |
|---|---|
| `sfzinstruments/GregSullivan.E-Pianos` `8c3e581acda3594b553948ff0222d4f84a698376` | 178 ; 21 484 396 ; `8bf952064c83ac86e63d052688eb6ba9bed2f8c24e00eabd8cd3537c88051fa9` |
| `sfzinstruments/karoryfer.sneakybass` `cc1a67e5c913678b85eaad1a3b5cb180d3be3e2d` | 715 ; 418 713 231 ; `42abd5e08970bca64c9b91f96722c9789a4ae513a59ff59cd32d965eb5bb8462` |
| `sfzinstruments/karoryfer.swirly-drums` `c40dafe0011cb2e54c0c220ff0fa308a11fc60f5` | 5 103 ; 1 826 949 043 ; `b09752235efcf8c5a244613e632f111e53b5d1e6599c163201a1064280da2d2e` |
| `sgossner/VCSL` `c1ea7bcc3c7309650ab0da9d15c9cd1fbc4a4c7e` ; sous-arbre WAV sélectionnés | 354 ; 163 373 032 ; `82e65c4344a912ffec2f0ca31742aedb564500c9de4fe7fe1627489e2cfc6509` |

La licence de chaque composant incorporé doit accompagner la redistribution finale (MIT/CC BY 3.0/CC0 et éventuelles notices de GeneralUser et de MS Basic). Pas de SF2/SF3/WAV dans Git. Attribution explicitement requise pour Greg Sullivan et pour les éléments MS Basic concernés.

## Trois dossiers prêts pour le studio

| Banque / dossier `~/orca/harmonium-020/b1/` | Contenu et intention | SF2 (octets) | SF3 q4 (octets, banque générale) |
|---|---|---:|---:|
| `ms-basic/` | MS Basic 2.0.0 intact, contrôle « autre banque GM » | 215 641 790 | 26 825 958 (conversion q4 ; SF3 du miroir q8 : 39 926 686) |
| `b1-gu/` | GeneralUser + programmes isolés, kit Swirly/VCSL | 607 766 210 | 53 710 982 |
| `b1-msb/` | MS Basic mélodique + mêmes programmes isolés et kits sur socle GU | 785 939 234 | 72 626 426 |

Chaque dossier contient `GeneralUser-GS.sf2` et **le même** `harmonium_trad.sf2` (154 186 octets ; SHA256 `4dc8159837b44c754d3bac9eb645e6fb85e53983d9e52a551f8256b67ef28443`). Le studio consomme le SF2 local, tandis que la taille du SF3 chiffre le futur transfert web ; ce sont deux fichiers alternatifs, pas à additionner. Si le client web télécharge aussi la banque trad, le coût total des SF3 q4 + trad est de 26 980 144 / 53 865 168 / 72 780 612 octets respectivement (`ms-basic` / `b1-gu` / `b1-msb`), contre 7 774 936 octets pour GeneralUser GS q4 + trad selon L1. Les SF3 mesurés ne sont pas placés dans les dossiers d'écoute. Toutes les tailles et empreintes ne valent que pour ces fichiers précis :

| Fichier de banque | SHA256 |
|---|---|
| `ms-basic/GeneralUser-GS.sf2` | `f4ed62be6986e70438c995c03fb457610c75d01976bdc3248da9d0cc43a5613c` |
| `b1-gu/GeneralUser-GS.sf2` | `119cc908114a404afc4ae1c4460d41519b3d747acdad4c5e9b141847be659203` |
| `b1-msb/GeneralUser-GS.sf2` | `a3239bd5bda89e3b4efdcf0716fba8f8bec915d28326d84de8329e096dca4ea1` |
| SF3 q4 `b1-gu` / `b1-msb` | `5628da785b97022706f98a7c8d5b335043120c04ad560daf9a9d5106d2fb4f78` / `56692000931c3e6f2e4999c090d0c312316724583d3c4f24b745bb6be9957f4d` |
| SF3 q4 `ms-basic` ; SF3 miroir q8 | `74f261b02fb7a0decb5f3c38569546dc8d853a77af5d6d7375d918ae7c3cdd86` ; `3e7c855b8956b68fe03112bca98d629c6656ce7f24d7ecdce0919646b19f5e9c` |

La sélection couvre les programmes GM 4, 7, 16–18, 24, 26–30, 32–36 et 66 dans les trois banques (certains par conservation de la base) et les kits GM 0, 8, 16, 24, 25, 32, 40. La présence du preset ne garantit pas que chaque touche possède un sample distinct ou que le son soit meilleur. Les cuivres/section, le piano acoustique, le vrai Rhodes, le slap/fretless, la guitare muted, les timbales/cuica/surdo restent les limites les plus visibles : ne pas présenter ces sons comme « résolus ».

## Écoute demandée à William

Ouvrir **un seul dossier de banque à la fois** dans le studio, puis répéter **les mêmes liens, dans le même ordre** avec GeneralUser GS d'origine comme référence, `ms-basic`, `b1-gu` et `b1-msb`. Les agents n'ouvrent aucun lien ni audio. La graine 7 expose les programmes et kits ci-dessous sur les huit premières mesures contrôlées ; l'habillage ultérieur peut faire varier les programmes. La piste `harmonium_trad.sf2` reste la même.

| Profil et lien | Ce qui change / point à écouter |
|---|---|
| [wjazzd_swing](harmonium://open?profile=wjazzd_swing&seed=7) | Sax ténor 66 en mélodie, guitare jazz 26 dans les accords, contrebasse 32 ; attaque/sustain du walking et articulation du swing. |
| [wjazzd_ballad](harmonium://open?profile=wjazzd_ballad&seed=7) | Contrebasse 32, kit jazz 32 et piano de base ; présence de la basse sans écraser la ballade. |
| [gmd_jazz_waltz](harmonium://open?profile=gmd_jazz_waltz&seed=7) | Piano et Wurlitzer programme 4 aux accords, contrebasse 32 ; balance des trois temps. |
| [gmd_latin_brazilian_bossa](harmonium://open?profile=gmd_latin_brazilian_bossa&seed=7) | Guitare nylon 24 et programme 25 de la base, contrebasse 32, bongo/conga/clave ; netteté du pattern léger. |
| [gmd_latin_brazilian_samba](harmonium://open?profile=gmd_latin_brazilian_samba&seed=7) | Guitare 24, kit balais 40 et percussions latines ; énergie des accents, manque éventuel de surdo/cuica. |
| [gmd_latin_brazilian_baiao](harmonium://open?profile=gmd_latin_brazilian_baiao&seed=7) | Guitare 24, basse 32/33, kit 40 ; poids de la grosse caisse et cohérence avec les syncopes. |
| [gmd_afrocuban](harmonium://open?profile=gmd_afrocuban&seed=7) | Piano/Wurlitzer 4, contrebasse 32 et percussion latine ; claves/bongos audibles sans masque. |
| [gmd_funk](harmonium://open?profile=gmd_funk&seed=7) | Guitare clean 27, basse doigts 33, kit 24 ; attaques funk, vérifier l'absence de vrai clavinet/slap. |
| [gmd_soul](harmonium://open?profile=gmd_soul&seed=7) | Wurlitzer 4, basse doigts 33, kit 40 ; éviter un EP ou des balais trop en avant. |
| [gmd_reggae](harmonium://open?profile=gmd_reggae&seed=7) | Guitare clean/muted 27/28, basse 33, kit 24 ; contretemps secs et basse souple. |
| [gmd_rock](harmonium://open?profile=gmd_rock&seed=7) | Distorsion 29/30, basse 33/34, kit 16 ; densité et équilibre de la saturation et du backbeat. |
| [gmd_pop](harmonium://open?profile=gmd_pop&seed=7) | Piano de base, basse 33/34 et kit standard ; test témoin de l'effet d'un changement de batterie/basse sans guitare. |
| [gmd_blues_shuffle](harmonium://open?profile=gmd_blues_shuffle&seed=7) | Piano de base, basse 32/33 ; relief du shuffle et notes graves de contrebasse. |

## Méthode des mesures

`harmonium-cli --export --bars 8 --seed 7 --style <id> --soundfont <banque> --percussion-soundfont <trad> --record-wav <wav> --record-midi <midi>` puis les quatre rendus isolés `--solo-track Chords|Bass|PercussionLow|PercussionHigh`, exactement la procédure de L1 §8. `ffmpeg -af ebur128=peak=true -f null -` relève la valeur intégrée `I` (LUFS) et la crête vraie dBFS ; fichier WAV et MIDI temporaires effacés après lecture. Les pistes muettes mesurées à -70 LUFS ne définissent pas un gain utile : inscrire « — ». Identité des SHA256 MIDI sur des rendus de même style/piste = contrôle des mêmes notes entre les banques, pas validation du timbre ou des performances. Les LUFS comparent l'énergie, **pas** la qualité musicale. Pour la proposition de mix, une correction initiale de `mix` de `-Δ LU` dB par piste (candidate – référence) réduirait l'écart, à valider à l'oreille et en crête ; les pistes muettes n'ont pas de correction. Ne pas appliquer ces corrections avant la décision de William.
