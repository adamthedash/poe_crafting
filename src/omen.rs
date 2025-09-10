use std::collections::HashSet;

/**
*   Omens do one of these things:
*   1) Modify the list of mods to be removed
*   2) Modify the list of mods to be added
*   3) Modify the number of mods to be added
*/
use crate::{
    MODS, TIERS,
    types::{Affix, ModTag, TierId},
};

/// For Sinistral/Dextral
fn filter_affix(candidate_mods: &[TierId], affix: Affix) -> Vec<TierId> {
    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();

    candidate_mods
        .iter()
        .filter(|tier_id| {
            let tier = &tiers[*tier_id];
            let modifier = &mods[&tier.mod_id];

            modifier.affix == affix
        })
        .cloned()
        .collect()
}

/// For Homogenising
fn filter_tags(candidate_mods: &[TierId], tags: HashSet<ModTag>) -> Vec<TierId> {
    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();

    candidate_mods
        .iter()
        .filter(|tier_id| {
            let tier = &tiers[*tier_id];
            let modifier = &mods[&tier.mod_id];

            modifier.tags.iter().any(|tag| tags.contains(tag))
        })
        .cloned()
        .collect()
}

/// For Whittling
fn filter_lowest_tier(candidate_mods: &[TierId]) -> Vec<TierId> {
    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();

    // TODO: Need base-specific tiers
    todo!()
}

/// Next annul hits prefixes only
pub struct SinistralAnnulment;
impl SinistralAnnulment {
    pub fn filter_mods(candidate_mods: &[TierId]) -> Vec<TierId> {
        filter_affix(candidate_mods, Affix::Prefix)
    }
}

/// Next annul hits suffixes only
pub struct DextralAnnulment;
impl DextralAnnulment {
    pub fn filter_mods(candidate_mods: &[TierId]) -> Vec<TierId> {
        filter_affix(candidate_mods, Affix::Suffix)
    }
}

/// Next regal hits prefixes only
pub struct SinistralCoronation;
impl SinistralCoronation {
    pub fn filter_mods(candidate_mods: &[TierId]) -> Vec<TierId> {
        filter_affix(candidate_mods, Affix::Prefix)
    }
}

/// Next regal hits suffixes only
pub struct DextralCoronation;
impl DextralCoronation {
    pub fn filter_mods(candidate_mods: &[TierId]) -> Vec<TierId> {
        filter_affix(candidate_mods, Affix::Suffix)
    }
}

/// Next perfect/corrupted essence removes prefixes only
pub struct SinistralCrystallisation;
impl SinistralCrystallisation {
    pub fn filter_mods(candidate_mods: &[TierId]) -> Vec<TierId> {
        filter_affix(candidate_mods, Affix::Prefix)
    }
}

/// Next perfect/corrupted essence removes suffixes only
pub struct DextralCrystallisation;
impl DextralCrystallisation {
    pub fn filter_mods(candidate_mods: &[TierId]) -> Vec<TierId> {
        filter_affix(candidate_mods, Affix::Suffix)
    }
}
