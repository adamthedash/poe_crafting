use crate::types::Affix;
use crate::types::Modifier;
use crate::types::Tier;
use crate::{
    MODS, TIERS,
    item_state::{ItemState, Rarity},
    types::TierId,
};

pub trait Currency {
    /// Whether this currency can currently be used on the given item
    fn can_be_used(item: &ItemState) -> bool;

    /// Gets the pool of mods that can roll if this currency is used.
    /// all_mods: The pool of mods that can possibly roll on this item
    fn possible_tiers<'a>(item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId>;
}

pub struct Transmute;

impl Currency for Transmute {
    fn can_be_used(item: &ItemState) -> bool {
        item.rarity == Rarity::Normal
    }

    fn possible_tiers<'a>(_item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        // TODO: Filter out non-standard mods, such as essences or abyss
        candidate_tiers.to_vec()
    }
}

pub struct Augmentation;

impl Currency for Augmentation {
    fn can_be_used(item: &ItemState) -> bool {
        item.rarity == Rarity::Magic && item.mods.len() < 2
    }

    fn possible_tiers<'a>(item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let mods = MODS.get().unwrap();
        let tiers = TIERS.get().unwrap();

        let num_prefixes = item.num_prefixes();
        let num_suffixes = item.num_suffixes();

        candidate_tiers
            .iter()
            .filter(|tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[&tier.modifier];

                modifier.affix == Affix::Prefix && num_prefixes < 1
                    || modifier.affix == Affix::Suffix && num_suffixes < 1
            })
            .cloned()
            .collect()
    }
}

pub struct Regal;

impl Currency for Regal {
    fn can_be_used(item: &ItemState) -> bool {
        item.rarity == Rarity::Magic
    }

    fn possible_tiers<'a>(item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let mods = MODS.get().unwrap();
        let tiers = TIERS.get().unwrap();

        let existing_mod_groups = item
            .mods
            .iter()
            .map(|tier_id| {
                let tier = &tiers[tier_id];
                &mods[&tier.modifier].family
            })
            .collect::<Vec<_>>();

        candidate_tiers
            .iter()
            // Can't roll mods from the same mod group
            .filter(|tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[&tier.modifier];

                !existing_mod_groups.contains(&&modifier.family)
            })
            .cloned()
            .collect()
    }
}

pub struct Exalt;

impl Currency for Exalt {
    fn can_be_used(item: &ItemState) -> bool {
        item.rarity == Rarity::Rare && item.mods.len() < 6
    }

    fn possible_tiers<'a>(item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let mods = MODS.get().unwrap();
        let tiers = TIERS.get().unwrap();

        let num_prefixes = item.num_prefixes();
        let num_suffixes = item.num_suffixes();

        let existing_mod_groups = item
            .mods
            .iter()
            .map(|tier_id| {
                let tier = &tiers[tier_id];
                &mods[&tier.modifier].family
            })
            .collect::<Vec<_>>();

        candidate_tiers
            .iter()
            .filter(|tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[&tier.modifier];

                let has_space = modifier.affix == Affix::Prefix && num_prefixes < 3
                    || modifier.affix == Affix::Suffix && num_suffixes < 3;

                has_space && !existing_mod_groups.contains(&&modifier.family)
            })
            .cloned()
            .collect()
    }
}
