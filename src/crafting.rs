use random_choice::random_choice;

use crate::{TIERS, types::TierId};

/// Randomly roll a mod from the given pool
pub fn roll_mod(candidate_tiers: &[TierId]) -> TierId {
    let tiers = TIERS.get().unwrap();

    let mut outcomes = vec![];
    let mut weights = vec![];
    for tier_id in candidate_tiers {
        let tier = &tiers[tier_id];
        outcomes.push(tier_id);
        weights.push(tier.weight as f32);
    }

    let choices = random_choice().random_choice_f32(&outcomes, &weights, 1);
    let choice = choices.first().unwrap();

    (**choice).clone()
}
