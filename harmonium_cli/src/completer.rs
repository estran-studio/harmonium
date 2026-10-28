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
    record_formats: Vec<&'static str>,
}

impl HarmoniumCompleter {
    pub fn new() -> Self {
        Self {
            commands: vec![
                "set",
                "emotion",
                "direct",
                "record",
                "stop",
                "state",
                "show",
                "status",
                "seek",
                "loop",
                "reset",
                "swing",
                "tempo",
                "beat",
                "style",
                "feet",
                "seed",
                "intensity",
                "help",
                "quit",
                "exit",
            ],
            set_params: vec![
                "bpm",
                "volume",
                "master_volume",
                "time",
                "time_signature",
                "gain",
                "mute",
                "unmute",
                "rhythm",
                "harmony",
                "melody",
                "route",
                "output",
                "lookahead",
                "seed",
            ],
            record_formats: vec!["wav", "midi", "musicxml"],
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
                        "time" | "time_signature" => vec!["4/4", "3/4", "5/4", "7/8", "6/8"]
                            .iter()
                            .map(|&sig| Pair {
                                display: sig.to_string(),
                                replacement: sig.to_string(),
                            })
                            .collect(),
                        "rhythm" | "harmony" | "melody" => ["on", "off"]
                            .iter()
                            .map(|&v| Pair { display: v.to_string(), replacement: v.to_string() })
                            .collect(),
                        "output" => ["mute", "unmute"]
                            .iter()
                            .map(|&v| Pair { display: v.to_string(), replacement: v.to_string() })
                            .collect(),
                        _ => vec![],
                    }
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
                    ["reel_quebecois", "gigue_quebecoise", "valse_quebecoise"]
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
                    ["0", "1", "2"]
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
                    [
                        "set",
                        "emotion",
                        "enable",
                        "disable",
                        "record",
                        "swing",
                        "tempo",
                        "beat",
                        "style",
                        "feet",
                        "intensity",
                    ]
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
