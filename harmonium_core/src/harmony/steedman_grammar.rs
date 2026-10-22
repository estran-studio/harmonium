//! Adaptateur de la grammaire neutre pour le driver Legacy, conservé jusqu'à P2-T5c.
use std::sync::{Arc, RwLock};

use super::{HarmonyContext, HarmonyDecision, HarmonyStrategy, RngCore, TransitionType};
use crate::{
    models::jazz::{GrammarStyle, SteedmanGrammar as Grammar},
    theory::lydian_chromatic::LydianChromaticConcept,
};

pub struct SteedmanGrammar {
    grammar: RwLock<Grammar>,
    lcc: Arc<LydianChromaticConcept>,
}
impl SteedmanGrammar {
    #[must_use]
    pub fn new(lcc: Arc<LydianChromaticConcept>) -> Self {
        Self { grammar: RwLock::new(Grammar::default()), lcc }
    }
    pub fn set_style(&self, style: GrammarStyle) {
        if let Ok(mut grammar) = self.grammar.write() {
            grammar.set_style(style);
        }
    }
}
impl HarmonyStrategy for SteedmanGrammar {
    fn next_chord(&self, ctx: &HarmonyContext, rng: &mut dyn RngCore) -> HarmonyDecision {
        let mut grammar = self.grammar.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        let numeral = grammar.next_numeral(ctx.valence, &mut || rng.next_f32());
        let next_chord = Grammar::realize(numeral, ctx.global_key, ctx.valence, grammar.style());
        let parent = self.lcc.parent_lydian(&next_chord);
        let level = self.lcc.level_for_tension(ctx.tension);
        let suggested_scale = self.lcc.get_scale(parent, level);
        HarmonyDecision { next_chord, transition_type: TransitionType::Functional, suggested_scale }
    }
    fn name(&self) -> &'static str {
        "Steedman Grammar"
    }
}
