//! Tab completion for Harmonium CLI

use rustyline::{
    completion::{Completer, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
    Context, Helper,
};

/// Tab completer for Harmonium commands
pub struct HarmoniumCompleter {
    commands: Vec<&'static str>,
    set_params: Vec<&'static str>,
    modules: Vec<&'static str>,
    record_formats: Vec<&'static str>,
    rhythm_modes: Vec<&'static str>,
    harmony_modes: Vec<&'static str>,
}

impl HarmoniumCompleter {
    pub fn new() -> Self {
        Self {
            commands: vec![
                "set", "emotion", "direct", "enable", "disable", "record", "stop", "state", "show",
                "status", "seek", "loop", "reset", "swing", "tempo", "beat", "style", "feet",
                "seed", "help", "quit", "exit",
            ],
            set_params: vec![
                "bpm",
                "volume",
                "master_volume",
                "time",
                "time_signature",
                "rhythm_mode",
                "steps",
                "pulses",
                "rotation",
                "density",
                "rhythm_density",
                "rhythm_tension",
                "harmony_mode",
                "harmony_tension",
                "valence",
                "harmony_valence",
                "smoothness",
                "melody_smoothness",
                "octave",
                "melody_octave",
                "voicing_density",
                "voicing_tension",
                "gain",
                "mute",
                "unmute",
            ],
            modules: vec!["rhythm", "harmony", "melody", "voicing"],
            record_formats: vec!["wav", "midi", "musicxml"],
            rhythm_modes: vec![
                "euclidean",
                "perfect",
                "perfectbalance",
                "classic",
                "classicgroove",
            ],
            harmony_modes: vec!["basic", "driver"],
        }
    }

    fn get_candidates(&self, line: &str, pos: usize) -> Vec<Pair> {
        let text = &line[..pos];
        let tokens: Vec<&str> = text.split_whitespace().collect();

        if tokens.is_empty() {
            // Complete command names
            return self
                .commands
                .iter()
                .map(|&cmd| Pair { display: cmd.to_string(), replacement: cmd.to_string() })
                .collect();
        }

        match tokens[0] {
            "set" => {
                if tokens.len() == 1 {
                    // Complete parameter names
                    self.set_params
                        .iter()
                        .map(|&param| Pair {
                            display: param.to_string(),
                            replacement: param.to_string(),
                        })
                        .collect()
                } else if tokens.len() == 2 {
                    // Complete parameter names (user started typing)
                    let partial = tokens[1].to_lowercase();
                    self.set_params
                        .iter()
                        .filter(|param| param.starts_with(&partial))
                        .map(|&param| Pair {
                            display: param.to_string(),
                            replacement: param.to_string(),
                        })
                        .collect()
                } else if tokens.len() == 3 {
                    // Complete parameter values based on parameter type
                    match tokens[1] {
                        "rhythm_mode" | "rhythm-mode" => self
                            .rhythm_modes
                            .iter()
                            .map(|&mode| Pair {
                                display: mode.to_string(),
                                replacement: mode.to_string(),
                            })
                            .collect(),
                        "harmony_mode" | "harmony-mode" => self
                            .harmony_modes
                            .iter()
                            .map(|&mode| Pair {
                                display: mode.to_string(),
                                replacement: mode.to_string(),
                            })
                            .collect(),
                        "time" | "time_signature" => vec!["4/4", "3/4", "5/4", "7/8", "6/8"]
                            .iter()
                            .map(|&sig| Pair {
                                display: sig.to_string(),
                                replacement: sig.to_string(),
                            })
                            .collect(),
                        _ => vec![],
                    }
                } else {
                    vec![]
                }
            }

            "enable" | "disable" => {
                if tokens.len() <= 2 {
                    let partial =
                        if tokens.len() == 2 { tokens[1].to_lowercase() } else { String::new() };
                    self.modules
                        .iter()
                        .filter(|module| module.starts_with(&partial))
                        .map(|&module| Pair {
                            display: module.to_string(),
                            replacement: module.to_string(),
                        })
                        .collect()
                } else {
                    vec![]
                }
            }

            "record" => {
                if tokens.len() <= 2 {
                    let partial =
                        if tokens.len() == 2 { tokens[1].to_lowercase() } else { String::new() };
                    self.record_formats
                        .iter()
                        .filter(|fmt| fmt.starts_with(&partial))
                        .map(|&fmt| Pair { display: fmt.to_string(), replacement: fmt.to_string() })
                        .collect()
                } else {
                    vec![]
                }
            }

            "loop" => {
                if tokens.len() <= 2 {
                    let partial =
                        if tokens.len() == 2 { tokens[1].to_lowercase() } else { String::new() };
                    vec!["off"]
                        .iter()
                        .filter(|s| s.starts_with(&partial))
                        .map(|&s| Pair { display: s.to_string(), replacement: s.to_string() })
                        .collect()
                } else {
                    vec![]
                }
            }


            "style" => {
                if tokens.len() <= 2 {
                    let partial =
                        if tokens.len() == 2 { tokens[1].to_lowercase() } else { String::new() };
                    vec!["reel", "legacy"]
                        .iter()
                        .filter(|s| s.starts_with(&partial))
                        .map(|&s| Pair { display: s.to_string(), replacement: s.to_string() })
                        .collect()
                } else {
                    vec![]
                }
            }

            "feet" => {
                if tokens.len() <= 2 {
                    let partial =
                        if tokens.len() == 2 { tokens[1].to_lowercase() } else { String::new() };
                    vec!["0", "1", "2"]
                        .iter()
                        .filter(|s| s.starts_with(&partial))
                        .map(|&s| Pair { display: s.to_string(), replacement: s.to_string() })
                        .collect()
                } else {
                    vec![]
                }
            }
            "help" => {
                if tokens.len() <= 2 {
                    vec!["set", "emotion", "enable", "disable", "record", "swing", "tempo", "beat", "style", "feet"]
                        .iter()
                        .map(|&cmd| Pair { display: cmd.to_string(), replacement: cmd.to_string() })
                        .collect()
                } else {
                    vec![]
                }
            }

            _ => {
                // If first token is incomplete, complete command names
                if tokens.len() == 1 && !text.ends_with(' ') {
                    let partial = tokens[0].to_lowercase();
                    self.commands
                        .iter()
                        .filter(|cmd| cmd.starts_with(&partial))
                        .map(|&cmd| Pair { display: cmd.to_string(), replacement: cmd.to_string() })
                        .collect()
                } else {
                    vec![]
                }
            }
        }
    }
}

impl Completer for HarmoniumCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let candidates = self.get_candidates(line, pos);

        if candidates.is_empty() {
            return Ok((pos, vec![]));
        }

        // Find start of current word
        let start = line[..pos].rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);

        Ok((start, candidates))
    }
}

impl Hinter for HarmoniumCompleter {
    type Hint = String;
}

impl Highlighter for HarmoniumCompleter {}

impl Validator for HarmoniumCompleter {}

impl Helper for HarmoniumCompleter {}
