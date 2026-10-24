# Voix — étude de faisabilité (phase 012, point 8)

**Statut** : étude seulement. Aucun code moteur n'est modifié. Date : 2026-10-23.
**Lecture** : écrit pour quelqu'un qui n'est pas spécialiste (ni du son, ni du folklore).
Les mots techniques sont expliqués au premier emploi. Quand une chose n'a pas été vérifiée,
elle est marquée **[À VÉRIFIER]** ou **[INFERENCE]**.

## En bref

| Question | Réponse courte |
|---|---|
| Peut-on faire chanter le moteur **avec des paroles** ? | Pas sous la constitution actuelle, et pas en temps réel. Une banque SoundFont ne chante pas de mots, seulement des sons échantillonnés. |
| Peut-on faire de la **turlutte** (syllabes sans paroles) qui double la mélodie du reel ? | **Oui.** C'est la voie réaliste : une SoundFont de syllabes enregistrées par le propriétaire, un canal de plus dans le moteur, des règles de choix de syllabe en données de profil. Aucun amendement de la constitution. |
| Et les **voix « GM »** déjà dans `GeneralUser-GS.sf2` (programmes 52–54) ? | Ce sont des voyelles tenues (« aah », « ooh ») en boucle, sans consonnes. Bonnes pour un fond de chœur, pas pour de la turlutte. Mesures plus bas. |
| **Chanson à répondre** et **chanson de canot** ? | La forme (un meneur, le groupe répond) est faisable avec un algorithme d'appel-réponse générique (prévu par le principe V). Les **mots** ne le sont pas avec une SoundFont ; on peut chanter des **vocables** (« rouli roulant », « lon lon la »). |
| Les **droits** ? | Les recueils imprimés anciens (Gagnon 1865/1880, etc.) sont probablement du domaine public au Canada. Les **enregistrements** d'archives ne sont **pas** utilisables dans un jeu commercial sans autorisation écrite. On s'en sert comme **référence** ; tout son embarqué doit être enregistré par le propriétaire. |
| **Autochtone** ? | Rien. Ni corpus, ni chant, ni imitation. Principe VII : consultation des nations concernées avant tout. |

**Recommandation** (détail section 4) : premier pas = « turlutte sur le reel québécois » en SoundFont de
syllabes, mesurée au labo avant d'être écoutée, puis appel-réponse générique, puis canot.

---

## 1. Les trois formes

### 1.1 Turlutte (« giguer avec la bouche »)

**Ce que c'est.** Chanter des syllabes qui n'ont pas de sens (« tam ti delam », « tam tidelidelam »)
sur un air, au lieu de paroles. Le dictionnaire Usito définit *turluter* comme « chantonner un air sans
paroles en jouant sur des assonances de syllabes que l'on répète et fait rouler sur un rythme rapide »,
avec une citation de La Bolduc (1930) [[Usito]]. Le mot vient de *turlut*, « alouette » (1735).

**Structure.**

- Pas de couplet/refrain propre : la turlutte **prend la forme de l'air**. Sur un air de violon, c'est
  la forme du reel (AABB, parties de 8 mesures), ou celle de la gigue ou de la valse. [[Turlutage, Wikipédia]]
  la décrit comme « chanter des onomatopées sur des airs traditionnels de violon ».
- Deux usages se retrouvent dans les sources : (1) un air entier turluté à la place du violon ;
  (2) un **intermède** dans une chanson à paroles : le chanteur arrête les paroles et turlute
  un passage (résumé de l'article de Gilles Plante, Mnémo 2002 [[Mnémo]] ; page non lisible en entier
  ici, erreur 403 : **[À VÉRIFIER]**).
- Le propriétaire dit « giguer avec la bouche » : c'est l'usage (1).

**Rythme.** « Tempo enlevé » ; les turlutes « prennent souvent l'allure d'impossibles virelangues » ;
les mélodies sont « dansantes et joyeuses » [[Turlutage, Wikipédia]]. Notre reel québécois : 230 à la
noire, donc une croche = **130 ms**, soit environ 6 à 8 notes par seconde. Le reel rendu pour cette
étude (seed 3, 8 mesures) compte 53 notes en 8,3 s : 6,4 notes/s. Une syllabe par note est donc
plausible ; une voix tenue (chœur « aah », attaque lente) ne suit pas.

**Syllabes.** Variations sur quelques motifs (« tamtidelam » → « tam tidelidelam ») avec une palette
restreinte de sons : consonnes [t], [d], [m] ; voyelles [i] et [a] [[Turlutage, Wikipédia]]. C'est
bon pour nous : **8 à 10 syllabes suffisent** pour une première banque. Le choix de la syllabe suit la
position dans la mesure (temps fort, contretemps), la durée de la note et la répétition de la hauteur.
Cette règle de choix est **à mesurer** sur de vraies turluttes ; aucune distribution publiée n'a été
trouvée.

**Tessiture** (étendue des notes). Une voix chante environ une octave et demie. Les profils ont une
tessiture mélodique de 55 à 88 (sol3–mi6, un violon) ; le reel rendu va de 55 à 79. La turlutte doit
donc être **transposée d'une octave** et repliée dans l'étendue de la voix (le moteur replie déjà par
octave les notes qui sortent de la tessiture ; voir `profiles/README.md`).

**Lien avec les profils existants.** La turlutte n'a pas besoin d'un profil à elle : elle **double la
piste `Melody`** de `reel_quebecois`, `reel_acadien`, `gigue_quebecoise`, `valse_quebecoise`. Elle
ajoute une **piste de plus** (une voix) dont les notes dérivent de la mélodie (même hauteur −12, même
rythme) et dont chaque note reçoit une syllabe. Pas de nouveau profil de style. La marche britannique
(fifre, tessiture 72–102, hors voix) est exclue.

**Personne de référence.** La Bolduc (Mary Travers, 1894–1941) a rendu la turlutte célèbre dans les
années 1930 [[Canada.ca]] [[DCB]]. Le guide de la Library of Congress la cite aussi pour la **chanson à
dialogue** (deux chanteurs qui se répondent) [[LoC]].

### 1.2 Chanson à répondre

**Ce que c'est.** Un meneur chante, l'assemblée répond (reprend le refrain ou le couplet). Usito :
« chanson où le public, l'assemblée reprend en chœur le refrain ou un couplet d'abord chanté par un
chanteur » [[Usito chanson]]. Conrad Laforte classe l'essentiel de ces chansons dans les **chansons en
laisse** : « ces airs communément désignés au Canada *chansons à répondre* », « généralement à refrain »,
dont la forme de base est une suite de vers de même longueur sur la même assonance ; le refrain
« camoufle » souvent cette suite [[Arsenault, CJTM 1983]].
Exemples cités : *Trois beaux canards* (vers de 8 pieds) et *À la claire fontaine* (12 pieds).

**Structure (exemple vérifiable).** *En roulant ma boule* (variante acadienne, Beaton Institute) [[Beaton]] :

- un couplet = deux vers, chacun suivi d'un bout de refrain (« En roulant ma boule » ; « Rouli, roulant,
  ma boule roulant ») ;
- le **second vers d'un couplet devient le premier du suivant** (chaîne : « Trois beaux canards s'en
  vont baignant » termine le couplet 1 et ouvre le couplet 2) ;
- un refrain complet entre les couplets ;
- 6 couplets dans la version filmée (abrégée), jusqu'à une douzaine dans les versions longues
  (*V'là l'bon vent* sur la même page).
- Plus de 90 variantes canadiennes ont été relevées ; le texte est une chanson de jongleur du XVᵉ
  siècle adaptée en chanson d'aviron [[Beaton]].

Wikipédia décrit aussi la forme la plus simple : le meneur chante une phrase, le chœur la **répète**
[[Chant à répondre]] (**[À VÉRIFIER]** : article sans références suffisantes).

**Rythme et longueur.** Binaire ou ternaire selon la chanson ; **un couplet = 4 à 8 mesures**, la
chanson dure autant de couplets que le chanteur en sait **[INFERENCE]**. Le moteur peut produire cette
longueur variable : `form.sequence` et les rôles `CopyOf` existent déjà.

**Tessiture.** Voix parlée-chantée, une octave environ, au registre du chanteur ; le chœur à l'unisson
(ou à l'octave).

**Lien avec les profils.** Forme propre, **pas** AABB. Le principe V du manifeste prévoit justement
« appel-réponse » comme algorithme à écrire une fois en code générique. Un profil `chanson_a_repondre`
décrirait : une partie « meneur » (4 à 8 mesures), une partie « chœur » (copie ou refrain), la chaîne de
couplets. Ce qui manque au moteur : **deux voix distinctes** (meneur solo, chœur) ; voir section 3.

### 1.3 Chanson de canot / d'aviron

**Ce que c'est.** Chanson de voyageurs qui rythme la pagaie. *En roulant ma boule*, *À Saint-Malo*,
*À la claire fontaine* sont souvent citées ; James Huston (1848) soutenait que *À la claire fontaine* était
« l'œuvre des voyageurs des pays d'en haut qui l'entonnaient pour rythmer la cadence de leurs avirons »
[[Deschênes]] (affirmation d'époque, pas un fait établi). Presque toutes sont aussi des chansons à
répondre (même structure).

**Rythme.** La cadence vient de la pagaie : **45 à 55 coups par minute** [[Waukesha]], 55 selon
[[Voyageurs, Wikipédia]]. Un coup dure donc de 1,1 à 1,3 s. **[INFERENCE]** : un coup par pulse du chant
(donc 45–55 sur la pulse dansée), ce qui est **très lent** comparé au reel (115 pulses/min).

**Meneur.** Un chanteur mène en canot (documentation pédagogique des Lake County Forest Preserves, qui
note qu'il était le deuxième mieux payé de l'équipage [[Lake County]]) ; les pagayeurs répondent au
refrain. **[À VÉRIFIER]** sur source historique (Béland 1982, Barbeau).

**Corpus.** Madeleine Béland, *Chansons de voyageurs, coureurs de bois et forestiers* (91 chansons,
Presses de l'Université Laval, 1982) ; la collection Ermatinger (vers 1820), « très certainement les
premières chansons traditionnelles canadiennes-françaises à avoir été relevées » ; Marius Barbeau,
*En roulant ma boule* (posthume, 1982) [[Deschênes, CJTM 1983]].

**Lien avec les profils.** Même mécanique que la chanson à répondre, avec : tempo très bas (plage
45–55 pulses/min), pas de pieds, percussion de « coups de pagaie » (bois), pas de violon obligatoire
(voix et pagaies). **Attention** : l'aviron des voyageurs est aussi une histoire autochtone et métisse.
Ne retenir que le répertoire canadien-français documenté ; tout ce qui touche à une nation → consultation
(principe VII).

---

## 2. Corpus et droits

### 2.1 Ce que « droits » veut dire ici

Le labo applique **D1** : les fichiers sont téléchargés pour usage **interne**, jamais redistribués ;
le moteur ne reçoit que des statistiques [[plan DS]]. Pour la voix, D1 reste vrai pour **comprendre**
(structure, rythme, choix des syllabes). Mais un jeu qui **embarque** une SoundFont de syllabes
**distribue des sons** : il faut alors des droits clairs sur chaque son. Deux usages à ne pas mélanger :

| Usage | Exemple | Risque |
|---|---|---|
| **Référence** (écouter, transcrire, compter) | Mesurer quelles syllabes tombent sur quels temps | Faible : rien n'est redistribué |
| **Embarqué** (le son est dans le jeu) | Une syllabe échantillonnée d'une vieille bande | Fort : droits de l'enregistrement, de l'interprète, de la chanson |

### 2.2 Sources

| Source | Accès | Licence / conditions | Référence | Embarqué dans un jeu commercial |
|---|---|---|---|---|
| **Musée canadien de l'histoire** (fonds Marius Barbeau, plus de 72 000 enregistrements sonores du musée, dont des cylindres de cire) | Recherche en ligne ; consultation sur rendez-vous, préavis de 4 à 6 semaines [[CMH centre]] | Reproduction privée et non commerciale libre ; **distribution commerciale interdite sans permission écrite** (`permission@historymuseum.ca`) [[CMH droits]] | Oui, avec citation | **Non** sans permission écrite |
| **Archives de folklore et d'ethnologie, Université Laval** (fonds Lacourcière, Savard, etc. : plus de 5 000 enregistrements au départ ; en 1990, 8 700 bandes) [[Archives de folklore]] | Archives de l'université (aujourd'hui la Division de la gestion des documents administratifs et des archives [[Laval archives]]) ; modalités de consultation **non vérifiées** | **[À VÉRIFIER]** : droits des collecteurs et des informateurs (déposants) | Oui, sur demande | **Non** sans autorisation écrite des ayants droit **[INFERENCE]** |
| **Conrad Laforte, *Catalogue de la chanson folklorique française*** (6 volumes, PUL, 1977–87) | Livres, bibliothèques | Droit d'auteur des Presses de l'Université Laval | **Oui** : outil de classement (type, forme, refrain, nombre de variantes), sans audio | Sans objet (pas de son) ; ne pas recopier les notices |
| **BAnQ numérique** (partitions, enregistrements, imprimés patrimoniaux) | En ligne | Chaque document porte une mention : « Domaine public au Canada », licence Creative Commons ou déclaration RightsStatements ; usage commercial hors mention = **autorisation écrite** (délai de traitement 15 jours ouvrables) ; citer l'auteur et « Bibliothèque et Archives nationales du Québec » [[BAnQ CGU]] [[BAnQ droits]] | Oui | **Oui seulement** pour un document marqué « Domaine public au Canada » (ou licence compatible), et à condition de respecter la citation |
| **Smithsonian Folkways** | Écoute/achat | Usage personnel ; **licence à demander** pour reproduire (enregistrement maître + édition) [[Folkways licence]] ; conditions détaillées **non lues** (erreur 403) | Oui (écoute) | **Non** sans licence payante |
| **Library of Congress** (American Folklife Center : p. ex. collection Helen Creighton, chansons québécoises de Nouvelle-Écosse 1943–44) | Catalogue en ligne ; salles de lecture [[LoC]] | Variable selon collection **[À VÉRIFIER]** | Oui | **Non** par défaut |
| **Recueils imprimés du domaine public** : Ernest Gagnon, *Chansons populaires du Canada* (1865–67, 2ᵉ éd. 1880) ; collection Ermatinger (v. 1820) ; recueils anciens de Barbeau | Scans IMSLP [[IMSLP Gagnon]], Wikisource [[Wikisource Gagnon]], BAnQ | Gagnon est mort en 1915 : domaine public **[INFERENCE]**. Au Canada, les œuvres d'un auteur mort avant 1972 sont dans le domaine public (la prolongation à 70 ans, en vigueur le 30 décembre 2022, n'est **pas** rétroactive [[Cassels]]) : Barbeau (mort en 1969) et La Bolduc (1941) sont donc libres **au Canada** pour leurs **textes et partitions** | Oui | **Oui pour les textes et mélodies** (transcriptions anciennes). Pas les éditions posthumes récentes (ex. *En roulant ma boule*, 1982) ni les enregistrements : **[À VÉRIFIER]** par un juriste |
| **Enregistrements de La Bolduc (années 1930)** | Disques, rééditions | Chanson libre (voir ci-dessus) ; l'**enregistrement** a sa propre durée et ses propres titulaires (producteur, éditeur de la réédition) | Oui | **[À VÉRIFIER]** ; ne pas échantillonner |
| **Enregistrements du propriétaire** | Studio / maison | Entièrement à lui, avec formulaire de consentement des chanteurs | — | **Oui** (chemin conseillé) |

**Ce qu'il faut retenir.** Les *chansons* (mélodies, textes anciens) sont largement libres. Les
*enregistrements* d'archives et de labels ne le sont pas, même quand la chanson l'est : la bande a ses
propres droits. Le jeu doit donc embarquer **uniquement** des sons enregistrés par le propriétaire (ou
sous licence CC0/équivalente, comme les pieds de `harmonium_trad.sf2`).

### 2.3 Rien d'autochtone

**Le corpus voix n'inclut aucun chant, aucune syllabe, aucun enregistrement autochtone.** Principe VII :
pas de chants sacrés ou cérémoniels, pas de corpus sans droits clairs, validation par les nations
concernées avant qu'un style entre dans le jeu. Précisions utiles :

- Les collections de Barbeau au Musée canadien de l'histoire contiennent **des enregistrements
  huron-wendat et haudenosaunee** (résultat de recherche dans le catalogue du musée [[CMH Wendat]]) mêlés
  au répertoire canadien-français. Il faudra les **écarter explicitement** à toute extraction.
  Le musée rappelle lui-même de consulter les communautés pour ces contenus [[CMH droits]].
- Les chansons de voyageurs côtoient l'histoire autochtone et métisse. Si une chanson est rattachée à
  une nation ou à la nation métisse, elle sort du périmètre tant qu'il n'y a pas de consultation.
- Le lab applique déjà cette exclusion pour les airs de violon [[sources du lab]] ; la voix hérite de la
  même règle.

---

## 3. Techniques de rendu

Quatre options. Le moteur rend tout par **oxisynth** (licence LGPL-2.1, une SoundFont par « banque »,
canaux MIDI) ; les pieds suivent déjà ce chemin : `harmonium_trad.sf2`, `FEET_BANK_OFFSET = 1`,
chargement à la **construction** du moteur (jamais en plein jeu) [[voice_manager.rs]].

Mots utiles :

- **SoundFont (SF2)** : un fichier qui contient des échantillons (sons enregistrés) et des règles :
  pour chaque touche MIDI, quel échantillon jouer, de quelle façon. Elle joue des **notes**, pas des phrases.
- **Préset** : un « instrument » de la SoundFont, choisi par un numéro (programme).
- **Boucle (loop)** : un bout d'échantillon répété pour tenir une note longue.
- **Piste** : une des voix du moteur (`Melody`, `Chords`, `Bass`, `Heel`, `Toe`) ; chaque piste a son canal.

### 3.1 Tableau des options

| | **(a)** SF2 existantes (« aah/ooh », chœur) | **(b)** SF2 de syllabes enregistrées par le propriétaire | **(c)** Synthèse vocale chantée (paroles) | **(d)** Phrases entières pré-enregistrées |
|---|---|---|---|---|
| **Ce qu'on entend** | Voyelle tenue en boucle, sans consonnes | Syllabes courtes, avec attaque, au bon rythme | Paroles françaises intelligibles | Vraie voix, vraies paroles, vraie interprétation |
| **Qualité attendue** | Fond de chœur correct ; **turlutte non crédible** | **Turlutte crédible** si bien échantillonnée ; limites : monotonie des répétitions, formants qui changent avec la hauteur | Bonne à très bonne (modèles récents) mais « voix de synthèse » | La meilleure, mais **figée** |
| **Mots intelligibles ?** | Non | Non (vocables seulement) | Oui | Oui |
| **Suit le tempo du jeu ?** | Oui | Oui | Oui si temps réel (hors portée), non si pré-rendu | **Non** (le tempo bouge avec les jauges ; une phrase enregistrée à 230 ne suit pas) |
| **Coût moteur** | Faible : piste voix + programme du profil | Moyen : piste voix, règles syllabe/note en données, chargement de banque, snapshot étendu | Très élevé et incompatible (voir ci-dessous) | Moyen à élevé : lecteur d'échantillons hors SoundFont |
| **Coût lab / propriétaire** | Aucun | Séance d'enregistrement + pipeline de banque (extension de `feet-sf2`) | Licences de voix + outil externe | Enregistrements par air ; une version par tempo |
| **Droits** | GeneralUser GS : « sans restriction pour la création musicale privée ou commerciale », mais l'auteur précise que l'origine de certains échantillons est incertaine [[GeneralUser licence]] ; FluidR3 et MuseScore General : MIT [[manifeste SK3]] | Entièrement au propriétaire si enregistré par lui | Chaque moteur/voix a sa licence (voir 3.4) | Entièrement au propriétaire si enregistré par lui |
| **Constitution** (« Synthèse : SoundFont uniquement ») | **Conforme** | **Conforme** | **Non conforme** si le chant est synthétisé dans le jeu : **amendement nécessaire**. Conforme si le résultat est **gravé hors ligne** en SF2 | **Non conforme** s'il faut un lecteur d'échantillons ; **conforme** si une phrase tient dans un préset SF2 (une touche = une phrase), mais sans suivre le tempo |
| **Risque temps réel** | Aucun nouveau | Faible ; **à mesurer** : le `ProgramChange` émis par le playhead dans le callback ne doit pas allouer (`RTCheckAllocator`) | Inférence neuronale = allocations, CPU : impossible dans le callback | I/O disque interdite dans le callback : tout préchargé en mémoire |
| **Verdict** | Chœur de fond seulement | **Premier pas recommandé** | Hors portée ; pré-rendu seulement | Pour des moments scriptés du jeu, sans passer par le moteur |

### 3.2 (a) Les voix GM et les chœurs existants

**Ce que contient `GeneralUser-GS.sf2`** (inventaire fait pour cette étude, programmes 0-based) :

| Banque / programme | Nom | Échantillons réels | Observations |
|---|---|---|---|
| 0 / 52 | Concert Choir | 12 zones « Concert Choir-A2 … G5 », 28 kHz, 1,0 à 1,2 s chacune, boucle de 12 000 à 20 000 échantillons (0,45 à 0,73 s) | Chœur, instruments en double décalé (variante stéréo présumée **[INFERENCE]**) |
| 1 / 52 | Concert Choir Mono | Mêmes échantillons, une seule voie | Variante mono |
| 0 / 53 | Voice Oohs | 8 zones « Voice Oohs-G2 … C6 » (0,09 à 0,36 s) **plus** les 6 zones de « Synth Vox Aahs » | Échantillons très courts, **boucles de 0,15 à 0,19 s** (risque de battement audible) |
| 0 / 54 | Synth Voice | 6 zones « Synth Vox Aahs-G3 … B5 », 28,8 kHz, 0,74 à 1,47 s | Synthétique |
| 0 / 85 | Solo Vox | 7 zones « Solo Vox-G3 … A#7 », 11 à 22 kHz, 0,17 à 0,31 s | Voix solo, boucles de 0,16 à 0,30 s |
| 0 / 91 | Space Voice | 7 zones « Solo Vox » + « Bell Organ » | Fond « spatial » |

**Lecture simple** : ce sont des **voyelles** (« aah », « ooh ») enregistrées une fois puis bouclées.
Il n'y a **aucune consonne** : impossible de faire « ta », « di », « lam ».

**Mesures** (fluidsynth, réverbération et chorus désactivés, vélocité 100, enveloppe lissée sur 5 ms,
notes de 130 ms = croche de reel, ou 1 s ; attaque = temps de 10 % à 90 % de la crête) :

| Préset | Attaque, note 130 ms (do4 / sol4 / do5) | Attaque, note 1 s | Relâchement après la fin de la note |
|---|---|---|---|
| Violon (40), pour comparaison | 96 / 111 / 63 ms | 415 / 319 / 445 ms | 80 à 100 ms |
| Concert Choir (52) | 99 / 93 / 118 ms | 419 / 226 / 341 ms | 100 à 300 ms |
| Voice Oohs (53) | 32 / 17 / 38 ms | 32 / 17 / 130 ms | 80 à 140 ms |
| Synth Voice (54) | 35 / 24 / 120 ms | 263 / 174 / 125 ms | 115 à 270 ms |
| Solo Vox (85) | 27 / 26 / 70 ms | 27 / 927\* / 72 ms | ~30 ms |

\* valeur aberrante de la mesure (sol4, 1 s) ; non interprétée.

**Interprétation.** Pour une croche de 130 ms, le Concert Choir met presque autant de temps à monter que
la note n'en dure : il joue comme un nuage plutôt que des syllabes. Voice Oohs et Solo Vox attaquent
vite, mais ce sont des voyelles bouclées sans consonne, donc aucun effet de syllabe. **Mesure seulement
des durées ; le jugement à l'oreille reste à faire par le propriétaire** (rendus ci-dessous).

**Rendus pour l'écoute** (8 mesures du moteur, seed 3, `--feet off`) : voir section 5. Dans les rendus
« doublé » (violon + voix), le chœur sur la mélodie du reel s'entend comme un **épaississement du violon**
**[INFERENCE : non jugé à l'oreille par l'auteur du rapport]**.

**Droits.** GeneralUser GS (S. Christian Collins) : « You may use GeneralUser GS without restriction for
your own music creation, private or commercial », avec la réserve de l'auteur sur l'origine incertaine de
certains échantillons [[GeneralUser licence]] ; le labo l'a déjà classée « acceptée avec risque »
[[manifeste SK3]]. FluidR3 GM (MIT) et MuseScore General (MIT) sont acceptées par le lab pour le violon ;
leurs chœurs n'ont pas été inventoriés ici **[À VÉRIFIER]**.

**Ce que le moteur devrait ajouter.** Peu : une piste `Voice` (canal MIDI propre, même banque 0), un
programme dans `[instruments]` (p. ex. `voice_program = 52`). C'est un « doublage de fond » sans
syllabes.

### 3.3 (b) SoundFont de syllabes enregistrées par le propriétaire

**Idée.** Même démarche que les pieds : le propriétaire enregistre, le labo prépare, une SF2
`assets/harmonium_voix.sf2` est chargée à côté des autres. Le pipeline des pieds
(`make trad-sf2`, écrivain SF2 de la bibliothèque standard) est déjà là [[feet/sf2.py]] ; il écrit
aujourd'hui des frappes sans boucle sur une touche fixe. Il faut y ajouter : plusieurs **hauteurs** par
syllabe (les sons ne se transposent bien que sur quelques demi-tons), des **boucles** pour les voyelles
tenues, et un **préset par syllabe**.

**Taille de la banque.** 8 syllabes × 6 hauteurs × 2 intensités × 0,3 s à 44,1 kHz (16 bits) = 96
échantillons de 26 Ko ≈ **2,5 Mo**. Petit.

**Pourquoi un préset par syllabe ?** Une note MIDI ne porte que la **hauteur** et la vélocité. Il faut
donc un autre moyen de dire *quelle syllabe*. Options :

1. **Un préset par syllabe + changement de programme** avant chaque note : utilise l'événement
   `AudioEvent::ProgramChange` qui existe déjà. La règle de choix de la syllabe est faite à la
   génération (RNG injecté, principe II). **Choix recommandé** : pas de nouveau chemin.
2. Un canal par syllabe : le moteur n'a que 16 canaux (0–4 utilisés, 9 = percussions GM) ; peu extensible.
3. Couches de vélocité (une syllabe par plage de vélocité, comme les pieds) : détruit la dynamique de
   la voix. À écarter.

**Ce que le moteur devrait ajouter** (principe V : données de profil + algorithme générique) :

- `TrackId::Voice` (canal 5) ; la mixette `[mix]` (point 1) lui donne un niveau et un panoramique.
- Section `[voice]` du profil : suit quelle piste (`follows = "Melody"`), décalage d'octave, étendue de
  la voix, table des syllabes (nom → programme SF2), **règles d'attribution** (par position dans la
  mesure, durée de la note, répétition de la hauteur, avec poids) ; tirage par le `ChaCha8Rng` du générateur.
- **Alignement syllabe/note** : une syllabe par note (V1). Les notes très courtes reçoivent une syllabe brève,
  les notes longues une syllabe à voyelle tenue (boucle). Notes liées → même syllabe prolongée.
- Le snapshot porte la syllabe choisie (`TimelineNote` : nouveau champ) → **rupture de contrat** :
  `TUNING_VERSION` incrémenté et goldens régénérés dans le même commit (principe I). Une alternative
  sans toucher le snapshot (choisir la syllabe dans l'hôte) est refusée : le tirage doit rester sur le
  `ChaCha8Rng` du générateur (principe II) et la syllabe doit faire partie du contrat publié.
- L'export MIDI doit contenir les `ProgramChange` du canal voix, pour qu'un rendu externe
  (fluidsynth du lab) entende la même chose.

**Qualité attendue [INFERENCE].** Une SoundFont est un **échantillonneur** : chaque note rejoue un son
enregistré, accéléré ou ralenti pour changer de hauteur. Plus on s'éloigne de la hauteur d'origine, plus
la voix « change de taille » (effet « voix de dessin animé »). Contre-mesure : enregistrer chaque
syllabe à 5 ou 6 hauteurs sur ~1,5 octave (le coût ci-dessus). Les répétitions identiques s'entendent ;
2 ou 3 prises par syllabe/hauteur, en rotation, limitent l'effet « mitraillette ».

**Risques temps réel.** Aucun en soi (même mécanisme que les pieds). À vérifier : que le changement de
programme sur le canal voix n'alloue pas dans le callback (test `RTCheckAllocator` en debug). La banque
doit être chargée à la construction (comme `FEET_BANK_OFFSET`), **pas** par `AudioEvent::LoadFont` pendant
le jeu.

**Droits.** Complets si le propriétaire chante et signe un consentement ; aucune ambiguïté d'archive.

### 3.4 (c) Synthèse vocale chantée (paroles)

**Idée.** Un logiciel chante des paroles à partir de notes et de texte.

| Outil | Licence | Remarques |
|---|---|---|
| **Synthesizer V** (Dreamtonics/AHS) | Licence propriétaire. Son EULA : « you may use the synthesized sound generated by using this product, whether commercial or non-commercial, free of charge » (art. 10) ; **mais** interdit de distribuer « any compilation of audio rendered by the product in isolation that facilitates the creation of derived works » (art. 13(8) et 13(13)) ; intégrer le produit à un logiciel commercial exige de les contacter (art. 12) [[SynthV EULA]] | Une banque de **syllabes** rendues puis mises en SF2 est une compilation de ce genre : **à ne pas faire**. Un morceau chanté complet n'a pas ce problème. |
| **VOCALOID6** | Licence propriétaire (EULA à lire avant tout usage commercial) [[Vocaloid EULA]] | Même question de redistribution **[À VÉRIFIER]** |
| **DiffSinger (OpenVPI)** | Code Apache-2.0 [[DiffSinger]] | Les **modèles de voix** ont chacun leur licence ; voix française libre non trouvée **[À VÉRIFIER]** |
| **NNSVS** | Code MIT [[NNSVS]] | Même remarque |
| **Sinsy** | Modified BSD [[Sinsy]] | Modèle de voix HMM, qualité datée, voix principalement japonaises **[À VÉRIFIER]** |

**Conséquences pour le moteur.**

- Un modèle neuronal en direct dans le callback : allocations, calcul lourd, I/O. **Impossible** sous le
  principe III.
- Chanter dans le jeu = ne plus être « SoundFont uniquement » : **amendement de la constitution**
  nécessaire (contrainte « Synthèse : SoundFont uniquement »), avec un nouveau moteur de rendu, dans
  le callback ou hors ligne.
- Seule variante **sans amendement** : pré-rendre hors ligne des syllabes ou des phrases, puis les mettre
  en SF2/échantillons (équivalent de (b) ou (d), avec les restrictions de licence ci-dessus).

**Verdict.** Utile, éventuellement, comme **outil de studio** pour produire un morceau chanté (option d),
pas comme moteur. Ne pas amender la constitution pour cela.

### 3.5 (d) Phrases entières pré-enregistrées

**Idée.** Enregistrer une chanson complète (ou un couplet) avec paroles, la rejouer.

- **Problème du tempo.** Le tempo du moteur bouge avec les jauges (le profil donne 220–240 à la noire pour
  le reel). Une phrase enregistrée à 230 ne peut pas être accélérée sans changer la hauteur et le grain de
  la voix, car une SoundFont n'étire pas le temps. Il faudrait enregistrer plusieurs tempos ou limiter
  l'usage à des **scènes à tempo fixe**.
- **Problème de la mélodie.** La mélodie est générée : une voix pré-enregistrée chanterait une autre
  mélodie. Seules des chansons à air fixe (chanson à répondre dont l'air est connu) s'y prêtent, et alors
  la voix **est** la mélodie, pas un doublage.
- **Faisabilité.** SF2 : une touche = une phrase (comme un pied) : possible, sans suivi de tempo. Lecteur
  d'échantillons hors SF2 : amendement. Dans les deux cas, le chemin le plus simple est de **jouer le son
  côté jeu** (Bevy), hors du moteur.
- **Risque temps réel.** Aucun si le son est préchargé en mémoire ; pas de lecture de fichier dans le callback.

**Verdict.** Pour des scènes scénarisées (veillée, canot au générique) : à faire côté jeu. Pas dans le moteur.

---

## 4. Recommandation

### 4.1 Premier pas réaliste et mesurable : turlutte sur le reel québécois

Une voix qui **double la mélodie** de `reel_quebecois` avec des syllabes de turlutte, en SoundFont (b).

**Pourquoi celui-là.**

- C'est la forme la plus proche de l'existant : aucune nouvelle structure de style, une piste de plus.
- C'est la seule option qui donne un résultat **crédible** sans paroles, **sans amendement**.
- Il est **mesurable avant l'écoute** : alignement syllabe/note, niveau par rapport au violon, variété.
- Il réutilise tout : pipeline lab (pieds), banque chargée comme `FEET_BANK_OFFSET`, `[mix]` (point 1),
  hétérophonie (point 4, une 2ᵉ voix qui suit la ligne).

**Critères d'acceptation observables.**

1. Chaque note de la piste voix tombe **à moins de 15 ms** (seuil proposé) du début d'une note de la
   mélodie (mesuré sur le WAV, détection d'attaque, comme pour les pieds).
2. Niveau de la voix **entre 6 et 10 dB sous le violon** (cible proposée) au mélange, réglé via `[mix]`,
   pas de saturation.
3. Variété : au moins 3 syllabes différentes par 8 mesures ; aucune syllabe plus de 4 fois de suite ;
   mêmes règles d'attribution aux mêmes positions (déterminisme : même seed ⇒ même WAV octet pour octet).
4. Écoute à l'aveugle : turlutte SF2 contre chœur GM (option a) sur 3 airs ; le propriétaire dit laquelle
   sonne « bouche qui gigue ».
5. Aucune allocation dans le callback (`RTCheckAllocator`).

### 4.2 Tâches suivantes (dans l'ordre)

| # | Tâche | Où | Dépend de |
|---|---|---|---|
| V0 | **Propriétaire** : enregistrer le kit de syllabes (voir 4.3) | `harmonium_lab/samples/voice/own/` | — |
| V1 | `voice-prepare` / `voice-sf2` : banque `assets/harmonium_voix.sf2` (préset par syllabe, plusieurs hauteurs, boucles de voyelle), aperçu par fluidsynth | lab | V0 ; extension de `feet/sf2.py` |
| V2 | `[voice]` dans `StyleProfile` (suit `Melody`, octave, étendue, table des syllabes, règles de choix) + `TrackId::Voice` + champ syllabe dans `TimelineNote` ; **bump `TUNING_VERSION` et goldens** | `harmonium_core` | V1 (noms des syllabes) |
| V3 | Chargement de la banque voix (`VOICE_BANK_OFFSET`), `ProgramChange` avant `NoteOn`, entrée `[mix]` de la piste Voice, export MIDI avec les changements de programme | `harmonium_audio`, `harmonium_host` | V2, point 1 (mix) |
| V4 | Mesure lab : alignement, niveau, variété, comparaison au chœur GM ; **puis** écoute du propriétaire | lab | V3 |
| V5 | **Appel-réponse générique** : deux voix (meneur : voix solo ; chœur : préset de chœur) ; refrain en **vocables** ; profil `chanson_a_repondre` ; statistiques de forme tirées de recueils **du domaine public** (Gagnon) transcrits en ABC | core + profil + lab | V3 |
| V6 | **Chanson de canot** : profil lent (45–55 pulses/min), percussion « pagaie » (enregistrée), même mécanique que V5 | profil | V5 |
| V7 | Option (d) côté **jeu** pour les scènes scénarisées avec paroles ; aucune tâche moteur | jeu | — |

V5 et V6 sont **conditionnées** aux décisions du propriétaire (4.3) et à la validation des droits de
chaque chanson utilisée. La turlutte (V0–V4) n'a besoin ni des archives ni d'un amendement.

### 4.3 Ce que le propriétaire doit fournir ou décider

**À fournir (V0).**

- Enregistrements propres, mono 44,1 kHz ou 48 kHz, sans réverbération, sans musique en fond :
  - 8 à 10 syllabes (suggestion de départ : `ta`, `ti`, `di`, `li`, `lam`, `lon`, `dle`, `la`),
    chacune **courte** (~0,3 s) et **tenue** (voyelle ~1 s, pour les notes longues) ;
  - à **5 ou 6 hauteurs** sur une octave et demie, à **2 intensités** ;
  - 2 ou 3 prises par cas, pour varier ;
  - plus quelques groupes de syllabes enchaînées « tam ti delam » (pour la musique des transitions) ;
  - 4 à 6 interjections de chœur (pour V5) si possible.
- **Consentement écrit** de toute personne qui chante (cession des droits sur la prise pour le jeu,
  durée, territoire, usage commercial).

**À décider.**

1. **Pas d'amendement de la constitution** (recommandé) : le chant avec paroles reste hors du moteur ;
   la voix du jeu se limite aux **vocables** (turlutte, refrains), les chansons à paroles passent par le
   jeu (option d). Un amendement ne serait justifié que pour une **synthèse vocale en direct** ; ce
   n'est pas recommandé (3.4).
2. **Archives** : demander ou non une autorisation écrite au Musée canadien de l'histoire et aux Archives
   de folklore de l'Université Laval pour **étudier** (référence, pas d'embarquement). Aucune autorisation
   n'est requise pour lire les recueils du domaine public.
3. **Chanson** à retenir pour V5 et V6 : choisir parmi des chansons dont le texte et l'air sont du domaine
   public **et** hors tout contexte autochtone/métis.
4. **Consultation** : si une chanson touche une nation autochtone ou métisse, activer le processus du
   principe VII **avant** tout travail.

---

## 5. Ce qui a été vérifié en pratique

**Aucun code du dépôt n'a été modifié.** Les scripts jetables (mido, librosa, fluidsynth) ont tourné
dans `/tmp/v8voix/` ; ils sont archivés tels quels dans [`etudes/voix/`](etudes/voix/) (chemins `/tmp`
codés en dur) et les rendus d'illustration de `harmonium_lab/target/voix_etude/` ont été effacés.

### 5.1 Méthode

1. Compilation du CLI : `cargo build -q -p harmonium-cli --release` (worktree neuf).
2. Export de 8 mesures, seed 3, sans pieds, avec la banque GM :

   ```text
   ./target/release/harmonium-cli --export --bars 8 --style <profil> --seed 3 --feet off \
     --soundfont /Users/william.quintal/Project/bascanada/harmonium/assets/GeneralUser-GS.sf2 \
     --record-midi <dossier>/<profil>.mid --record-wav <dossier>/<profil>_gm_complet.wav
   ```
3. Mélodie extraite (canal MIDI 0). Reel : mélodie seule (les accords entrent aux tours suivants dans
   l'arrangement ; le MIDI du reel ne contient que le canal 0 sur ces 8 mesures). Programme de la
   mélodie remplacé par 52, 53, 54 puis 85 ; MIDI réécrit avec `mido`.
4. Rendu : `fluidsynth -ni -q -F <sortie>.wav -r 44100 GeneralUser-GS.sf2 <entrée>.mid` (réverbération et
   chorus par défaut de fluidsynth actifs pour les rendus d'écoute ; désactivés pour les mesures du 3.2).
5. Versions « doublées » : la mélodie copiée sur le canal 3, programme 52 ou 53, volume (CC 7) = 80,
   violon, accords et basse conservés.

**Limites.** L'auteur du rapport n'a **pas écouté** les fichiers (pas de sortie audio). Seules des
mesures (attaques, durées, niveaux) et l'inventaire du contenu de la SoundFont sont rapportés ; le
jugement de qualité revient au propriétaire. Le rendu utilise fluidsynth, pas oxisynth du moteur ;
les deux appliquent la même SoundFont, mais de petites différences de rendu sont possibles
**[À VÉRIFIER si un écart est audible]**.

### 5.2 Fichiers (hors dépôt)

Dossier : `/Users/william.quintal/Project/bascanada/harmonium_lab/target/voix_etude/`
(`target/` du lab, ignoré par git ; peut être effacé par `make clean`).

| Fichier | Contenu |
|---|---|
| `reel_quebecois.mid`, `valse_quebecoise.mid`, `gigue_quebecoise.mid` | Export MIDI du moteur, 8 mesures, seed 3 |
| `*_gm_complet.wav` | Rendu du moteur (oxisynth, banque GM), tel quel |
| `*_ref_violon.wav` | Mélodie seule, violon (programme 40), fluidsynth |
| `*_voix_choir_aahs.wav` | Mélodie seule sur Concert Choir (52) |
| `*_voix_voice_oohs.wav` | Mélodie seule sur Voice Oohs (53) |
| `*_voix_synth_voice.wav` | Mélodie seule sur Synth Voice (54) |
| `*_voix_solo_vox.wav` | Mélodie seule sur Solo Vox (85) |
| `*_complet_double_choir_aahs.wav` | Moteur complet + même mélodie sur Concert Choir (canal 3) |
| `*_complet_double_voice_oohs.wav` | Moteur complet + même mélodie sur Voice Oohs (canal 3) |
| `reel_quebecois_turlutte_proto_syllabes_say_voix_seule.wav`, `…_avec_violon.wav` (et `gigue_quebecoise_…`) | **Maquette d'alignement** (5.3), pas de la qualité finale |

(`*` = `reel_quebecois`, `valse_quebecoise`, `gigue_quebecoise`.) Niveaux bruts des rendus fluidsynth :
crêtes de −23 à −28 dBFS ; ce sont des niveaux d'écoute à monter, pas un mélange final.

### 5.3 Maquette « turlutte » d'alignement (jetable)

Pour voir si la **mécanique** (une syllabe par note, choix par position) donne un résultat net, un
script jetable remplace les notes du canal 0 par des syllabes « ta, ti, di, li, lam » générées avec la
voix système macOS « Eddy (Français (Canada)) », transposées d'une octave plus bas par rééchantillonnage
(comme le ferait une SoundFont). Règles : temps fort et note longue → `lam` ; temps fort → `ta` ;
contretemps → `di` (ou `ti` si la hauteur répète) ; autres → `li`.

**Cette maquette n'est pas une preuve de qualité** : la voix source est de la parole synthétique
(grave, ~100 Hz), non chantée, dont la licence d'emploi dans un jeu n'a pas été vérifiée (ne rien en
embarquer) ; elle sert seulement à vérifier
que le choix de syllabe et le calage sur les notes fonctionnent. Résultat : le script produit une
syllabe par note, soit 53 syllabes sur 8 mesures de reel (38 sur la gigue), posées sur les débuts de
note MIDI. La qualité réelle dépendra des enregistrements du propriétaire.

### 5.4 Constats utiles pour la suite

- Les voix GM sont des voyelles tenues ; aucune syllabe → option (a) = fond de chœur.
- Le choix recommandé (préset par syllabe) utilise un événement existant (`ProgramChange`) : peu de
  nouveau code côté audio.
- Le moteur passe par le contrat de snapshot (principe I) : ajouter une syllabe à une note est un
  changement de contrat → bump de version et goldens à régénérer.
- Les pieds sont la référence pour le pipeline : chargement à la construction, banque à décalage,
  mixette par piste.

---

## Sources

Les renvois `[[Nom]]` du texte pointent vers les définitions de liens ci-dessous.

- Usito, « turluter » et « chanson » (chanson à répondre).
- Wikipédia, « Turlutage » (et version anglaise « Turlutte (music) »).
- Gilles Plante, « Qu'est-ce que la turlutte », *Bulletin Mnémo*, vol. 7, no 3, automne 2002
  (403 à la lecture ; résumé par les extraits de recherche).
- Library of Congress, *Québec: French Culture, First Nations & Folk Music*, « Québécois Music ».
- Canada.ca et *Dictionnaire biographique du Canada* sur Mary Travers « La Bolduc ».
- Georges Arsenault (sur Laforte) et Donald Deschênes (sur Barbeau), *Canadian Journal for Traditional
  Music*, 1983 ; Rika Ruebsaat (sur Béland), même numéro.
- Wikipédia, « Chant à répondre » ; Beaton Institute, « En Roulant Ma Boule » ; Gaston Deschênes,
  « Hymnes et chants nationaux (1) ».
- Waukesha County Museum ; Lake County Forest Preserves ; Wikipédia, « Voyageurs ».
- Musée canadien de l'histoire : centre de recherche, droits d'auteur, notice huron-wendat.
- *L'Encyclopédie canadienne*, « Archives de folklore » ; Université Laval, archives.
- BAnQ : conditions générales d'utilisation, déclarations de droits.
- Smithsonian Folkways, licences (403 à la lecture) ; IMSLP et Wikisource pour Gagnon ; Cassels sur la
  durée du droit d'auteur au Canada.
- Licence de GeneralUser GS ; manifeste SK3 du lab (GeneralUser, FluidR3 MIT, MuseScore General MIT,
  Sonatina CC Sampling Plus 1.0 ; Arachno et autres rejetées) ; contrats de licence Synthesizer V et
  VOCALOID6 ; dépôts DiffSinger (Apache-2.0), NNSVS (MIT), Sinsy (Modified BSD).
- Documents internes : plan DS du lab (D1), sources du lab (source 12 : musiques autochtones exclues),
  `voice_manager.rs` (`FEET_BANK_OFFSET`, `add_font`), `harmonium_core/src/events.rs`
  (`AudioEvent::ProgramChange`), `harmonium_core/src/timeline/mod.rs` (`TrackId`, `TimelineNote`),
  `feet/sf2.py`.

[Usito]: https://usito.usherbrooke.ca/d%C3%A9finitions/turluter
[Usito chanson]: https://usito.usherbrooke.ca/d%C3%A9finitions/chanson-th%C3%A8me
[Turlutage, Wikipédia]: https://fr.wikipedia.org/wiki/Turlutage
[Mnémo]: https://mnemo.qc.ca/bulletin-mnemo/article/quest-ce-que-la-turlutte
[LoC]: https://guides.loc.gov/quebec/quebecois-music
[Canada.ca]: https://www.canada.ca/en/women-gender-equality/commemorations-celebrations/women-impact/arts/mary-rose-anna-travers.html
[DCB]: http://www.biographi.ca/en/bio/travers_mary_17E.html
[Arsenault, CJTM 1983]: https://cjtm.icaap.org/content/11/v11art6.html
[Deschênes, CJTM 1983]: https://cjtm.icaap.org/content/11/v11art6.html
[Chant à répondre]: https://fr.wikipedia.org/wiki/Chant_%C3%A0_r%C3%A9pondre
[Beaton]: https://beatoninstitutemusic.ca/acadian/en-roulant-ma-boule/
[Deschênes]: https://blogue.septentrion.qc.ca/gaston-deschenes/2023/03/29/hymnes-et-chants-nationaux-1
[Waukesha]: https://waukeshacountymuseum.org/wp-content/uploads/2024/09/Voyageur-Song-Canoe-Activity.pdf
[Voyageurs, Wikipédia]: https://en.wikipedia.org/wiki/Voyageurs
[Lake County]: https://www.lcfpd.org/assets/1/28/Lives-of-the-Voyageurs-Teacher-Info-Packet.pdf
[CMH centre]: https://www.historymuseum.ca/research-and-collections/canadian-history-research-centre
[CMH droits]: https://www.historymuseum.ca/copyright
[CMH Wendat]: https://www.historymuseum.ca/collections/archive/3197498
[Archives de folklore]: https://thecanadianencyclopedia.ca/en/article/archives-de-folklore-emc
[Laval archives]: https://www.archives.ulaval.ca/
[BAnQ CGU]: https://www.banq.qc.ca/conditions-generales-dutilisation/
[BAnQ droits]: https://www2.banq.qc.ca/outils/declarations_droits_licences/
[Folkways licence]: https://folkways.si.edu/license-request
[IMSLP Gagnon]: https://imslp.org/wiki/Chansons_populaires_du_Canada_(Gagnon,_Ernest)
[Wikisource Gagnon]: https://fr.wikisource.org/wiki/Auteur:Ernest_Gagnon
[Cassels]: https://cassels.com/insights/canada-extends-term-of-copyright-in-works-to-life-of-author-plus-70-years
[GeneralUser licence]: https://raw.githubusercontent.com/mrbumpy409/GeneralUser-GS/main/documentation/LICENSE.txt
[manifeste SK3]: file:///Users/william.quintal/Project/bascanada/harmonium_lab/samples/fiddles/manifest.jsonl
[SynthV EULA]: https://www.ah-soft.com/synth-v/eula_e.html
[Vocaloid EULA]: https://s3.ap-northeast-1.amazonaws.com/download.vocaloid/license/VOCALOID6/LICENSES_VOCALOID6_ENG.PDF
[DiffSinger]: https://github.com/openvpi/DiffSinger
[NNSVS]: https://github.com/nnsvs/nnsvs
[Sinsy]: https://sinsy.sourceforge.net/
[plan DS]: file:///Users/william.quintal/Project/bascanada/harmonium_lab/specs/001-trad-dataset/plan.md
[sources du lab]: file:///Users/william.quintal/Project/bascanada/harmonium_lab/corpus/sources.md
[voice_manager.rs]: ../../harmonium_audio/src/voice_manager.rs
[feet/sf2.py]: file:///Users/william.quintal/Project/bascanada/harmonium_lab/src/harmonium_lab/feet/sf2.py
