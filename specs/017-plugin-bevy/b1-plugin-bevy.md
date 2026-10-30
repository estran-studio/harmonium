# B1 — Plugin Bevy mince

## Livraison

`harmonium_bevy`, membre du workspace hors `default-members`, dépend de Bevy **0.19.1** avec ses options par défaut désactivées. La bibliothèque n'active que `std`, l'exécuteur asynchrone, les assets et les journaux. L'option `demo` ajoute la fenêtre, l'UI et le rendu ; aucune option n'active `bevy_audio`. `odin` propage exactement `harmonium/odin` et `harmonium_audio/odin`, désactivées par défaut. Licence héritée du workspace ; un binaire avec Odin est GPL.

Fichiers de la livraison : manifeste et verrou Cargo, `harmonium_bevy/{src,tests,examples,README.md}`, cibles Makefile et présent compte rendu. Aucun changement dans le studio, la phase 015 ou alacod.

## Décisions

- **Jeu → moteur** : `HarmoniumControl`, ressource de cibles (profil distribué ou `StyleProfile`, quatre émotions, volume, pause, graine), consommée dans `PostUpdate` / `HarmoniumSet`. Les commandes passent par `LiveHost::apply` ; aucune règle musicale dans le plugin.
- **Assets** : un chargeur Bevy de données binaires pour `.sf2`, `.sf3`, `.odin`. Noms des presets = fichiers sans extension. Toutes les ressources déclarées doivent être chargées avant d'ouvrir l'hôte ; échec terminal visible par `HarmoniumStatus`. Les avis `missing_resources()` restent ceux de l'hôte, sans seconde journalisation par le plugin.
- **Fil principal** : `HarmoniumRuntime` non-Send possède un `Rc<RefCell<LiveHost>>` et le lecteur d'horloge public. Pas de mutex sur le callback audio. Le partage local est nécessaire au pompage web hors rendu et à la reprise synchrone dans le geste DOM.
- **Réserve et cadence** : 16 mesures, pompage natif borné à 50 ms par défaut, Worker web à 100 ms. Pas de rafale après une frame lente. Les mètres des mesures composées sont conservés dans une fenêtre bornée, y compris si les frames web s'arrêtent mais que le Worker continue.
- **Émotions** : seuil 0,02 depuis la dernière valeur envoyée, cadence maximale d'une commande par 100 ms de `Time<Real>`. La petite dérive s'accumule, les quatre axes sont envoyés ensemble. Une cible pendant un redémarrage attend la publication de la nouvelle session : invalider avant le callback aurait repris la génération depuis l'ancien playhead.
- **Fondus** : durée configurée par demi-fondu. Descente → commande de volume zéro acquittée, gain rendu zéro puis tampon suivant entièrement silencieux → profil/émotions/graine conservée → remontée. Une cible changée pendant la descente remplace l'attente ; pendant la montée, une nouvelle descente part du gain courant. Pas d'installation anticipée sur la seule horloge UI.
- **Nouvelle forme** : `SetSeed` avec la graine cible est la dernière invalidation du changement de profil. `Seek(1)` seul conserve le passé composé ; il aurait rejoué l'ancienne première mesure. `SetStyleProfile` seul conserve la mesure entendue : elle ne doit pas être réentendue pendant la remontée.
- **Horloge** : `MusicClock` vient du `TransportClock` public et de `BeatInfo`, non de l'horloge du jeu. `beat` est à la noire, `pulse` dansée ; le tempo entendu utilise la durée audio et le mètre effectivement composé. `Beat` et `Bar` sont des messages Bevy, sans doublon sur une frame sans nouveau tampon ni sur une reprise. Un redémarrage invalide l'horloge jusqu'à une publication de la mesure 1 ; pas de faux rattrapage de la session abandonnée.
- **Navigateur** : ouverture en pause ; reprise synchrone au premier geste après chargement (`pointerdown`, `keydown`, `touchend`) quand la cible demande la lecture. Worker et écouteurs détruits avec le runtime. Les gestes antérieurs au chargement ne sont pas mémorisés.
- **Démo** : sources Bevy natives nommées `banks://` et `odin://`, sans copie. Banques du checkout principal comme `STUDIO_BANKS` ; presets plats via `HARMONIUM_ODIN`. HUD français et témoin pulsé par `Beat`. Le mode `--smoke-seconds` est explicitement muet, le lancement normal joue pour l'écoute du propriétaire.

## Ajustements minimaux de l'hôte

Deux limites de l'API existante empêchaient une garantie correcte de transition ; pas de nouveau chemin de pilotage musical.

1. **Acquittement du silence** : une progression de `buffer_start_frame` ne prouve pas que le callback a consommé `SetMasterVolume`, notamment lorsque la contrepression Odin retient les commandes. `LiveHost::volume_revision()` expose la révision de la dernière commande acceptée ; `TransportClock::{volume_revision, output_gain}` publie son acquittement et le gain atteint. `PlaybackCommand::SetMasterVolume` porte cette révision, son seul appelant de test direct est migré. Publication à la fin du tampon, payload `Copy`, sans allocation ni verrou. Le plugin n'a plus de temporisation fondée sur une copie de la constante privée de rampe.
2. **Saut dur sur la mesure déjà chargée** : le playback conservait cette boîte comme pour un saut doux. Reproduction par le plugin, profil reel 2/2 → désert 4/4 pendant la mesure 1 : profil commandé désert mais horloge rendue toujours à deux pulses. Le saut dur décharge maintenant aussi la mesure cible et invalide sa pulse jusqu'au rendu de la remplaçante ; le saut doux conserve son comportement de rejeu. Le test de régression vérifie les quatre pulses réellement rendues, pas seulement `profile()`.

L'interface `EngineCommand` ne change pas. Aucun changement de génération, de profils, de banques musicales ou de RNG. Le studio et le REPL gardent la même API de haut niveau.

## Limites assumées

- Transition sans raccord harmonique, avec redémarrage de forme ; J3 n'est pas implémenté ici.
- `LiveHost::start_paused` analyse les instruments sur le fil principal après le chargement asynchrone des octets. Pas de rechargement à chaud d'instruments ni de sélection automatique des ressources depuis le profil.
- Une pause sans callback ne peut pas acquitter un nouveau volume : la transition attend la reprise, même avec une durée de fondu nulle.
- Un fil principal natif bloqué ou un onglet complètement gelé peut épuiser les 16 mesures ; le Worker évite seulement la dépendance à `requestAnimationFrame`. CSP web : autoriser les Workers Blob.
- La validation auditive appartient au propriétaire. La vérification automatisée rend les tampons hors ligne et lance la démo à volume zéro ; aucune écoute revendiquée.

## Preuves de validation

- `cargo test -p harmonium_bevy` : **11 tests réussis**, sans carte son.
- `cargo test -p harmonium_bevy --features odin` : **11 tests réussis**, sans carte son.
- Contrats exercés : cibles du jeu, graine, signal SoundFont réellement réduit par le volume, silence exact à zéro, seuil/cadence des émotions, pause/reprise sans doublon, horloge/tempo et messages de chaque pulse/mesure, fondu jusqu'au silence rendu, remplacement de cible en pause, redémarrage avec émotions modifiées avant le callback, remplacement effectif de la mesure 1 lors d'un changement de mètre.
- `make bevy/check` : réussi. Clippy stable `--no-deps --all-targets -- -D warnings`, sans Odin puis avec `demo,odin` ; compilation `wasm32-unknown-unknown` sans et avec Odin.
- `cargo +nightly fmt --all -- --check` : réussi sur les fichiers livrés, après retrait des sondes temporaires.
- `make bevy/demo ARGS="--smoke-seconds 8"` : compilation release et exécution réussies, arrêt automatique après 8,00 s ; volume zéro.
- `cargo test --workspace -q --no-fail-fast` : **réussi**, sans relance isolée nécessaire.
- La reprise audio par geste DOM est compilée et documentée ; elle n'a pas été exercée dans un navigateur pendant cette session.
- Limite de vérification visuelle : la lecture AX macOS est refusée par l'environnement et les sondes GPU de fenêtre ne recevaient pas de texture de présentation. Les captures étaient noires ; **pas de validation visuelle du carré revendiquée**. La caméra, la mise en page, l'horloge et les variations de taille étaient présentes dans l'application. Toutes les sondes de capture ont été retirées. L'écoute et la vérification visuelle restent à faire par le propriétaire dans sa session graphique.
- Avis externes observés : compatibilité future Rust signalée par des dépendances Bevy/wgpu/winit/nix ; oxisynth ignore un échantillon vide de la banque de pieds. Aucun avertissement Clippy du crate livré.

### Journal de la démo release finale

Extrait du lancement du 2026-10-30 (sortie muette, aucun écouteur simulé) :

```text
Harmonium : ressources chargées (principale=true, pieds=true, presets=260)
Harmonium : hôte démarré en pause, initialisation des cibles
Harmonium : composition mesures 1–16 (16 mesures)
Harmonium : composition mesures 17–17 (1 mesures)
Harmonium : composition mesures 20–20 (1 mesures)
Fin automatique de la démo après 8.00 s
```

Le chargement et la composition en avance continuent donc après le premier remplissage de 16 mesures. Les deux banques font respectivement 32 319 396 et 154 186 octets ; les 260 presets sont ceux du dossier local configuré, aucun n'est ajouté au dépôt.

## Intégration avec G2

B1 reste sur sa base de branche : aucun merge, rebase ou push. Le changement G2 de `main` (`AudioEvent::NoteOn.legato`) n'est pas reporté ici ; le propriétaire résout l'intégration à la fusion. Le plugin ne construit pas lui-même d'`AudioEvent::NoteOn`.
