use std::{collections::HashSet, ops::RangeInclusive};

use crate::{
    TIERS_HV,
    currency::CurrencyType,
    hashvec::OpaqueIndex,
    item_state::{ItemState, Rarity},
    types::{Modifier, Omen, Tier},
};

/// Eg. LocalAttackSpeed T2-T1
#[derive(Clone, Debug)]
pub struct ModifierCondition {
    pub mod_group: OpaqueIndex<Modifier>,
    pub levels: Vec<u32>,
}

impl ModifierCondition {
    pub fn check(&self, tier: &Tier) -> bool {
        tier.mod_id == self.mod_group && self.levels.contains(&tier.ilvl)
    }
}

#[derive(Clone, Debug)]
pub enum ConditionGroup {
    Count {
        count: RangeInclusive<usize>,
        mods: Vec<ModifierCondition>,
    },
    /// None of these
    Not(HashSet<OpaqueIndex<Modifier>>),
    AffixCount {
        suffixes: RangeInclusive<usize>,
        prefixes: RangeInclusive<usize>,
        affixes: RangeInclusive<usize>,
    },
}
impl ConditionGroup {
    pub fn check(&self, item: &ItemState) -> bool {
        let tiers = TIERS_HV.get().unwrap();

        let item_tiers = item
            .mods
            .iter()
            .map(|&tier_id| &tiers[tier_id])
            .collect::<Vec<_>>();

        match self {
            ConditionGroup::Count { count, mods } => {
                let num_passed = mods
                    .iter()
                    .filter(|cond| item_tiers.iter().any(|tier| cond.check(tier)))
                    .count();

                count.contains(&num_passed)
            }
            ConditionGroup::Not(mod_groups) => {
                let item_mod_groups = item_tiers
                    .iter()
                    .map(|tier| tier.mod_id)
                    .collect::<HashSet<_>>();

                mod_groups.is_disjoint(&item_mod_groups)
            }
            ConditionGroup::AffixCount {
                suffixes,
                prefixes,
                affixes,
            } => {
                let item_prefixes = item.num_prefixes();
                let item_suffixes = item.num_suffixes();

                prefixes.contains(&item_prefixes)
                    && suffixes.contains(&item_suffixes)
                    && affixes.contains(&(item_prefixes + item_suffixes))
            }
        }
    }
}

/// Represents the state of an item.
#[derive(Debug)]
pub struct Condition {
    pub rarity: Rarity,
    /// All of these groups must be true
    pub groups: Vec<ConditionGroup>,
}

impl Condition {
    pub fn check(&self, item: &ItemState) -> bool {
        item.rarity == self.rarity && self.groups.iter().all(|group| group.check(item))
    }
}

#[derive(Debug)]
pub struct Strategy(pub Vec<(Condition, Option<(HashSet<Omen>, CurrencyType)>)>);

impl Strategy {
    /// Select a crafting method given the item's current state
    pub fn get_craft(&self, item: &ItemState) -> Option<&(HashSet<Omen>, CurrencyType)> {
        let matching_states = self
            .0
            .iter()
            .filter(|(cond, _)| cond.check(item))
            .collect::<Vec<_>>();

        assert!(!matching_states.is_empty(), "No matching states!");

        // Always return the first match
        matching_states[0].1.as_ref()
    }
}
