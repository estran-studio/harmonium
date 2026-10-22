//! Help system for Harmonium CLI

use colored::Colorize;

/// Print main help message
pub fn print_help() {
    println!();
    println!("{}", "Harmonium CLI - Interactive Generative Music Engine".bold().cyan());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    print_section("CONTROL MODES", &[
        ("emotion", "Switch to emotion mode (EmotionMapper translates to musical params)"),
        ("emotion <a> <v> <d> <t>", "Set emotions: arousal valence density tension → BPM, patterns, etc."),
        ("emotion a+0.1 v-0.2", "Relative: a=arousal→BPM, v=valence→major/minor, d=density→pulses, t=tension→strategy"),
        ("direct", "Switch to direct mode (bypass EmotionMapper, set params directly)"),
    ]);

    print_section(
        "GLOBAL PARAMETERS",
        &[
            ("set bpm <70-180>", "Set BPM (overrides emotion-mapped BPM)"),
            ("set volume <0-1>", "Set master volume"),
            ("set time <n/d>", "Set time signature (e.g., 4/4)"),
        ],
    );

    print_section(
        "BEAT, SWING & TEMPO (phase 0)",
        &[
            ("beat", "Métronome visuel calé sur BeatInfo (Entrée/Échap/q/Ctrl+C pour sortir)"),
            ("swing <0.5-0.75>", "Swing des croches (0.5 = droit ; effet à la prochaine mesure)"),
            (
                "tempo <bpm> [pulses]",
                "Rampe vers le tempo audible (défaut : saut, moteur borné à 40–300)",
            ),
            ("tempo off", "Retour au tempo des mesures"),
        ],
    );

    print_section(
        "TRAD / REEL (phase 1)",
        &[
            (
                "style <identifiant|dossier|legacy>",
                "Charge un profil (reel_quebecois, alias reel) ou son dossier ; legacy conserve l’ancien moteur",
            ),
            ("feet <0|1|2>", "Densité de podorythmie (0 = talon 0,8 ; 1 = standard ; 2 = ornée)"),
            (
                "intensity <0..1>",
                "Énergie, articulation, pieds et chop ; effet à la prochaine mesure",
            ),
            ("seed <u64>", "Fixe le seed de session et régénère depuis la mesure 1"),
        ],
    );

    print_section(
        "RHYTHM PARAMETERS",
        &[
            ("set rhythm_mode <mode>", "euclidean | perfect | classic"),
            ("set steps <4-192>", "Number of steps in pattern"),
            ("set pulses <1-16>", "Number of pulses (triggers)"),
            ("set rotation <0-15>", "Pattern rotation offset"),
            ("set density <0-1>", "Rhythmic density"),
            ("set rhythm_tension <0-1>", "Rhythmic tension"),
        ],
    );

    print_section(
        "HARMONY PARAMETERS",
        &[
            ("set harmony_mode <mode>", "basic | driver"),
            ("set valence <-1 to 1>", "Positive/negative emotion"),
            ("set harmony_tension <0-1>", "Harmonic dissonance"),
        ],
    );

    print_section(
        "MELODY & VOICING",
        &[
            ("set smoothness <0-1>", "Melodic smoothness (Hurst)"),
            ("set octave <-2 to 2>", "Melody octave shift"),
            ("set voicing_density <0-1>", "Chord voicing density"),
            ("set voicing_tension <0-1>", "Voicing tension"),
        ],
    );

    print_section(
        "MODULE TOGGLES",
        &[
            ("enable <module>", "Enable rhythm|harmony|melody|voicing"),
            ("disable <module>", "Disable rhythm|harmony|melody|voicing"),
        ],
    );

    print_section(
        "MIXER",
        &[
            ("set gain <ch> <0-1>", "Set channel gain (ch: 0-4)"),
            ("set mute <ch>", "Mute channel (0=melody, 1=chords, 2=bass, 3=heel, 4=toe)"),
            ("set unmute <ch>", "Unmute channel"),
        ],
    );

    print_section(
        "TIMELINE (timeline engine only)",
        &[
            ("seek <bar>", "Seek playhead to a specific bar"),
            ("loop <start> <end>", "Loop playback between two bars"),
            ("loop off", "Clear loop region"),
        ],
    );

    print_section(
        "RECORDING",
        &[
            ("record wav", "Start WAV recording"),
            ("record midi", "Start MIDI recording"),
            ("record musicxml", "Start MusicXML recording"),
            ("stop", "Stop all recordings"),
            ("stop <format>", "Stop specific format (wav|midi|musicxml)"),
        ],
    );

    print_section(
        "UTILITY",
        &[
            ("state | show | status", "Show current engine state"),
            ("reset", "Reset engine to defaults"),
            ("reset bpm", "Clear BPM override, return to emotion-mapped BPM"),
            ("help | ?", "Show this help message"),
            ("quit | exit", "Exit the CLI"),
        ],
    );

    println!();
    println!("{}", "KEYBOARD SHORTCUTS:".bold().yellow());
    println!("  {} - Autocomplete commands and parameters", "Tab".green());
    println!("  {} - Quit immediately", "Ctrl+C".green());
    println!("  {} - Navigate command history", "↑/↓".green());

    println!();
    println!("{}", "EXAMPLES:".bold().yellow());
    println!("  {}", "set bpm 140".green());
    println!("  {}", "emotion 0.9 0.5 0.7 0.6    # High arousal, neutral valence".green());
    println!("  {}", "emotion a+0.1 t-0.2        # Increase arousal, decrease tension".green());
    println!("  {}", "set rhythm_mode perfect".green());
    println!("  {}", "enable voicing".green());
    println!("  {}", "seek 4                         # Jump to bar 4".green());
    println!("  {}", "loop 4 8                       # Loop bars 4-8".green());
    println!("  {}", "record midi output.mid".green());
    println!();
}

/// Print help for a specific command
pub fn print_command_help(command: &str) {
    println!();

    match command {
        "set" => {
            println!("{}", "SET COMMAND".bold().cyan());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("Sets a specific engine parameter.");
            println!();
            println!("{}", "SYNTAX:".bold());
            println!("  set <parameter> <value>");
            println!();
            println!("{}", "AVAILABLE PARAMETERS:".bold());
            println!("  {}  - BPM (70-180)", "bpm".green());
            println!("  {}  - Master volume (0-1)", "volume".green());
            println!("  {}  - Time signature (e.g., 4/4)", "time".green());
            println!("  {}  - euclidean|perfect|classic", "rhythm_mode".green());
            println!("  {}  - Pattern steps (4-192)", "steps".green());
            println!("  {}  - Pattern pulses (1-16)", "pulses".green());
            println!("  {}  - Pattern rotation (0-15)", "rotation".green());
            println!("  {}  - Rhythmic density (0-1)", "density".green());
            println!("  {}  - basic|driver", "harmony_mode".green());
            println!("  {}  - Emotion (-1 to 1)", "valence".green());
            println!("  {}  - Melodic smoothness (0-1)", "smoothness".green());
            println!("  {}  - Octave shift (-2 to 2)", "octave".green());
        }

        "intensity" => {
            println!("intensity <0..1> : énergie du style ; 0.5 conserve le rendu par défaut.");
            println!("Reel : tempo borné autour du style, notes plus sèches, accents et pieds plus forts.");
            println!("L'air courant garde ses hauteurs ; la nouvelle distribution de cellules attend l'air suivant.");
        }

        "emotion" => {
            println!("{}", "EMOTION MODE".bold().cyan());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("Control the engine using emotional parameters.");
            println!("EmotionMapper translates emotions to musical parameters.");
            println!();
            println!("{}", "SYNTAX:".bold());
            println!("  emotion                          - Switch to emotion mode");
            println!("  emotion <arousal> <valence> <density> <tension>");
            println!();
            println!("{}", "PARAMETERS:".bold());
            println!("  {}  - Energy level (0-1) → affects BPM", "arousal".green());
            println!("  {}  - Mood (-1=sad, +1=happy) → affects harmony", "valence".green());
            println!("  {}  - Rhythmic complexity (0-1)", "density".green());
            println!("  {}  - Harmonic dissonance (0-1)", "tension".green());
            println!();
            println!("{}", "EXAMPLES:".bold());
            println!("  emotion 0.9 0.7 0.6 0.3    # High energy, happy, moderate density");
            println!("  emotion 0.3 -0.5 0.2 0.6   # Low energy, sad, sparse, tense");
        }

        "record" => {
            println!("{}", "RECORDING".bold().cyan());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("Record engine output to file.");
            println!();
            println!("{}", "SYNTAX:".bold());
            println!("  record <format> [filename]");
            println!("  stop                         - Stop all recordings");
            println!();
            println!("{}", "FORMATS:".bold());
            println!("  {}  - Waveform audio", "wav".green());
            println!("  {}  - MIDI sequence", "midi".green());
            println!("  {}  - MusicXML notation", "musicxml".green());
            println!();
            println!("{}", "EXAMPLES:".bold());
            println!("  record wav output.wav");
            println!("  record midi groove.mid");
            println!("  stop");
        }

        "swing" => {
            println!("{}", "SWING".bold().cyan());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("Règle le swing des paires de croches (phase 0, D3).");
            println!();
            println!("{}", "SYNTAX:".bold());
            println!("  swing <0.5-0.75>");
            println!();
            println!("0.5 = droit, 0.6 ≈ reel. Hors plage : borné et signalé.");
            println!("Prend effet à la prochaine mesure générée.");
            println!();
            println!("{}", "EXAMPLES:".bold());
            println!("  swing 0.6");
        }

        "tempo" => {
            println!("{}", "TEMPO".bold().cyan());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("Tempo audible du playback (phase 0, D4), distinct du");
            println!("`set bpm` (tempo nominal des mesures générées).");
            println!();
            println!("{}", "SYNTAX:".bold());
            println!("  tempo <bpm> [pulses]");
            println!("  tempo off                    - retour au tempo des mesures");
            println!();
            println!("Sans `pulses` : saut à la prochaine pulse. Avec `pulses` :");
            println!("rampe linéaire atteignant la cible à la N-ième frontière.");
            println!("Le moteur borne à 40–300 BPM. `EngineReport.bpm` suit l'audible.");
            println!();
            println!("{}", "EXAMPLES:".bold());
            println!("  tempo 200 8");
            println!("  tempo off");
        }

        "beat" => {
            println!("{}", "BEAT".bold().cyan());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("Métronome visuel calé sur `BeatInfo` (~60 lectures/s) :");
            println!("mesure, pulse, barre de phase, marqueur de temps fort.");
            println!("Aucune décision sur `Instant::now` : seule `BeatInfo` compte.");
            println!();
            println!("{}", "SYNTAX:".bold());
            println!("  beat");
            println!();
            println!("Sortie : Entrée, Échap, q ou Ctrl+C (retour au REPL).");
        }
        "style" => {
            println!("{}", "STYLE".bold().cyan());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("Choisit le générateur actif (`reel` trad québécois ou `legacy`).");
            println!("En `reel`, applique les défauts de `TuningParams` : 230 BPM, swing 0.56,");
            println!(
                "signature 2/2 et programmes GM D10 (violon 40, guitare 24, basse 32, pieds ch 9)."
            );
            println!();
            println!("{}", "SYNTAX:".bold());
            println!("  style <identifiant|dossier|legacy>");
        }

        "feet" => {
            println!("{}", "FEET".bold().cyan());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("Règle la densité de podorythmie en style Reel (D9) :");
            println!("  0 = talon sur 0 et 8");
            println!("  1 = talon 0, 4, 8, 12 + pointe 2, 6, 10, 14 (défaut)");
            println!("  2 = talon 0, 4, 8, 12 + pointe doublée (2, 3, 6, 7, 10, 11, 14, 15)");
            println!();
            println!("{}", "SYNTAX:".bold());
            println!("  feet <0|1|2>");
        }
        _ => {
            println!("{}", format!("No detailed help available for '{}'", command).yellow());
            println!("Type {} for a list of all commands.", "help".green());
        }
    }

    println!();
}

/// Print a help section with commands
fn print_section(title: &str, commands: &[(&str, &str)]) {
    println!("{}", title.bold().yellow());
    for (cmd, desc) in commands {
        println!("  {:<30} {}", cmd.green(), desc.dimmed());
    }
    println!();
}
