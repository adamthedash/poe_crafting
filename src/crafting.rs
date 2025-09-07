use random_choice::random_choice;

use crate::parser::{Modifier, Tier};

/// Randomly roll a mod from the given pool
pub fn roll_mod<'a>(mods: &[(&'a Modifier, Vec<&'a Tier>)]) -> (&'a Modifier, &'a Tier) {
    let mut outcomes = vec![];
    let mut weights = vec![];
    for (modifier, tiers) in mods {
        for tier in tiers {
            outcomes.push((modifier, tier));
            weights.push(tier.weighting as f32);
        }
    }

    let choices = random_choice().random_choice_f32(&outcomes, &weights, 1);
    let choice = choices.first().unwrap();

    (choice.0, choice.1)
}
