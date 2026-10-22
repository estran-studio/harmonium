//! Claude API Integration for LLM-Assisted Tuning
//!
//! This module provides integration with the Anthropic Claude API
//! for suggesting parameter adjustments based on DNA analysis.

use harmonium_core::tuning::TuningParams;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dna_types::GlobalMetrics;

/// Errors that can occur during Claude API communication
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Response parse error: {0}")]
    ParseError(String),

    #[error("API key not configured")]
    NoApiKey,

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

/// Claude API request structure
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

/// Claude API response structure
#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

/// Claude agent for LLM-assisted tuning
#[derive(Clone, Debug)]
pub struct ClaudeAgent {
    api_key: Option<String>,
    model: String,
    client: reqwest::Client,
}

impl Default for ClaudeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeAgent {
    /// Create a new Claude agent
    #[must_use]
    pub fn new() -> Self {
        Self {
            api_key: None,
            model: "claude-sonnet-4-20250514".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Set the API key
    #[must_use]
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the model to use
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Check if the agent is configured with an API key
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    /// Request tuning suggestions from Claude (async)
    ///
    /// # Errors
    /// Returns error if API key is not configured or request fails
    pub async fn suggest_tuning(
        &self,
        reference: &GlobalMetrics,
        generated: &GlobalMetrics,
        current_tuning: &TuningParams,
    ) -> Result<TuningSuggestion, AgentError> {
        let api_key = self.api_key.as_ref().ok_or(AgentError::NoApiKey)?;

        // Build the prompt
        let prompt = self.build_prompt(reference, generated, current_tuning);

        // Make the API request
        let request_body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 2048,
            messages: vec![Message { role: "user".to_string(), content: prompt }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        // Check for rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AgentError::RateLimited);
        }

        // Check for other errors
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AgentError::RequestFailed(format!("HTTP {}: {}", status, body)));
        }

        // Parse response
        let claude_response: ClaudeResponse =
            response.json().await.map_err(|e| AgentError::ParseError(e.to_string()))?;

        // Extract text from response
        let response_text: String = claude_response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .filter_map(|c| c.text.as_ref())
            .cloned()
            .collect();

        // Parse the JSON response from Claude
        self.parse_suggestion(&response_text, current_tuning)
    }

    /// Request tuning suggestions from Claude (blocking)
    ///
    /// # Errors
    /// Returns error if API key is not configured or request fails
    pub fn suggest_tuning_blocking(
        &self,
        reference: &GlobalMetrics,
        generated: &GlobalMetrics,
        current_tuning: &TuningParams,
    ) -> Result<TuningSuggestion, AgentError> {
        let api_key = self.api_key.as_ref().ok_or(AgentError::NoApiKey)?;

        // Build the prompt
        let prompt = self.build_prompt(reference, generated, current_tuning);

        // Make the API request using blocking client
        let client = reqwest::blocking::Client::new();

        let request_body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 2048,
            messages: vec![Message { role: "user".to_string(), content: prompt }],
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| AgentError::RequestFailed(e.to_string()))?;

        // Check for rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AgentError::RateLimited);
        }

        // Check for other errors
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(AgentError::RequestFailed(format!("HTTP {}: {}", status, body)));
        }

        // Parse response
        let claude_response: ClaudeResponse =
            response.json().map_err(|e| AgentError::ParseError(e.to_string()))?;

        // Extract text from response
        let response_text: String = claude_response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .filter_map(|c| c.text.as_ref())
            .cloned()
            .collect();

        // Parse the JSON response from Claude
        self.parse_suggestion(&response_text, current_tuning)
    }

    /// Build the prompt for Claude
    fn build_prompt(
        &self,
        reference: &GlobalMetrics,
        generated: &GlobalMetrics,
        current_tuning: &TuningParams,
    ) -> String {
        format!(
            r#"You are a music theory expert tuning a generative music algorithm.

## Target Style Profile (from real music corpus)
- avg_voice_leading_effort: {:.2}
- tension_variance: {:.4}
- tension_release_balance: {:.2}
- diatonic_percentage: {:.1}%
- harmonic_rhythm: {:.2} chords/measure

## Generated Music Profile
- avg_voice_leading_effort: {:.2} ({})
- tension_variance: {:.4} ({})
- tension_release_balance: {:.2} ({})
- diatonic_percentage: {:.1}% ({})
- harmonic_rhythm: {:.2} ({})

## Current Tuning Parameters (with explanations)
```toml
# Harmony - Voice Leading
max_semitone_movement = {}  # Max semitones a voice can move (1-3). Lower = smoother voice leading.
cardinality_morph_enabled = {}  # Allow triad ↔ tetrad transitions
trq_threshold = {:.2}  # Tension/Release Quotient threshold for neighbor selection (0.0-1.0)

# Harmony - Strategy Selection (Steedman vs Neo-Riemannian)
steedman_lower_threshold = {:.2}  # Stay in Steedman while tension > this
steedman_upper_threshold = {:.2}  # Enter Steedman when tension < this
neo_riemannian_lower_threshold = {:.2}  # Enter Neo-Riemannian when tension > this
neo_riemannian_upper_threshold = {:.2}  # Stay in Neo-Riemannian while tension < this
hysteresis_boost = {:.2}  # Stability boost for current strategy (0.0-0.3)

# Rhythm - Perfect Balance Polygons
kick_density_threshold = {:.2}  # Density below this uses low-density kick pattern
kick_low_density_vertices = {}  # Kick polygon vertices at low density (2 = half notes)
kick_high_density_vertices = {}  # Kick polygon vertices at high density (4 = quarter notes)
snare_density_threshold = {:.2}  # Density threshold for snare pattern
snare_low_density_vertices = {}  # Snare vertices at low density
snare_high_density_vertices = {}  # Snare vertices at high density
hat_very_low_density_vertices = {}  # Hi-hat vertices at very low density
hat_low_density_vertices = {}  # Hi-hat vertices at low density
hat_medium_density_vertices = {}  # Hi-hat vertices at medium density
hat_high_density_vertices = {}  # Hi-hat vertices at high density
```

## Your Task
Analyze the divergence between generated and target profiles.
Suggest parameter adjustments to move generated closer to target.

Key relationships:
- Higher max_semitone_movement → higher avg_voice_leading_effort
- Lower trq_threshold → more stable/predictable harmony
- More kick/snare vertices → faster harmonic rhythm
- Lower steedman thresholds → more functional harmony (lower tension variance)

Return ONLY valid JSON (no markdown code blocks) in this exact format:
{{"reasoning": "Your analysis of what needs to change and why", "changes": {{"parameter_name": new_value, "another_param": new_value}}, "confidence": 0.8}}

Example response:
{{"reasoning": "Voice leading effort is too high (3.2 vs target 1.5). Reducing max_semitone_movement from 2 to 1 will create smoother voice leading.", "changes": {{"max_semitone_movement": 1}}, "confidence": 0.85}}"#,
            // Reference metrics
            reference.average_voice_leading_effort,
            reference.tension_variance,
            reference.tension_release_balance,
            reference.diatonic_percentage,
            reference.harmonic_rhythm,
            // Generated metrics with direction indicators
            generated.average_voice_leading_effort,
            Self::direction_indicator(
                generated.average_voice_leading_effort,
                reference.average_voice_leading_effort
            ),
            generated.tension_variance,
            Self::direction_indicator(generated.tension_variance, reference.tension_variance),
            generated.tension_release_balance,
            Self::direction_indicator(
                generated.tension_release_balance,
                reference.tension_release_balance
            ),
            generated.diatonic_percentage,
            Self::direction_indicator(generated.diatonic_percentage, reference.diatonic_percentage),
            generated.harmonic_rhythm,
            Self::direction_indicator(generated.harmonic_rhythm, reference.harmonic_rhythm),
            // Current tuning parameters (mapped to new nested TuningParams)
            current_tuning.voice_leading.max_semitone_movement,
            current_tuning.voice_leading.allow_cardinality_morph,
            current_tuning.voice_leading.trq_threshold,
            current_tuning.harmony_driver.steedman_lower,
            current_tuning.harmony_driver.steedman_upper,
            current_tuning.harmony_driver.neo_lower,
            current_tuning.harmony_driver.neo_upper,
            current_tuning.harmony_driver.hysteresis_boost,
            current_tuning.perfect_balance.kick_polygon_low_threshold,
            2usize, // kick low density vertices (digon)
            4usize, // kick high density vertices (square)
            0.5f32, // snare density threshold (not directly mapped)
            3usize, // snare low density vertices
            6usize, // snare high density vertices
            current_tuning.perfect_balance.hat_vertex_counts[0],
            current_tuning.perfect_balance.hat_vertex_counts[1],
            current_tuning.perfect_balance.hat_vertex_counts[2],
            current_tuning.perfect_balance.hat_vertex_counts[3],
        )
    }

    /// Parse the suggestion from Claude's response
    fn parse_suggestion(
        &self,
        response_text: &str,
        current_tuning: &TuningParams,
    ) -> Result<TuningSuggestion, AgentError> {
        // Try to find JSON in the response (Claude might add text around it)
        let json_start = response_text.find('{');
        let json_end = response_text.rfind('}');

        let json_str = match (json_start, json_end) {
            (Some(start), Some(end)) if end > start => &response_text[start..=end],
            _ => {
                return Err(AgentError::ParseError("No JSON object found in response".to_string()));
            }
        };

        // Parse the JSON
        let raw: RawSuggestion = serde_json::from_str(json_str)
            .map_err(|e| AgentError::ParseError(format!("JSON parse error: {}", e)))?;

        // Convert changes to ParameterChange structs
        let parameter_changes = self.extract_parameter_changes(&raw.changes, current_tuning);

        Ok(TuningSuggestion {
            reasoning: raw.reasoning,
            parameter_changes,
            confidence: raw.confidence.unwrap_or(0.5),
        })
    }

    /// Extract parameter changes from the raw JSON
    fn extract_parameter_changes(
        &self,
        changes: &serde_json::Value,
        current: &TuningParams,
    ) -> Vec<ParameterChange> {
        let mut result = Vec::new();

        if let Some(obj) = changes.as_object() {
            for (key, value) in obj {
                let current_value = self.get_current_value(key, current);
                let suggested_value = format_json_value(value);

                if current_value != suggested_value {
                    result.push(ParameterChange {
                        name: key.clone(),
                        current: current_value,
                        suggested: suggested_value,
                        reason: String::new(), // Claude provides overall reasoning
                    });
                }
            }
        }

        result
    }

    /// Get the current value of a parameter as a string
    fn get_current_value(&self, name: &str, tuning: &TuningParams) -> String {
        match name {
            "max_semitone_movement" => tuning.voice_leading.max_semitone_movement.to_string(),
            "cardinality_morph_enabled" => tuning.voice_leading.allow_cardinality_morph.to_string(),
            "trq_threshold" => format!("{:.2}", tuning.voice_leading.trq_threshold),
            "steedman_lower_threshold" => format!("{:.2}", tuning.harmony_driver.steedman_lower),
            "steedman_upper_threshold" => format!("{:.2}", tuning.harmony_driver.steedman_upper),
            "neo_riemannian_lower_threshold" => format!("{:.2}", tuning.harmony_driver.neo_lower),
            "neo_riemannian_upper_threshold" => format!("{:.2}", tuning.harmony_driver.neo_upper),
            "hysteresis_boost" => format!("{:.2}", tuning.harmony_driver.hysteresis_boost),
            "kick_density_threshold" => {
                format!("{:.2}", tuning.perfect_balance.kick_polygon_low_threshold)
            }
            "hat_very_low_density_vertices" => {
                tuning.perfect_balance.hat_vertex_counts[0].to_string()
            }
            "hat_low_density_vertices" => tuning.perfect_balance.hat_vertex_counts[1].to_string(),
            "hat_medium_density_vertices" => {
                tuning.perfect_balance.hat_vertex_counts[2].to_string()
            }
            "hat_high_density_vertices" => tuning.perfect_balance.hat_vertex_counts[3].to_string(),
            _ => "unknown".to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // Style-from-description generation (CORELIB-26)
    // -----------------------------------------------------------------------

    /// Generate TuningParams from a natural-language style description.
    ///
    /// # Errors
    /// Returns error if API key is not configured or request fails.
    pub fn generate_style_blocking(
        &self,
        name: &str,
        description: &str,
    ) -> Result<StyleGenerationResult, AgentError> {
        let api_key = self.api_key.as_ref().ok_or(AgentError::NoApiKey)?;
        let prompt = Self::build_style_prompt(name, description);

        let client = reqwest::blocking::Client::new();
        let request_body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![Message { role: "user".to_string(), content: prompt }],
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| AgentError::RequestFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AgentError::RateLimited);
        }
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(AgentError::RequestFailed(format!("HTTP {}: {}", status, body)));
        }

        let claude_response: ClaudeResponse =
            response.json().map_err(|e| AgentError::ParseError(e.to_string()))?;

        let response_text: String = claude_response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .filter_map(|c| c.text.as_ref())
            .cloned()
            .collect();

        Self::parse_style_response(&response_text)
    }

    /// Refine a previously generated TuningParams based on human rating and feedback.
    ///
    /// # Errors
    /// Returns error if API key is not configured or request fails.
    pub fn refine_style_blocking(
        &self,
        name: &str,
        description: &str,
        current: &TuningParams,
        rating: u8,
        feedback: &str,
    ) -> Result<StyleGenerationResult, AgentError> {
        let api_key = self.api_key.as_ref().ok_or(AgentError::NoApiKey)?;
        let prompt = Self::build_refinement_prompt(name, description, current, rating, feedback);

        let client = reqwest::blocking::Client::new();
        let request_body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![Message { role: "user".to_string(), content: prompt }],
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| AgentError::RequestFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AgentError::RateLimited);
        }
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(AgentError::RequestFailed(format!("HTTP {}: {}", status, body)));
        }

        let claude_response: ClaudeResponse =
            response.json().map_err(|e| AgentError::ParseError(e.to_string()))?;

        let response_text: String = claude_response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .filter_map(|c| c.text.as_ref())
            .cloned()
            .collect();

        Self::parse_style_response(&response_text)
    }

    /// Build prompt for initial style generation from a description.
    fn build_style_prompt(name: &str, description: &str) -> String {
        let schema = serde_json::to_string_pretty(&TuningParams::default())
            .unwrap_or_else(|_| "{}".to_string());

        format!(
            r#"You are an expert music producer and algorithm tuner for a generative music engine called Harmonium.

## Task
Generate a complete TuningParams JSON for the style: "{name}"
Description: {description}

## TuningParams Schema (with current defaults)
The following JSON shows ALL available parameters with their default values.
Every field is tunable. Adjust the values that this style would change.

```json
{schema}
```

## Parameter Group Guide
- **harmony_driver**: Strategy selection thresholds. Lower steedman values = more functional harmony. Higher neo values = more chromatic/experimental.
- **grammar**: Rule weight multipliers on top of the grammar_style preset. >1.0 amplifies, <1.0 suppresses.
- **neo_riemannian**: P/L/R operation probabilities per valence zone. Controls harmonic color.
- **voice_leading**: Max semitone movement per voice, tension filtering thresholds.
- **melody**: Fractal contour, motif repetition, gap-fill behavior.
- **classic_groove**: Drum pattern density thresholds and velocities for realistic grooves.
- **perfect_balance**: Polygon-based rhythm with hat vertex counts and swing.
- **arrangement**: Energy thresholds, fill zones, harmonic rhythm rate, crash/ghost velocities.
- **emotional_quadrant**: Valence/tension thresholds for basic chord palette selection.

## Important
- Only change parameters that this style specifically requires. Keep others at defaults.
- All f32 values in 0.0-1.0 range unless documented otherwise.
- hat_vertex_counts are [very_low, low, mid, high] density polygon counts.
- kick_density_thresholds are [sparse, secondary, straight, anticipation] breakpoints.

Return ONLY valid JSON (no markdown, no explanation before/after) in this exact format:
{{"reasoning": "Your musical analysis of why these parameter values suit this style", "tuning": {{ ... complete TuningParams object ... }}, "confidence": 0.85}}"#
        )
    }

    /// Build prompt for refinement based on rating and feedback.
    fn build_refinement_prompt(
        name: &str,
        description: &str,
        current: &TuningParams,
        rating: u8,
        feedback: &str,
    ) -> String {
        let current_json =
            serde_json::to_string_pretty(current).unwrap_or_else(|_| "{}".to_string());

        format!(
            r#"You are an expert music producer tuning the Harmonium generative music engine.

## Style
Name: "{name}"
Description: {description}

## Previous TuningParams (scored {rating}/5)
```json
{current_json}
```

## Human Feedback
Rating: {rating}/5
Comments: "{feedback}"

## Task
Adjust the TuningParams to address the feedback. A rating of 5 means perfect — make minimal changes. A rating of 1 means major rework needed.

Return ONLY valid JSON (no markdown) in this exact format:
{{"reasoning": "What you changed and why based on the feedback", "tuning": {{ ... complete TuningParams object ... }}, "confidence": 0.85}}"#
        )
    }

    /// Parse a style generation/refinement response from Claude.
    fn parse_style_response(response_text: &str) -> Result<StyleGenerationResult, AgentError> {
        let json_start = response_text.find('{');
        let json_end = response_text.rfind('}');

        let json_str = match (json_start, json_end) {
            (Some(start), Some(end)) if end > start => &response_text[start..=end],
            _ => {
                return Err(AgentError::ParseError("No JSON object found in response".to_string()));
            }
        };

        let raw: RawStyleResponse = serde_json::from_str(json_str)
            .map_err(|e| AgentError::ParseError(format!("JSON parse error: {}", e)))?;

        Ok(StyleGenerationResult {
            reasoning: raw.reasoning,
            tuning: raw.tuning,
            confidence: raw.confidence.unwrap_or(0.5),
        })
    }

    /// Generate a direction indicator (up/down/equal)
    fn direction_indicator(generated: f32, reference: f32) -> &'static str {
        let diff = generated - reference;
        if diff.abs() < 0.01 {
            "="
        } else if diff > 0.0 {
            "↑ too high"
        } else {
            "↓ too low"
        }
    }
}

/// Raw suggestion from Claude (before processing)
#[derive(Deserialize)]
struct RawSuggestion {
    reasoning: String,
    changes: serde_json::Value,
    confidence: Option<f32>,
}

/// Raw style generation response from Claude (before processing)
#[derive(Deserialize)]
struct RawStyleResponse {
    reasoning: String,
    tuning: TuningParams,
    confidence: Option<f32>,
}

/// Result of LLM-assisted style generation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyleGenerationResult {
    /// Claude's reasoning about why these parameter values suit the style
    pub reasoning: String,
    /// The generated TuningParams
    pub tuning: TuningParams,
    /// Confidence level (0.0-1.0)
    pub confidence: f32,
}

/// Format a JSON value as a string
fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                format!("{:.2}", f)
            } else {
                n.to_string()
            }
        }
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

/// A tuning suggestion from Claude
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TuningSuggestion {
    /// Reasoning behind the suggestion
    pub reasoning: String,

    /// Suggested parameter changes
    pub parameter_changes: Vec<ParameterChange>,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
}

impl TuningSuggestion {
    /// Apply this suggestion to a TuningParams, returning a new modified copy
    #[must_use]
    pub fn apply_to(&self, tuning: &TuningParams) -> TuningParams {
        let mut new_tuning = tuning.clone();

        for change in &self.parameter_changes {
            self.apply_change(&mut new_tuning, change);
        }

        new_tuning
    }

    /// Apply a single parameter change
    fn apply_change(&self, tuning: &mut TuningParams, change: &ParameterChange) {
        match change.name.as_str() {
            "max_semitone_movement" => {
                if let Ok(v) = change.suggested.parse::<u8>() {
                    tuning.voice_leading.max_semitone_movement = v;
                }
            }
            "cardinality_morph_enabled" => {
                if let Ok(v) = change.suggested.parse::<bool>() {
                    tuning.voice_leading.allow_cardinality_morph = v;
                }
            }
            "trq_threshold" => {
                if let Ok(v) = change.suggested.parse::<f32>() {
                    tuning.voice_leading.trq_threshold = v;
                }
            }
            "steedman_lower_threshold" => {
                if let Ok(v) = change.suggested.parse::<f32>() {
                    tuning.harmony_driver.steedman_lower = v;
                }
            }
            "steedman_upper_threshold" => {
                if let Ok(v) = change.suggested.parse::<f32>() {
                    tuning.harmony_driver.steedman_upper = v;
                }
            }
            "neo_riemannian_lower_threshold" => {
                if let Ok(v) = change.suggested.parse::<f32>() {
                    tuning.harmony_driver.neo_lower = v;
                }
            }
            "neo_riemannian_upper_threshold" => {
                if let Ok(v) = change.suggested.parse::<f32>() {
                    tuning.harmony_driver.neo_upper = v;
                }
            }
            "hysteresis_boost" => {
                if let Ok(v) = change.suggested.parse::<f32>() {
                    tuning.harmony_driver.hysteresis_boost = v;
                }
            }
            "kick_density_threshold" => {
                if let Ok(v) = change.suggested.parse::<f32>() {
                    tuning.perfect_balance.kick_polygon_low_threshold = v;
                }
            }
            "hat_very_low_density_vertices" => {
                if let Ok(v) = change.suggested.parse::<usize>() {
                    tuning.perfect_balance.hat_vertex_counts[0] = v;
                }
            }
            "hat_low_density_vertices" => {
                if let Ok(v) = change.suggested.parse::<usize>() {
                    tuning.perfect_balance.hat_vertex_counts[1] = v;
                }
            }
            "hat_medium_density_vertices" => {
                if let Ok(v) = change.suggested.parse::<usize>() {
                    tuning.perfect_balance.hat_vertex_counts[2] = v;
                }
            }
            "hat_high_density_vertices" => {
                if let Ok(v) = change.suggested.parse::<usize>() {
                    tuning.perfect_balance.hat_vertex_counts[3] = v;
                }
            }
            _ => {} // Unknown parameter, ignore
        }
    }

    /// Check if this suggestion has any changes
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.parameter_changes.is_empty()
    }
}

/// A single parameter change suggestion
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterChange {
    /// Parameter name
    pub name: String,

    /// Current value (as string for display)
    pub current: String,

    /// Suggested new value (as string for display)
    pub suggested: String,

    /// Reason for this specific change
    pub reason: String,
}

impl std::fmt::Display for ParameterChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} → {}", self.name, self.current, self.suggested)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = ClaudeAgent::new();
        assert!(!agent.is_configured());
    }

    #[test]
    fn test_agent_with_key() {
        let agent = ClaudeAgent::new().with_api_key("test-key");
        assert!(agent.is_configured());
    }

    #[test]
    fn test_direction_indicator() {
        assert_eq!(ClaudeAgent::direction_indicator(2.0, 1.0), "↑ too high");
        assert_eq!(ClaudeAgent::direction_indicator(1.0, 2.0), "↓ too low");
        assert_eq!(ClaudeAgent::direction_indicator(1.0, 1.005), "=");
    }

    #[test]
    fn test_prompt_generation() {
        let agent = ClaudeAgent::new();
        let reference = GlobalMetrics::default();
        let generated = GlobalMetrics::default();
        let tuning = TuningParams::default();

        let prompt = agent.build_prompt(&reference, &generated, &tuning);

        assert!(prompt.contains("Target Style Profile"));
        assert!(prompt.contains("Generated Music Profile"));
        assert!(prompt.contains("Current Tuning Parameters"));
        assert!(prompt.contains("max_semitone_movement"));
    }

    #[test]
    fn test_parse_suggestion() {
        let agent = ClaudeAgent::new();
        let tuning = TuningParams::default();

        let response = r#"{"reasoning": "Voice leading is too high", "changes": {"max_semitone_movement": 1}, "confidence": 0.8}"#;

        let suggestion = agent.parse_suggestion(response, &tuning).unwrap();

        assert_eq!(suggestion.confidence, 0.8);
        assert!(suggestion.reasoning.contains("Voice leading"));
        assert!(suggestion.has_changes());
    }

    #[test]
    fn test_apply_suggestion() {
        let tuning = TuningParams::default();
        let suggestion = TuningSuggestion {
            reasoning: "Test".to_string(),
            parameter_changes: vec![ParameterChange {
                name: "max_semitone_movement".to_string(),
                current: "2".to_string(),
                suggested: "1".to_string(),
                reason: "".to_string(),
            }],
            confidence: 0.8,
        };

        let new_tuning = suggestion.apply_to(&tuning);
        assert_eq!(new_tuning.voice_leading.max_semitone_movement, 1);
    }

    // -----------------------------------------------------------------------
    // Style generation prompt & parsing tests (CORELIB-26)
    // -----------------------------------------------------------------------

    #[test]
    fn test_style_prompt_contains_schema() {
        let prompt = ClaudeAgent::build_style_prompt(
            "Bossa Nova",
            "straight 8ths, anticipated bass, sparse comping",
        );

        // Prompt must contain the full TuningParams schema
        assert!(prompt.contains("harmony_driver"));
        assert!(prompt.contains("grammar"));
        assert!(prompt.contains("neo_riemannian"));
        assert!(prompt.contains("voice_leading"));
        assert!(prompt.contains("melody"));
        assert!(prompt.contains("classic_groove"));
        assert!(prompt.contains("perfect_balance"));
        assert!(prompt.contains("arrangement"));
        assert!(prompt.contains("emotional_quadrant"));
        // Must contain the style info
        assert!(prompt.contains("Bossa Nova"));
        assert!(prompt.contains("anticipated bass"));
        // Must contain default values so Claude knows the baseline
        assert!(prompt.contains("0.45")); // steedman_lower default
    }

    #[test]
    fn test_refinement_prompt_includes_rating_and_feedback() {
        let tuning = TuningParams::default();
        let prompt = ClaudeAgent::build_refinement_prompt(
            "Swing",
            "medium swing feel",
            &tuning,
            3,
            "bass too busy, hat too loud",
        );

        assert!(prompt.contains("scored 3/5"));
        assert!(prompt.contains("bass too busy"));
        assert!(prompt.contains("Swing"));
    }

    /// Simulate a realistic Bossa Nova response from Claude and parse it.
    #[test]
    fn test_parse_bossa_nova_style_response() {
        let response = r#"{"reasoning": "Bossa Nova uses straight 8ths (no swing), anticipated bass patterns, sparse ghost notes, and warm jazz harmony with moderate tension. Key changes: lower hat density for sparser feel, reduce ghost note velocity, widen Steedman zone for more functional harmony, increase hurst for smoother melody.", "tuning": {"harmony_driver": {"steedman_lower": 0.40, "steedman_upper": 0.60, "neo_lower": 0.70, "neo_upper": 0.80, "hysteresis_boost": 0.12, "dramatic_tension_drop_upper": 0.7, "dramatic_tension_drop_lower": 0.5, "cadential_resolution_probability": 0.65, "max_retries": 3}, "grammar": {"grammar_style": "Jazz", "max_recursion_depth": 2, "preparation_weight": 1.3, "backcycle_weight": 0.8, "tritone_sub_weight": 0.6, "cadential_weight": 1.0, "deceptive_weight": 0.7, "modal_interchange_weight": 0.5, "chord_quality_valence_threshold": 0.3}, "neo_riemannian": {"positive_valence_threshold": 0.3, "negative_valence_threshold": -0.3, "positive_r_prob": 0.5, "positive_p_cumulative": 0.8, "negative_l_prob": 0.5, "negative_p_cumulative": 0.8, "neutral_p_prob": 0.4, "neutral_l_cumulative": 0.7, "composite_tension_threshold": 0.85, "composite_probability": 0.3}, "voice_leading": {"max_semitone_movement": 2, "allow_cardinality_morph": true, "trq_threshold": 0.45, "high_tension_threshold": 0.6, "low_tension_threshold": 0.4}, "melody": {"pink_noise_depth": 5, "default_hurst_factor": 0.8, "gap_fill_threshold": 4, "motif_new_material_bias": 0.5, "fractal_boost": 1.5, "fractal_range": 18.0, "consecutive_direction_limit": 5, "leading_tone_resolution_weight": 55}, "classic_groove": {"kick_density_thresholds": [0.2, 0.35, 0.55, 0.75], "kick_downbeat_velocity": 0.9, "kick_secondary_velocity": 0.75, "kick_anticipation_velocity": 0.65, "ghost_note_tension_threshold": 0.35, "ghost_note_velocity": 0.18, "hat_density_thresholds": [0.25, 0.55, 0.8], "hat_on_beat_velocity": 0.5, "hat_off_beat_velocity": 0.35, "hat_dense_on_velocity": 0.55, "hat_dense_off_velocity": 0.4, "hat_dense_ghost_velocity": 0.25, "hat_sparse_velocity": 0.45, "hat_masking_density_threshold": 0.7, "bass_split_tension_threshold": 0.35, "snare_backbeat_velocity": 0.85}, "perfect_balance": {"kick_polygon_low_threshold": 0.25, "kick_low_velocity": 0.9, "kick_normal_velocity": 0.9, "snare_velocity": 0.8, "hat_vertex_counts": [6, 8, 10, 14], "hat_density_thresholds": [0.2, 0.55, 0.8], "hat_velocity_coefficient": 0.5, "swing_tension_threshold": 0.4, "swing_scaling_factor": 0.3, "bass_polygon_velocity": 0.75, "bass_low_density_threshold": 0.35, "lead_polygon_velocity": 0.65, "hat_masking_density_threshold": 0.7}, "arrangement": {"fill_zone_size": 3, "crash_velocity": 95, "ghost_velocity_factor": 0.55, "tom_velocity_boost": 1.0, "progression_switch_interval_slow": 4, "progression_switch_interval_normal": 2, "progression_switch_interval_fast": 1, "progression_tension_thresholds": [0.2, 0.45], "measures_per_chord_hysteresis": 0.35, "energy_high_tension": 0.65, "energy_high_density": 0.6, "energy_high_arousal": 0.75}, "emotional_quadrant": {"happy_valence_threshold": 0.25, "sad_valence_threshold": -0.25, "energetic_tension_threshold": 0.55}}, "confidence": 0.82}"#;

        let result = ClaudeAgent::parse_style_response(response).expect("should parse");

        // Check metadata
        assert!(result.reasoning.contains("Bossa Nova"));
        assert!((result.confidence - 0.82).abs() < 0.01);

        // Validate the generated TuningParams
        let tp = &result.tuning;
        tp.validate().expect("generated params should be valid");

        // Musical consistency checks for Bossa Nova:
        // 1. Wider Steedman zone (more functional harmony)
        assert!(tp.harmony_driver.steedman_upper > tp.harmony_driver.steedman_lower);
        assert!(tp.harmony_driver.steedman_upper >= 0.55, "Bossa needs wide functional zone");

        // 2. Higher hurst = smoother melody
        assert!(tp.melody.default_hurst_factor >= 0.75, "Bossa melody should be smooth");

        // 3. Lower hat velocities (sparse feel)
        assert!(tp.classic_groove.hat_on_beat_velocity <= 0.55, "Bossa hats should be soft");

        // 4. Lower ghost note velocity (subtle)
        assert!(tp.classic_groove.ghost_note_velocity < 0.25, "Ghost notes should be subtle");

        // 5. Jazz grammar style
        assert_eq!(tp.grammar.grammar_style, harmonium_core::models::jazz::GrammarStyle::Jazz);

        // 6. Lower crash velocity (not aggressive)
        assert!(tp.arrangement.crash_velocity < 110, "Bossa crashes should be gentle");

        // 7. Slower harmonic rhythm (more space)
        assert!(tp.arrangement.progression_switch_interval_slow >= 3);
    }

    /// Simulate a Medium Swing response — different character from Bossa.
    #[test]
    fn test_parse_swing_style_response() {
        let response = r#"{"reasoning": "Medium swing has shuffle feel, walking bass, strong backbeat on 2 and 4, ride cymbal pattern, moderate complexity.", "tuning": {"harmony_driver": {"steedman_lower": 0.42, "steedman_upper": 0.52, "neo_lower": 0.62, "neo_upper": 0.72, "hysteresis_boost": 0.1, "dramatic_tension_drop_upper": 0.7, "dramatic_tension_drop_lower": 0.5, "cadential_resolution_probability": 0.6, "max_retries": 3}, "grammar": {"grammar_style": "Jazz", "max_recursion_depth": 3, "preparation_weight": 1.5, "backcycle_weight": 1.3, "tritone_sub_weight": 1.2, "cadential_weight": 0.8, "deceptive_weight": 1.0, "modal_interchange_weight": 0.8, "chord_quality_valence_threshold": 0.3}, "neo_riemannian": {"positive_valence_threshold": 0.3, "negative_valence_threshold": -0.3, "positive_r_prob": 0.5, "positive_p_cumulative": 0.8, "negative_l_prob": 0.5, "negative_p_cumulative": 0.8, "neutral_p_prob": 0.4, "neutral_l_cumulative": 0.7, "composite_tension_threshold": 0.8, "composite_probability": 0.5}, "voice_leading": {"max_semitone_movement": 2, "allow_cardinality_morph": true, "trq_threshold": 0.5, "high_tension_threshold": 0.6, "low_tension_threshold": 0.4}, "melody": {"pink_noise_depth": 5, "default_hurst_factor": 0.65, "gap_fill_threshold": 6, "motif_new_material_bias": 0.7, "fractal_boost": 2.0, "fractal_range": 25.0, "consecutive_direction_limit": 7, "leading_tone_resolution_weight": 45}, "classic_groove": {"kick_density_thresholds": [0.25, 0.4, 0.6, 0.8], "kick_downbeat_velocity": 1.0, "kick_secondary_velocity": 0.85, "kick_anticipation_velocity": 0.7, "ghost_note_tension_threshold": 0.25, "ghost_note_velocity": 0.3, "hat_density_thresholds": [0.3, 0.6, 0.85], "hat_on_beat_velocity": 0.65, "hat_off_beat_velocity": 0.45, "hat_dense_on_velocity": 0.7, "hat_dense_off_velocity": 0.5, "hat_dense_ghost_velocity": 0.35, "hat_sparse_velocity": 0.55, "hat_masking_density_threshold": 0.75, "bass_split_tension_threshold": 0.4, "snare_backbeat_velocity": 1.0}, "perfect_balance": {"kick_polygon_low_threshold": 0.3, "kick_low_velocity": 1.0, "kick_normal_velocity": 1.0, "snare_velocity": 0.95, "hat_vertex_counts": [6, 8, 12, 16], "hat_density_thresholds": [0.25, 0.6, 0.85], "hat_velocity_coefficient": 0.65, "swing_tension_threshold": 0.2, "swing_scaling_factor": 0.6, "bass_polygon_velocity": 0.85, "bass_low_density_threshold": 0.35, "lead_polygon_velocity": 0.75, "hat_masking_density_threshold": 0.75}, "arrangement": {"fill_zone_size": 4, "crash_velocity": 110, "ghost_velocity_factor": 0.65, "tom_velocity_boost": 1.1, "progression_switch_interval_slow": 2, "progression_switch_interval_normal": 1, "progression_switch_interval_fast": 1, "progression_tension_thresholds": [0.2, 0.4], "measures_per_chord_hysteresis": 0.4, "energy_high_tension": 0.6, "energy_high_density": 0.6, "energy_high_arousal": 0.7}, "emotional_quadrant": {"happy_valence_threshold": 0.3, "sad_valence_threshold": -0.3, "energetic_tension_threshold": 0.6}}, "confidence": 0.88}"#;

        let result = ClaudeAgent::parse_style_response(response).expect("should parse");
        let tp = &result.tuning;
        tp.validate().expect("swing params should be valid");

        // Swing-specific checks:
        // 1. Deeper recursion for jazz ii-V chains
        assert!(tp.grammar.max_recursion_depth >= 3);

        // 2. Higher preparation weight (ii-V is central to swing)
        assert!(tp.grammar.preparation_weight >= 1.3);

        // 3. More motif variety (bebop improvisation)
        assert!(tp.melody.motif_new_material_bias >= 0.65);

        // 4. Swing factor should be present (non-zero)
        assert!(tp.perfect_balance.swing_scaling_factor >= 0.5, "Swing needs swing factor");

        // 5. Faster harmonic rhythm than Bossa
        assert!(tp.arrangement.progression_switch_interval_slow <= 2);

        // 6. Strong snare backbeat
        assert!(tp.classic_groove.snare_backbeat_velocity >= 0.95);
    }

    /// Test that the refinement prompt round-trips: we can parse a refined response.
    #[test]
    fn test_parse_refinement_response() {
        // Simulate: Bossa was rated 3/5, "bass too busy"
        let response = r#"{"reasoning": "Reduced bass polygon velocity and increased bass_low_density_threshold so bass follows kick more, creating space. Also lowered kick anticipation velocity.", "tuning": {"harmony_driver": {"steedman_lower": 0.40, "steedman_upper": 0.60, "neo_lower": 0.70, "neo_upper": 0.80, "hysteresis_boost": 0.12, "dramatic_tension_drop_upper": 0.7, "dramatic_tension_drop_lower": 0.5, "cadential_resolution_probability": 0.65, "max_retries": 3}, "grammar": {"grammar_style": "Jazz", "max_recursion_depth": 2, "preparation_weight": 1.3, "backcycle_weight": 0.8, "tritone_sub_weight": 0.6, "cadential_weight": 1.0, "deceptive_weight": 0.7, "modal_interchange_weight": 0.5, "chord_quality_valence_threshold": 0.3}, "neo_riemannian": {"positive_valence_threshold": 0.3, "negative_valence_threshold": -0.3, "positive_r_prob": 0.5, "positive_p_cumulative": 0.8, "negative_l_prob": 0.5, "negative_p_cumulative": 0.8, "neutral_p_prob": 0.4, "neutral_l_cumulative": 0.7, "composite_tension_threshold": 0.85, "composite_probability": 0.3}, "voice_leading": {"max_semitone_movement": 2, "allow_cardinality_morph": true, "trq_threshold": 0.45, "high_tension_threshold": 0.6, "low_tension_threshold": 0.4}, "melody": {"pink_noise_depth": 5, "default_hurst_factor": 0.8, "gap_fill_threshold": 4, "motif_new_material_bias": 0.5, "fractal_boost": 1.5, "fractal_range": 18.0, "consecutive_direction_limit": 5, "leading_tone_resolution_weight": 55}, "classic_groove": {"kick_density_thresholds": [0.2, 0.35, 0.55, 0.75], "kick_downbeat_velocity": 0.85, "kick_secondary_velocity": 0.7, "kick_anticipation_velocity": 0.55, "ghost_note_tension_threshold": 0.35, "ghost_note_velocity": 0.15, "hat_density_thresholds": [0.25, 0.55, 0.8], "hat_on_beat_velocity": 0.5, "hat_off_beat_velocity": 0.35, "hat_dense_on_velocity": 0.55, "hat_dense_off_velocity": 0.4, "hat_dense_ghost_velocity": 0.25, "hat_sparse_velocity": 0.45, "hat_masking_density_threshold": 0.7, "bass_split_tension_threshold": 0.3, "snare_backbeat_velocity": 0.85}, "perfect_balance": {"kick_polygon_low_threshold": 0.25, "kick_low_velocity": 0.85, "kick_normal_velocity": 0.85, "snare_velocity": 0.8, "hat_vertex_counts": [6, 8, 10, 14], "hat_density_thresholds": [0.2, 0.55, 0.8], "hat_velocity_coefficient": 0.5, "swing_tension_threshold": 0.4, "swing_scaling_factor": 0.3, "bass_polygon_velocity": 0.65, "bass_low_density_threshold": 0.45, "lead_polygon_velocity": 0.6, "hat_masking_density_threshold": 0.7}, "arrangement": {"fill_zone_size": 3, "crash_velocity": 90, "ghost_velocity_factor": 0.5, "tom_velocity_boost": 1.0, "progression_switch_interval_slow": 4, "progression_switch_interval_normal": 2, "progression_switch_interval_fast": 1, "progression_tension_thresholds": [0.2, 0.45], "measures_per_chord_hysteresis": 0.35, "energy_high_tension": 0.65, "energy_high_density": 0.6, "energy_high_arousal": 0.75}, "emotional_quadrant": {"happy_valence_threshold": 0.25, "sad_valence_threshold": -0.25, "energetic_tension_threshold": 0.55}}, "confidence": 0.78}"#;

        let result = ClaudeAgent::parse_style_response(response).expect("should parse refinement");
        let tp = &result.tuning;
        tp.validate().expect("refined params should be valid");

        // Refinement addressed "bass too busy":
        assert!(
            tp.perfect_balance.bass_polygon_velocity < 0.7,
            "bass should be quieter after feedback"
        );
        assert!(
            tp.perfect_balance.bass_low_density_threshold > 0.4,
            "bass should follow kick more"
        );
        assert!(tp.classic_groove.kick_anticipation_velocity < 0.6, "kick anticipation reduced");
        assert!(result.reasoning.contains("bass"));
    }

    /// Verify TOML round-trip: generate → save → reload produces identical params.
    #[test]
    fn test_style_toml_roundtrip() {
        let response = r#"{"reasoning": "test", "tuning": {"harmony_driver": {"steedman_lower": 0.40, "steedman_upper": 0.60, "neo_lower": 0.70, "neo_upper": 0.80, "hysteresis_boost": 0.12, "dramatic_tension_drop_upper": 0.7, "dramatic_tension_drop_lower": 0.5, "cadential_resolution_probability": 0.65, "max_retries": 3}, "grammar": {"grammar_style": "Jazz", "max_recursion_depth": 2, "preparation_weight": 1.3, "backcycle_weight": 0.8, "tritone_sub_weight": 0.6, "cadential_weight": 1.0, "deceptive_weight": 0.7, "modal_interchange_weight": 0.5, "chord_quality_valence_threshold": 0.3}, "neo_riemannian": {"positive_valence_threshold": 0.3, "negative_valence_threshold": -0.3, "positive_r_prob": 0.5, "positive_p_cumulative": 0.8, "negative_l_prob": 0.5, "negative_p_cumulative": 0.8, "neutral_p_prob": 0.4, "neutral_l_cumulative": 0.7, "composite_tension_threshold": 0.85, "composite_probability": 0.3}, "voice_leading": {"max_semitone_movement": 2, "allow_cardinality_morph": true, "trq_threshold": 0.45, "high_tension_threshold": 0.6, "low_tension_threshold": 0.4}, "melody": {"pink_noise_depth": 5, "default_hurst_factor": 0.8, "gap_fill_threshold": 4, "motif_new_material_bias": 0.5, "fractal_boost": 1.5, "fractal_range": 18.0, "consecutive_direction_limit": 5, "leading_tone_resolution_weight": 55}, "classic_groove": {"kick_density_thresholds": [0.2, 0.35, 0.55, 0.75], "kick_downbeat_velocity": 0.9, "kick_secondary_velocity": 0.75, "kick_anticipation_velocity": 0.65, "ghost_note_tension_threshold": 0.35, "ghost_note_velocity": 0.18, "hat_density_thresholds": [0.25, 0.55, 0.8], "hat_on_beat_velocity": 0.5, "hat_off_beat_velocity": 0.35, "hat_dense_on_velocity": 0.55, "hat_dense_off_velocity": 0.4, "hat_dense_ghost_velocity": 0.25, "hat_sparse_velocity": 0.45, "hat_masking_density_threshold": 0.7, "bass_split_tension_threshold": 0.35, "snare_backbeat_velocity": 0.85}, "perfect_balance": {"kick_polygon_low_threshold": 0.25, "kick_low_velocity": 0.9, "kick_normal_velocity": 0.9, "snare_velocity": 0.8, "hat_vertex_counts": [6, 8, 10, 14], "hat_density_thresholds": [0.2, 0.55, 0.8], "hat_velocity_coefficient": 0.5, "swing_tension_threshold": 0.4, "swing_scaling_factor": 0.3, "bass_polygon_velocity": 0.75, "bass_low_density_threshold": 0.35, "lead_polygon_velocity": 0.65, "hat_masking_density_threshold": 0.7}, "arrangement": {"fill_zone_size": 3, "crash_velocity": 95, "ghost_velocity_factor": 0.55, "tom_velocity_boost": 1.0, "progression_switch_interval_slow": 4, "progression_switch_interval_normal": 2, "progression_switch_interval_fast": 1, "progression_tension_thresholds": [0.2, 0.45], "measures_per_chord_hysteresis": 0.35, "energy_high_tension": 0.65, "energy_high_density": 0.6, "energy_high_arousal": 0.75}, "emotional_quadrant": {"happy_valence_threshold": 0.25, "sad_valence_threshold": -0.25, "energetic_tension_threshold": 0.55}}, "confidence": 0.9}"#;

        let result = ClaudeAgent::parse_style_response(response).expect("parse");
        let tp = result.tuning;

        // Save to TOML and reload
        let toml_str = toml::to_string_pretty(&tp).expect("serialize to TOML");
        let reloaded: TuningParams = toml::from_str(&toml_str).expect("deserialize from TOML");

        assert_eq!(tp, reloaded, "TOML round-trip must be lossless");
    }

    /// Verify that two different styles produce meaningfully different params.
    #[test]
    fn test_styles_are_distinct() {
        let bossa_response = r#"{"reasoning": "bossa", "tuning": {"harmony_driver": {"steedman_lower": 0.40, "steedman_upper": 0.60, "neo_lower": 0.70, "neo_upper": 0.80, "hysteresis_boost": 0.12, "dramatic_tension_drop_upper": 0.7, "dramatic_tension_drop_lower": 0.5, "cadential_resolution_probability": 0.65, "max_retries": 3}, "grammar": {"grammar_style": "Jazz", "max_recursion_depth": 2, "preparation_weight": 1.3, "backcycle_weight": 0.8, "tritone_sub_weight": 0.6, "cadential_weight": 1.0, "deceptive_weight": 0.7, "modal_interchange_weight": 0.5, "chord_quality_valence_threshold": 0.3}, "neo_riemannian": {"positive_valence_threshold": 0.3, "negative_valence_threshold": -0.3, "positive_r_prob": 0.5, "positive_p_cumulative": 0.8, "negative_l_prob": 0.5, "negative_p_cumulative": 0.8, "neutral_p_prob": 0.4, "neutral_l_cumulative": 0.7, "composite_tension_threshold": 0.85, "composite_probability": 0.3}, "voice_leading": {"max_semitone_movement": 2, "allow_cardinality_morph": true, "trq_threshold": 0.45, "high_tension_threshold": 0.6, "low_tension_threshold": 0.4}, "melody": {"pink_noise_depth": 5, "default_hurst_factor": 0.8, "gap_fill_threshold": 4, "motif_new_material_bias": 0.5, "fractal_boost": 1.5, "fractal_range": 18.0, "consecutive_direction_limit": 5, "leading_tone_resolution_weight": 55}, "classic_groove": {"kick_density_thresholds": [0.2, 0.35, 0.55, 0.75], "kick_downbeat_velocity": 0.9, "kick_secondary_velocity": 0.75, "kick_anticipation_velocity": 0.65, "ghost_note_tension_threshold": 0.35, "ghost_note_velocity": 0.18, "hat_density_thresholds": [0.25, 0.55, 0.8], "hat_on_beat_velocity": 0.5, "hat_off_beat_velocity": 0.35, "hat_dense_on_velocity": 0.55, "hat_dense_off_velocity": 0.4, "hat_dense_ghost_velocity": 0.25, "hat_sparse_velocity": 0.45, "hat_masking_density_threshold": 0.7, "bass_split_tension_threshold": 0.35, "snare_backbeat_velocity": 0.85}, "perfect_balance": {"kick_polygon_low_threshold": 0.25, "kick_low_velocity": 0.9, "kick_normal_velocity": 0.9, "snare_velocity": 0.8, "hat_vertex_counts": [6, 8, 10, 14], "hat_density_thresholds": [0.2, 0.55, 0.8], "hat_velocity_coefficient": 0.5, "swing_tension_threshold": 0.4, "swing_scaling_factor": 0.3, "bass_polygon_velocity": 0.75, "bass_low_density_threshold": 0.35, "lead_polygon_velocity": 0.65, "hat_masking_density_threshold": 0.7}, "arrangement": {"fill_zone_size": 3, "crash_velocity": 95, "ghost_velocity_factor": 0.55, "tom_velocity_boost": 1.0, "progression_switch_interval_slow": 4, "progression_switch_interval_normal": 2, "progression_switch_interval_fast": 1, "progression_tension_thresholds": [0.2, 0.45], "measures_per_chord_hysteresis": 0.35, "energy_high_tension": 0.65, "energy_high_density": 0.6, "energy_high_arousal": 0.75}, "emotional_quadrant": {"happy_valence_threshold": 0.25, "sad_valence_threshold": -0.25, "energetic_tension_threshold": 0.55}}, "confidence": 0.82}"#;

        let swing_response = r#"{"reasoning": "swing", "tuning": {"harmony_driver": {"steedman_lower": 0.42, "steedman_upper": 0.52, "neo_lower": 0.62, "neo_upper": 0.72, "hysteresis_boost": 0.1, "dramatic_tension_drop_upper": 0.7, "dramatic_tension_drop_lower": 0.5, "cadential_resolution_probability": 0.6, "max_retries": 3}, "grammar": {"grammar_style": "Jazz", "max_recursion_depth": 3, "preparation_weight": 1.5, "backcycle_weight": 1.3, "tritone_sub_weight": 1.2, "cadential_weight": 0.8, "deceptive_weight": 1.0, "modal_interchange_weight": 0.8, "chord_quality_valence_threshold": 0.3}, "neo_riemannian": {"positive_valence_threshold": 0.3, "negative_valence_threshold": -0.3, "positive_r_prob": 0.5, "positive_p_cumulative": 0.8, "negative_l_prob": 0.5, "negative_p_cumulative": 0.8, "neutral_p_prob": 0.4, "neutral_l_cumulative": 0.7, "composite_tension_threshold": 0.8, "composite_probability": 0.5}, "voice_leading": {"max_semitone_movement": 2, "allow_cardinality_morph": true, "trq_threshold": 0.5, "high_tension_threshold": 0.6, "low_tension_threshold": 0.4}, "melody": {"pink_noise_depth": 5, "default_hurst_factor": 0.65, "gap_fill_threshold": 6, "motif_new_material_bias": 0.7, "fractal_boost": 2.0, "fractal_range": 25.0, "consecutive_direction_limit": 7, "leading_tone_resolution_weight": 45}, "classic_groove": {"kick_density_thresholds": [0.25, 0.4, 0.6, 0.8], "kick_downbeat_velocity": 1.0, "kick_secondary_velocity": 0.85, "kick_anticipation_velocity": 0.7, "ghost_note_tension_threshold": 0.25, "ghost_note_velocity": 0.3, "hat_density_thresholds": [0.3, 0.6, 0.85], "hat_on_beat_velocity": 0.65, "hat_off_beat_velocity": 0.45, "hat_dense_on_velocity": 0.7, "hat_dense_off_velocity": 0.5, "hat_dense_ghost_velocity": 0.35, "hat_sparse_velocity": 0.55, "hat_masking_density_threshold": 0.75, "bass_split_tension_threshold": 0.4, "snare_backbeat_velocity": 1.0}, "perfect_balance": {"kick_polygon_low_threshold": 0.3, "kick_low_velocity": 1.0, "kick_normal_velocity": 1.0, "snare_velocity": 0.95, "hat_vertex_counts": [6, 8, 12, 16], "hat_density_thresholds": [0.25, 0.6, 0.85], "hat_velocity_coefficient": 0.65, "swing_tension_threshold": 0.2, "swing_scaling_factor": 0.6, "bass_polygon_velocity": 0.85, "bass_low_density_threshold": 0.35, "lead_polygon_velocity": 0.75, "hat_masking_density_threshold": 0.75}, "arrangement": {"fill_zone_size": 4, "crash_velocity": 110, "ghost_velocity_factor": 0.65, "tom_velocity_boost": 1.1, "progression_switch_interval_slow": 2, "progression_switch_interval_normal": 1, "progression_switch_interval_fast": 1, "progression_tension_thresholds": [0.2, 0.4], "measures_per_chord_hysteresis": 0.4, "energy_high_tension": 0.6, "energy_high_density": 0.6, "energy_high_arousal": 0.7}, "emotional_quadrant": {"happy_valence_threshold": 0.3, "sad_valence_threshold": -0.3, "energetic_tension_threshold": 0.6}}, "confidence": 0.88}"#;

        let bossa = ClaudeAgent::parse_style_response(bossa_response).expect("parse bossa");
        let swing = ClaudeAgent::parse_style_response(swing_response).expect("parse swing");

        // They should be different in meaningful ways
        assert_ne!(bossa.tuning, swing.tuning, "two styles must produce different params");

        // Bossa should be smoother melody than Swing
        assert!(
            bossa.tuning.melody.default_hurst_factor > swing.tuning.melody.default_hurst_factor
        );

        // Swing should have more swing factor
        assert!(
            swing.tuning.perfect_balance.swing_scaling_factor
                > bossa.tuning.perfect_balance.swing_scaling_factor
        );

        // Swing should have louder snare
        assert!(
            swing.tuning.classic_groove.snare_backbeat_velocity
                > bossa.tuning.classic_groove.snare_backbeat_velocity
        );

        // Bossa should have slower harmonic rhythm
        assert!(
            bossa.tuning.arrangement.progression_switch_interval_slow
                > swing.tuning.arrangement.progression_switch_interval_slow
        );
    }
}
