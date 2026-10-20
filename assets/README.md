# SoundFonts pour Harmonium

- `test.sf2` : Banque minimale de test (450 Ko, 19 presets en banque 0). Versionnée pour exécuter les tests du workspace sans dépendance réseau. Ne contient ni violon (programme 40), ni guitare acier (24), ni basse acoustique (32), ni kit de batterie (banque 128).
- `GeneralUser-GS.sf2` : Banque General MIDI / GS complète recommandée pour le développement et l'écoute (~32 Mo). Téléchargée localement via `make soundfont` (ignorée par git).
  - Auteur : S. Christian Collins
  - Site officiel : <https://www.schristiancollins.com/generaluser.php>
  - Dépôt source : <https://github.com/mrbumpy409/GeneralUser-GS>
  - Licence : Gratuite et libre pour créations musicales privées ou commerciales (<https://raw.githubusercontent.com/mrbumpy409/GeneralUser-GS/main/documentation/LICENSE.txt>).
- `harmonium_trad.sf2` : banque de pieds de podorythmie (un preset « Podorythmie » banque 0 programme 0, talon touche 36, pointe touche 37). Construite par `harmonium_lab` via `make trad-sf2` à partir d'échantillons CC0 de Freesound (8 frappes, listées dans `samples/feet/manifest.jsonl` du lab). Ignorée par git ; chargée par `--feet-soundfont` (défaut `assets/harmonium_trad.sf2`, sinon repli sur le kit GM canal 9).
