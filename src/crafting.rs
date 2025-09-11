use std::collections::HashSet;

use itertools::Itertools;
use random_choice::random_choice;

use crate::{
    MODS, TIERS,
    types::{Affix, ModFamily, TierId},
};

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

/// Filter mods which can roll using a "Perfect" currency
pub fn filter_perfect(candidate_tiers: &[TierId]) -> Vec<TierId> {
    filter_better_currency(candidate_tiers, 50)
}
/// Filter mods which can roll using a "Greater" currency
pub fn filter_greater(candidate_tiers: &[TierId]) -> Vec<TierId> {
    filter_better_currency(candidate_tiers, 35)
}

/// Minimum Modifier Level: Added random modifiers are at least this level or higher,
/// except if a specific modifier type would be excluded entirely from being able to roll.
/// In other words, at least one tier of each mod will always be eligible to roll, respecting item level.
/// For example, if all tiers of a type of a modifier would be excluded, and the highest modifier tier
/// is below Level 35 (e.g. Light Radius), the highest tier of Light Radius (requiring Level 30) would still be able to roll.
fn filter_better_currency(candidate_tiers: &[TierId], min_ilvl: u32) -> Vec<TierId> {
    let tiers = TIERS.get().unwrap();

    let mut candidate_tiers = candidate_tiers
        .iter()
        .map(|tier_id| &tiers[tier_id])
        .collect::<Vec<_>>();

    // Group by mod group
    candidate_tiers.sort_by_key(|t| &t.mod_id);
    let tier_groups = candidate_tiers.into_iter().chunk_by(|t| &t.mod_id);

    tier_groups
        .into_iter()
        .flat_map(|(_, group)| {
            let group_tiers = group.collect::<Vec<_>>();
            // Filter tiers by ilvl
            let mut filtered = group_tiers
                .iter()
                .filter(|t| t.ilvl >= min_ilvl)
                .collect::<Vec<_>>();

            // If there's none left, take the highest instead
            if filtered.is_empty() {
                filtered = group_tiers
                    .iter()
                    .max_by_key(|t| t.ilvl)
                    .into_iter()
                    .collect::<Vec<_>>();
            }

            filtered.iter().map(|t| t.id.clone()).collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

/// For Sinistral/Dextral Omens
pub fn filter_affix<'a, I: Iterator<Item = &'a TierId>>(
    candidate_mods: I,
    affix: Affix,
) -> impl Iterator<Item = &'a TierId> {
    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();

    candidate_mods.filter(move |tier_id| {
        let tier = &tiers[*tier_id];
        let modifier = &mods[&tier.mod_id];

        modifier.affix == affix
    })
}

/// For Homogenising Omen
pub fn filter_tags<'a, I: Iterator<Item = &'a TierId>>(
    candidate_mods: I,
    tags: HashSet<TierId>,
) -> impl Iterator<Item = &'a TierId> {
    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();

    candidate_mods.filter(move |tier_id| {
        let tier = &tiers[*tier_id];
        let modifier = &mods[&tier.mod_id];

        !modifier.tags.is_disjoint(&tags)
    })
}

/// For Whittling Omen
fn filter_lowest_tier(candidate_mods: &[TierId]) -> Vec<TierId> {
    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();

    // TODO: Need base-specific tiers
    todo!()
}

/// Removes tiers which conflict with the given families
pub fn filter_out_families<'a, I: Iterator<Item = &'a TierId>>(
    candidate_mods: I,
    families: HashSet<ModFamily>,
) -> impl Iterator<Item = &'a TierId> {
    let mods = MODS.get().unwrap();
    let tiers = TIERS.get().unwrap();

    candidate_mods.filter(move |tier_id| {
        let tier = &tiers[*tier_id];
        let modifier = &mods[&tier.mod_id];

        !families.contains(&modifier.family)
    })
}
