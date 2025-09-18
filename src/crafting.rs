use std::collections::HashSet;

use itertools::Itertools;

use crate::{
    MODS_HV, TIERS_HV,
    hashvec::OpaqueIndex,
    types::{Affix, ModFamily, Tier, TierId},
};

/// Minimum Modifier Level: Added random modifiers are at least this level or higher,
/// except if a specific modifier type would be excluded entirely from being able to roll.
/// In other words, at least one tier of each mod will always be eligible to roll, respecting item level.
/// For example, if all tiers of a type of a modifier would be excluded, and the highest modifier tier
/// is below Level 35 (e.g. Light Radius), the highest tier of Light Radius (requiring Level 30) would still be able to roll.
pub fn filter_better_currency(
    candidate_tiers: &[OpaqueIndex<Tier>],
    min_ilvl: u32,
) -> Vec<OpaqueIndex<Tier>> {
    let tiers = TIERS_HV.get().unwrap();

    // Group by mod group
    let tier_groups = candidate_tiers
        .iter()
        .copied()
        .sorted_unstable_by_key(|&t| &tiers[t].mod_id)
        .chunk_by(|&t| &tiers[t].mod_id);

    tier_groups
        .into_iter()
        .flat_map(|(_, group)| {
            let group_tiers = group.collect::<Vec<_>>();
            // Filter tiers by ilvl
            let mut filtered = group_tiers
                .iter()
                .copied()
                .filter(|&t| tiers[t].ilvl >= min_ilvl)
                .collect::<Vec<_>>();

            // If there's none left, take the highest instead
            if filtered.is_empty() {
                filtered = group_tiers
                    .iter()
                    .copied()
                    .max_by_key(|&t| tiers[t].ilvl)
                    .into_iter()
                    .collect::<Vec<_>>();
            }

            filtered
        })
        .collect::<Vec<_>>()
}

/// For Sinistral/Dextral Omens
pub fn filter_affix<I: Iterator<Item = OpaqueIndex<Tier>>>(
    candidate_mods: I,
    affix: Affix,
) -> impl Iterator<Item = OpaqueIndex<Tier>> {
    let tiers = TIERS_HV.get().unwrap();

    candidate_mods.filter(move |&tier_id| {
        let tier = &tiers[tier_id];

        tier.affix == affix
    })
}

/// For Homogenising Omen
pub fn filter_tags<I: Iterator<Item = OpaqueIndex<Tier>>>(
    candidate_mods: I,
    tags: HashSet<TierId>,
) -> impl Iterator<Item = OpaqueIndex<Tier>> {
    let tiers = TIERS_HV.get().unwrap();
    let mods = MODS_HV.get().unwrap();

    candidate_mods.filter(move |&tier_id| {
        let tier = &tiers[tier_id];
        let modifier = &mods[tier.mod_id];

        !modifier.tags.is_disjoint(&tags)
    })
}

/// For Whittling Omen
pub fn filter_lowest_tier<I: Iterator<Item = OpaqueIndex<Tier>>>(
    candidate_mods: I,
) -> impl Iterator<Item = OpaqueIndex<Tier>> {
    let tiers = TIERS_HV.get().unwrap();

    let candidate_mods = candidate_mods.collect::<Vec<_>>();
    let min_ilvl = candidate_mods
        .iter()
        .map(|&tier_id| tiers[tier_id].ilvl)
        .min()
        // If candidate_mods is empty, this value doesn't matter anyway
        .unwrap_or(0);

    candidate_mods
        .into_iter()
        .filter(move |&tier_id| tiers[tier_id].ilvl == min_ilvl)
}

/// Removes tiers which conflict with the given families
pub fn filter_out_families<I: Iterator<Item = OpaqueIndex<Tier>>>(
    candidate_mods: I,
    families: HashSet<ModFamily>,
) -> impl Iterator<Item = OpaqueIndex<Tier>> {
    let mods = MODS_HV.get().unwrap();
    let tiers = TIERS_HV.get().unwrap();

    candidate_mods.filter(move |&tier_id| {
        let tier = &tiers[tier_id];
        let modifier = &mods[tier.mod_id];

        !families.contains(&modifier.family)
    })
}
